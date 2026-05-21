use std::path::PathBuf;
use std::process::ExitCode;

use ripley_core::config::{self, Config};
use ripley_core::db::AdvisoryDb;
use ripley_core::feed::osv::OsvClient;
use ripley_core::forensic::credentials;
use ripley_core::forensic::ioc::{self, IocProfileSet};
use ripley_core::forensic::persistence;
use ripley_core::lockfile::{self, InstalledPackage, LockfileWarning, RiskySpec};
use ripley_core::matcher;
use ripley_core::platform;

use crate::output;

pub async fn cmd_scan(
    path: PathBuf,
    format: &str,
    deep: bool,
    fix: bool,
    no_cache: bool,
) -> anyhow::Result<ExitCode> {
    let cwd = std::env::current_dir()?;
    let cfg = config::load_config(&cwd)?;
    let cache_dir = ripley_core::dirs::cache_dir()?;
    let data_dir = ripley_core::dirs::data_dir()?;
    let db_path = data_dir.join("advisories.redb");
    let db = AdvisoryDb::open(&db_path)?;

    let lockfiles = lockfile::find_lockfiles(&path);
    if lockfiles.is_empty() {
        eprintln!("No lockfiles found in {}", path.display());
        return Ok(ExitCode::from(2));
    }

    let mut all_packages = Vec::new();
    let mut all_warnings = Vec::new();
    let mut all_risky = Vec::new();

    for lockfile_path in &lockfiles {
        match lockfile::parse_lockfile(lockfile_path) {
            Ok(parsed) => {
                all_packages.extend(parsed.packages);
                all_warnings.extend(parsed.warnings);
                all_risky.extend(parsed.risky_specs);
            }
            Err(e) => {
                eprintln!("Warning: could not parse {}: {e}", lockfile_path.display());
            }
        }
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let last_poll = db.get_last_poll()?;
    let stale = last_poll
        .map(|lp| now - lp > cfg.feeds.stale_threshold_secs)
        .unwrap_or(true);

    if no_cache || stale {
        let client = match OsvClient::new(cache_dir) {
            Ok(c) => c,
            Err(e) => {
                if last_poll.is_none() {
                    eprintln!(
                        "No cached advisories and network unavailable — \
                         run with network access to populate the cache"
                    );
                    return Ok(ExitCode::from(2));
                }
                eprintln!("Warning: could not create feed client: {e}");
                eprintln!("Using cached advisories (may be stale)");
                return run_matcher_and_output(
                    &db,
                    &all_packages,
                    &all_warnings,
                    &all_risky,
                    &path,
                    format,
                    deep,
                    fix,
                    &cfg,
                );
            }
        };

        let unique_packages: Vec<_> = {
            let mut seen = std::collections::HashSet::new();
            all_packages
                .iter()
                .filter(|p| seen.insert((p.ecosystem, p.name.clone())))
                .map(|p| (p.ecosystem, p.name.clone()))
                .collect()
        };

        if !unique_packages.is_empty() {
            match client.query_batch(&unique_packages).await {
                Ok(advisories) => {
                    let mut by_key: std::collections::HashMap<_, Vec<_>> =
                        std::collections::HashMap::new();
                    for adv in &advisories {
                        by_key
                            .entry((adv.ecosystem, adv.package.clone()))
                            .or_default()
                            .push(adv.clone());
                    }
                    for ((eco, pkg), advs) in &by_key {
                        db.store_advisories(*eco, pkg, advs)?;
                    }
                    db.set_last_poll(now)?;
                }
                Err(e) => {
                    if last_poll.is_none() {
                        eprintln!(
                            "No cached advisories and network unavailable — \
                             run with network access to populate the cache"
                        );
                        return Ok(ExitCode::from(2));
                    }
                    eprintln!("Warning: could not fetch advisories: {e}");
                    eprintln!("Using cached advisories (may be stale)");
                }
            }
        }
    }

    run_matcher_and_output(
        &db,
        &all_packages,
        &all_warnings,
        &all_risky,
        &path,
        format,
        deep,
        fix,
        &cfg,
    )
}

#[allow(clippy::too_many_arguments)]
fn run_matcher_and_output(
    db: &AdvisoryDb,
    packages: &[InstalledPackage],
    warnings: &[LockfileWarning],
    risky_specs: &[RiskySpec],
    path: &std::path::Path,
    format: &str,
    deep: bool,
    fix: bool,
    cfg: &Config,
) -> anyhow::Result<ExitCode> {
    let all_advisories = db.get_all_advisories()?;
    let matches = matcher::find_matches(&all_advisories, packages, path);

    let deep_report = if deep {
        Some(run_deep_scan(path)?)
    } else {
        None
    };

    match format {
        "json" => {
            if let Some(ref report) = deep_report {
                output::print_deep_json(&matches, warnings, risky_specs, report)?;
            } else {
                output::print_json(&matches, warnings, risky_specs)?;
            }
        }
        _ => {
            output::print_table(&matches, warnings, risky_specs);
            if let Some(ref report) = deep_report {
                output::print_deep_table(report);
            }
        }
    }

    if fix && !matches.is_empty() {
        let prompt = ripley_core::prompt::generate_batch_prompt(&matches);

        match ripley_core::harness::detect_harness() {
            Some(harness) => {
                eprintln!("\nripley: launching {} with remediation prompt...", harness);
                match ripley_core::harness::launch(&harness, &prompt, path) {
                    Ok(_) => eprintln!("ripley: harness launched"),
                    Err(e) => eprintln!("ripley: failed to launch harness: {e}"),
                }
            }
            None => {
                eprintln!("\n--- Remediation Prompt ---\n");
                eprintln!("{prompt}");
                eprintln!("--- End Prompt ---\n");
                eprintln!(
                    "No AI harness found in PATH (claude, codex, opencode).\n\
                     Copy the prompt above and paste it into your preferred tool."
                );
            }
        }
    }

    let has_matches = !matches.is_empty();
    let strict_posture_fail =
        cfg.posture.strict && (!warnings.is_empty() || !risky_specs.is_empty());
    let has_deep_findings = deep_report.as_ref().is_some_and(|r| {
        !r.ioc_findings.is_empty()
            || !r.persistence_findings.is_empty()
            || !r.exposure.findings.is_empty()
    });

    if has_matches || strict_posture_fail || has_deep_findings {
        Ok(ExitCode::from(1))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

pub struct DeepScanReport {
    pub ioc_findings: Vec<ripley_core::forensic::IocFinding>,
    pub persistence_findings: Vec<ripley_core::forensic::PersistenceFinding>,
    pub exposure: ripley_core::forensic::ExposureReport,
}

fn run_deep_scan(path: &std::path::Path) -> anyhow::Result<DeepScanReport> {
    let config_dir = ripley_core::dirs::config_dir()?;
    let iocs_dir = config_dir.join("iocs");

    let compiled = IocProfileSet::load_compiled()?;
    let user = IocProfileSet::load_user_profiles(&iocs_dir)?;
    let all_profiles = IocProfileSet::merge(compiled, user);
    let profiles = all_profiles.profiles();

    let ioc_findings = ioc::scan_iocs(path, profiles);

    let home = platform::home_dir().unwrap_or_else(|| path.to_path_buf());
    let persistence_findings = persistence::audit_persistence(&home);

    let exposure = credentials::assess_exposure(profiles, &home);

    Ok(DeepScanReport {
        ioc_findings,
        persistence_findings,
        exposure,
    })
}
