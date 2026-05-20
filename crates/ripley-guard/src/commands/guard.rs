use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

use colored::Colorize;

use ripley_core::config;
use ripley_core::dirs;

pub fn cmd_install() -> anyhow::Result<()> {
    let data_dir = dirs::data_dir()?;
    let bin_dir = data_dir.join("bin");
    std::fs::create_dir_all(&bin_dir)?;

    let self_exe = std::env::current_exe()?;
    let self_dir = self_exe
        .parent()
        .ok_or_else(|| anyhow::anyhow!("could not determine binary directory"))?;

    let npm_shim_src = self_dir.join("ripley-npm-shim");
    let script_shell_src = self_dir.join("ripley-script-shell");

    let npm_shim_dst = bin_dir.join("npm");
    let script_shell_dst = bin_dir.join("ripley-script-shell");

    copy_binary(&npm_shim_src, &npm_shim_dst)?;
    copy_binary(&script_shell_src, &script_shell_dst)?;

    let rc_path = detect_shell_rc()?;
    let path_line = format!("export PATH=\"{}:$PATH\"", bin_dir.display());
    let marker = "# ripley guard";
    install_rc_line(&rc_path, &path_line, marker)?;

    let npmrc_path = home_dir()?.join(".npmrc");
    let script_shell_line = format!("script-shell={}", script_shell_dst.display());
    install_npmrc_setting(&npmrc_path, "script-shell", &script_shell_line)?;

    println!("{}", "Guard installed successfully.".green());
    println!();
    println!("  npm shim:       {}", npm_shim_dst.display());
    println!("  script-shell:   {}", script_shell_dst.display());
    println!("  shell RC:       {}", rc_path.display());
    println!("  npmrc:          {}", npmrc_path.display());
    println!();
    println!(
        "Restart your shell or run: {}",
        format!("source {}", rc_path.display()).dimmed()
    );

    Ok(())
}

pub fn cmd_uninstall() -> anyhow::Result<()> {
    let data_dir = dirs::data_dir()?;
    let bin_dir = data_dir.join("bin");

    let npm_shim = bin_dir.join("npm");
    let script_shell = bin_dir.join("ripley-script-shell");

    if npm_shim.exists() {
        std::fs::remove_file(&npm_shim)?;
    }
    if script_shell.exists() {
        std::fs::remove_file(&script_shell)?;
    }

    if let Ok(rc_path) = detect_shell_rc() {
        let marker = "# ripley guard";
        remove_rc_line(&rc_path, marker)?;
    }

    let npmrc_path = home_dir()?.join(".npmrc");
    if npmrc_path.exists() {
        remove_npmrc_setting(&npmrc_path, "script-shell")?;
    }

    println!("{}", "Guard uninstalled.".green());
    Ok(())
}

pub fn cmd_status() -> anyhow::Result<()> {
    let data_dir = dirs::data_dir()?;
    let bin_dir = data_dir.join("bin");

    let npm_shim = bin_dir.join("npm");
    let script_shell = bin_dir.join("ripley-script-shell");

    let npm_status = if npm_shim.exists() {
        "installed".green().to_string()
    } else {
        "not installed".yellow().to_string()
    };

    let shell_status = if script_shell.exists() {
        "installed".green().to_string()
    } else {
        "not installed".yellow().to_string()
    };

    let npmrc_path = home_dir()?.join(".npmrc");
    let npmrc_active = if npmrc_path.exists() {
        let content = std::fs::read_to_string(&npmrc_path).unwrap_or_default();
        content.lines().any(|l| l.starts_with("script-shell="))
    } else {
        false
    };
    let npmrc_status = if npmrc_active {
        "active".green().to_string()
    } else {
        "inactive".yellow().to_string()
    };

    let cwd = std::env::current_dir()?;
    let cfg = config::load_config(&cwd)?;
    let trust_count = cfg.guard.trust.len();

    println!("Guard Status");
    println!("─────────────────────────────");
    println!("  npm shim:         {npm_status}");
    println!("  script-shell:     {shell_status}");
    println!("  npmrc hook:       {npmrc_status}");
    println!("  trusted packages: {trust_count}");

    Ok(())
}

pub fn cmd_trust(package: &str) -> anyhow::Result<()> {
    let config_path = config::config_file_path()?;
    let cwd = std::env::current_dir()?;
    let mut cfg = config::load_config(&cwd)?;

    if cfg.guard.trust.iter().any(|p| p == package) {
        println!("{package} is already trusted");
        return Ok(());
    }

    cfg.guard.trust.push(package.to_string());
    write_config_atomic(&config_path, &cfg)?;

    println!("{} {package}", "Trusted:".green());
    Ok(())
}

pub fn cmd_untrust(package: &str) -> anyhow::Result<()> {
    let config_path = config::config_file_path()?;
    let cwd = std::env::current_dir()?;
    let mut cfg = config::load_config(&cwd)?;

    let before = cfg.guard.trust.len();
    cfg.guard.trust.retain(|p| p != package);

    if cfg.guard.trust.len() == before {
        println!("{package} was not in the trust list");
        return Ok(());
    }

    write_config_atomic(&config_path, &cfg)?;

    println!("{} {package}", "Untrusted:".green());
    Ok(())
}

pub fn cmd_log() -> anyhow::Result<()> {
    let data_dir = dirs::data_dir()?;
    let log_path = data_dir.join("guard.jsonl");

    if !log_path.exists() {
        println!("No guard log entries yet.");
        return Ok(());
    }

    let file = std::fs::File::open(&log_path)?;
    let reader = std::io::BufReader::new(file);
    let lines: Vec<String> = reader.lines().collect::<Result<Vec<_>, _>>()?;
    let recent: Vec<&String> = lines.iter().rev().take(20).collect::<Vec<_>>();

    if recent.is_empty() {
        println!("No guard log entries yet.");
        return Ok(());
    }

    println!(
        "{:<24} {:<10} {:<10} {:<10} Rules",
        "Timestamp", "Action", "Risk", "Source"
    );
    println!("{}", "─".repeat(72));

    for line in recent.iter().rev() {
        if let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) {
            let ts = entry["timestamp"].as_str().unwrap_or("-");
            let action = entry["action"].as_str().unwrap_or("-");
            let risk = entry["risk_level"].as_str().unwrap_or("-");
            let source = entry["source"].as_str().unwrap_or("-");
            let rules = entry["matched_rules"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default();

            let action_padded = format!("{action:<10}");
            let action_display = match action {
                "blocked" => action_padded.red().to_string(),
                "allowed" => action_padded.green().to_string(),
                _ => action_padded,
            };

            println!("{ts:<24} {action_display} {risk:<10} {source:<10} {rules}");
        }
    }

    Ok(())
}

fn copy_binary(src: &Path, dst: &Path) -> anyhow::Result<()> {
    if !src.exists() {
        anyhow::bail!(
            "binary not found: {} — build with `cargo build -p ripley-guard`",
            src.display()
        );
    }
    let temp = dst.with_extension("tmp");
    std::fs::copy(src, &temp)?;
    std::fs::rename(&temp, dst)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(dst, std::fs::Permissions::from_mode(0o755))?;
    }
    Ok(())
}

fn home_dir() -> anyhow::Result<PathBuf> {
    std::env::var("HOME")
        .map(PathBuf::from)
        .map_err(|_| anyhow::anyhow!("could not determine home directory (HOME not set)"))
}

fn detect_shell_rc() -> anyhow::Result<PathBuf> {
    let home = home_dir()?;
    let shell = std::env::var("SHELL").unwrap_or_default();

    if shell.ends_with("zsh") {
        return Ok(home.join(".zshrc"));
    }
    if shell.ends_with("bash") {
        let bashrc = home.join(".bashrc");
        if bashrc.exists() {
            return Ok(bashrc);
        }
        return Ok(home.join(".profile"));
    }
    let zshrc = home.join(".zshrc");
    if zshrc.exists() {
        return Ok(zshrc);
    }
    let bashrc = home.join(".bashrc");
    if bashrc.exists() {
        return Ok(bashrc);
    }
    Ok(home.join(".profile"))
}

fn install_rc_line(rc_path: &Path, line: &str, marker: &str) -> anyhow::Result<()> {
    let existing = std::fs::read_to_string(rc_path).unwrap_or_default();
    if existing.contains(marker) {
        return Ok(());
    }

    let entry = format!("{line} {marker}\n");
    let new_content = if existing.ends_with('\n') || existing.is_empty() {
        format!("{existing}{entry}")
    } else {
        format!("{existing}\n{entry}")
    };

    atomic_write(rc_path, new_content.as_bytes())
}

fn remove_rc_line(rc_path: &Path, marker: &str) -> anyhow::Result<()> {
    let existing = match std::fs::read_to_string(rc_path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e.into()),
    };

    let filtered: String = existing
        .lines()
        .filter(|l| !l.contains(marker))
        .collect::<Vec<_>>()
        .join("\n");

    let new_content = if filtered.is_empty() {
        filtered
    } else {
        format!("{filtered}\n")
    };

    atomic_write(rc_path, new_content.as_bytes())
}

fn install_npmrc_setting(path: &Path, key: &str, full_line: &str) -> anyhow::Result<()> {
    let existing = std::fs::read_to_string(path).unwrap_or_default();

    let prefix = format!("{key}=");
    let filtered: Vec<&str> = existing
        .lines()
        .filter(|l| !l.starts_with(&prefix))
        .collect();

    let mut new_lines = filtered;
    new_lines.push(full_line);

    let mut new_content = new_lines.join("\n");
    if !new_content.ends_with('\n') {
        new_content.push('\n');
    }

    atomic_write(path, new_content.as_bytes())
}

fn remove_npmrc_setting(path: &Path, key: &str) -> anyhow::Result<()> {
    let existing = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e.into()),
    };

    let prefix = format!("{key}=");
    let filtered: String = existing
        .lines()
        .filter(|l| !l.starts_with(&prefix))
        .collect::<Vec<_>>()
        .join("\n");

    let new_content = if filtered.is_empty() {
        filtered
    } else {
        format!("{filtered}\n")
    };

    atomic_write(path, new_content.as_bytes())
}

fn atomic_write(path: &Path, data: &[u8]) -> anyhow::Result<()> {
    let temp = path.with_extension("tmp");
    let mut file = std::fs::File::create(&temp)?;
    file.write_all(data)?;
    file.sync_all()?;
    std::fs::rename(&temp, path)?;
    Ok(())
}

fn write_config_atomic(path: &Path, cfg: &config::Config) -> anyhow::Result<()> {
    let toml_str = toml::to_string_pretty(cfg)?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    atomic_write(path, toml_str.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_install_rc_line_idempotent() {
        let dir = tempfile::tempdir().expect("tempdir");
        let rc = dir.path().join(".zshrc");
        std::fs::write(&rc, "existing content\n").expect("write");

        install_rc_line(&rc, "export PATH=\"/test:$PATH\"", "# ripley guard").expect("install");
        let content = std::fs::read_to_string(&rc).expect("read");
        assert!(content.contains("# ripley guard"));

        install_rc_line(&rc, "export PATH=\"/test:$PATH\"", "# ripley guard")
            .expect("install again");
        let content2 = std::fs::read_to_string(&rc).expect("read");
        assert_eq!(
            content2.matches("# ripley guard").count(),
            1,
            "should not duplicate"
        );
    }

    #[test]
    fn test_remove_rc_line() {
        let dir = tempfile::tempdir().expect("tempdir");
        let rc = dir.path().join(".zshrc");
        std::fs::write(
            &rc,
            "line one\nexport PATH=\"/test:$PATH\" # ripley guard\nline three\n",
        )
        .expect("write");

        remove_rc_line(&rc, "# ripley guard").expect("remove");
        let content = std::fs::read_to_string(&rc).expect("read");
        assert!(!content.contains("ripley guard"));
        assert!(content.contains("line one"));
        assert!(content.contains("line three"));
    }

    #[test]
    fn test_install_npmrc_setting() {
        let dir = tempfile::tempdir().expect("tempdir");
        let npmrc = dir.path().join(".npmrc");
        std::fs::write(&npmrc, "registry=https://registry.npmjs.org\n").expect("write");

        install_npmrc_setting(&npmrc, "script-shell", "script-shell=/test/bin/shell")
            .expect("install");
        let content = std::fs::read_to_string(&npmrc).expect("read");
        assert!(content.contains("script-shell=/test/bin/shell"));
        assert!(content.contains("registry="));

        install_npmrc_setting(&npmrc, "script-shell", "script-shell=/updated/path")
            .expect("update");
        let content2 = std::fs::read_to_string(&npmrc).expect("read");
        assert!(content2.contains("script-shell=/updated/path"));
        assert!(!content2.contains("/test/bin/shell"));
        assert_eq!(content2.matches("script-shell=").count(), 1);
    }

    #[test]
    fn test_remove_npmrc_setting() {
        let dir = tempfile::tempdir().expect("tempdir");
        let npmrc = dir.path().join(".npmrc");
        std::fs::write(
            &npmrc,
            "registry=https://registry.npmjs.org\nscript-shell=/test\n",
        )
        .expect("write");

        remove_npmrc_setting(&npmrc, "script-shell").expect("remove");
        let content = std::fs::read_to_string(&npmrc).expect("read");
        assert!(!content.contains("script-shell"));
        assert!(content.contains("registry="));
    }

    #[test]
    fn test_atomic_write() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("test.txt");
        atomic_write(&path, b"hello world").expect("write");
        assert_eq!(std::fs::read_to_string(&path).expect("read"), "hello world");
        assert!(!path.with_extension("tmp").exists());
    }
}
