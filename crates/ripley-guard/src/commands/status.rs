use ripley_core::config;
use ripley_core::db::AdvisoryDb;

pub fn cmd_status() -> anyhow::Result<()> {
    let data_dir = ripley_core::dirs::data_dir()?;
    let db_path = data_dir.join("advisories.redb");

    let cache_age = if db_path.exists() {
        let db = AdvisoryDb::open(&db_path)?;
        match db.get_last_poll()? {
            Some(ts) => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let age_secs = now.saturating_sub(ts);
                if age_secs < 60 {
                    format!("{age_secs}s ago")
                } else if age_secs < 3600 {
                    format!("{}m ago", age_secs / 60)
                } else if age_secs < 86400 {
                    format!("{}h ago", age_secs / 3600)
                } else {
                    format!("{}d ago", age_secs / 86400)
                }
            }
            None => "never".to_string(),
        }
    } else {
        "never".to_string()
    };

    let shim_path = data_dir.join("bin").join("npm");
    let shim_status = if shim_path.exists() {
        "installed"
    } else {
        "not installed"
    };

    let cwd = std::env::current_dir()?;
    let cfg = config::load_config(&cwd)?;
    let roots = if cfg.monitoring.project_roots.is_empty() {
        "none configured".to_string()
    } else {
        cfg.monitoring.project_roots.join(", ")
    };

    println!("Ripley Status");
    println!("─────────────────────────────");
    println!("  Advisory cache:   {cache_age}");
    println!("  Guard shim:       {shim_status}");
    println!("  Project roots:    {roots}");
    println!("  Daemon:           not running");

    Ok(())
}
