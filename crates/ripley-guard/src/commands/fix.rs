use std::path::PathBuf;
use std::process::ExitCode;

use ripley_core::db::AdvisoryDb;
use ripley_core::forensic::ioc::IocProfileSet;
use ripley_core::lockfile;
use ripley_core::matcher;
use ripley_core::prompt;

pub async fn cmd_fix(cve_id: &str, path: Option<PathBuf>) -> anyhow::Result<ExitCode> {
    let cwd = std::env::current_dir()?;
    let target = path.unwrap_or_else(|| cwd.clone());

    let data_dir = ripley_core::dirs::data_dir()?;
    let db_path = data_dir.join("advisories.redb");
    let db = AdvisoryDb::open(&db_path)?;

    let advisories = db.get_all_advisories()?;
    let cve_advisories: Vec<_> = advisories
        .into_iter()
        .filter(|a| a.id.contains(cve_id) || a.references.iter().any(|r| r.contains(cve_id)))
        .collect();

    if cve_advisories.is_empty() {
        eprintln!("No advisories found matching {cve_id} in local cache.");
        eprintln!("Run `ripley scan` first to populate the advisory database.");
        return Ok(ExitCode::from(1));
    }

    let lockfiles = lockfile::find_lockfiles(&target);
    if lockfiles.is_empty() {
        eprintln!("No lockfiles found in {}", target.display());
        return Ok(ExitCode::from(2));
    }

    let mut all_packages = Vec::new();
    for lockfile_path in &lockfiles {
        match lockfile::parse_lockfile(lockfile_path) {
            Ok(parsed) => all_packages.extend(parsed.packages),
            Err(e) => eprintln!("Warning: could not parse {}: {e}", lockfile_path.display()),
        }
    }

    let matches = matcher::find_matches(&cve_advisories, &all_packages, &target);

    if matches.is_empty() {
        println!(
            "No packages affected by {cve_id} found in {}",
            target.display()
        );
        return Ok(ExitCode::from(1));
    }

    let ioc_profiles = IocProfileSet::load_compiled().ok();
    let profiles_slice = ioc_profiles.as_ref().map(|ps| ps.profiles());

    let fix_prompt = prompt::generate_cve_fix_prompt(cve_id, &matches, profiles_slice);

    match ripley_core::harness::detect_harness() {
        Some(harness) => {
            println!(
                "Found {} affected package(s). Launching {} with fix prompt...",
                matches.len(),
                harness.name()
            );
            ripley_core::harness::launch(&harness, &fix_prompt, &cwd)?;
        }
        None => {
            println!(
                "Found {} affected package(s). No AI harness detected.\n",
                matches.len()
            );
            println!("{fix_prompt}");
        }
    }

    Ok(ExitCode::SUCCESS)
}
