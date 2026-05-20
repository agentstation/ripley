use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if is_install_command(&args)
        && let Some(package) = extract_package_name(&args)
    {
        check_advisories(&package);
    }

    let real_npm = match find_real_npm() {
        Some(path) => path,
        None => {
            eprintln!("ripley: could not find real npm in PATH");
            return ExitCode::from(2);
        }
    };

    let status = Command::new(&real_npm).args(&args).status();

    match status {
        Ok(s) => ExitCode::from(s.code().unwrap_or(1) as u8),
        Err(e) => {
            eprintln!("ripley: could not execute npm: {e}");
            ExitCode::from(2)
        }
    }
}

fn is_install_command(args: &[String]) -> bool {
    args.first()
        .is_some_and(|cmd| matches!(cmd.as_str(), "install" | "i" | "add" | "ci"))
}

fn extract_package_name(args: &[String]) -> Option<String> {
    args.iter().skip(1).find(|a| !a.starts_with('-')).cloned()
}

fn check_advisories(package: &str) {
    let data_dir = match ripley_core::dirs::data_dir() {
        Ok(d) => d,
        Err(_) => return,
    };

    let db_path = data_dir.join("advisories.redb");
    if !db_path.exists() {
        return;
    }

    let db = match ripley_core::db::AdvisoryDb::open(&db_path) {
        Ok(db) => db,
        Err(_) => return,
    };

    let advisories = match db.get_advisories(ripley_core::types::Ecosystem::Npm, package) {
        Ok(a) => a,
        Err(_) => return,
    };

    if !advisories.is_empty() {
        eprintln!(
            "ripley: warning — {package} has {} known advisories:",
            advisories.len()
        );
        for adv in &advisories {
            let severity = adv
                .severity
                .as_ref()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            eprintln!("  [{severity}] {} — {}", adv.id, adv.summary);
        }
        eprintln!();
    }
}

// `which -a` lists all matches in PATH; macOS/GNU coreutils only (Phase 1 = macOS).
fn find_real_npm() -> Option<String> {
    let self_exe = std::env::current_exe().ok()?;
    let self_dir = self_exe.parent()?;

    let output = Command::new("which").arg("-a").arg("npm").output().ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .find(|line| {
            let path = std::path::Path::new(line.trim());
            path.parent() != Some(self_dir)
        })
        .map(|s| s.trim().to_string())
}
