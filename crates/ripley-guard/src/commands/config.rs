use ripley_core::config;

pub fn cmd_config(path: bool, show: bool, init: bool) -> anyhow::Result<()> {
    let config_path = config::config_file_path()?;

    if path {
        println!("{}", config_path.display());
        return Ok(());
    }

    if init {
        if config_path.exists() {
            println!("Config already exists at {}", config_path.display());
        } else {
            config::write_default_config(&config_path)?;
            println!("Created default config at {}", config_path.display());
        }
        return Ok(());
    }

    if show {
        let cwd = std::env::current_dir()?;
        let cfg = config::load_config(&cwd)?;
        let toml_str = toml::to_string_pretty(&cfg)?;
        println!("{toml_str}");
        return Ok(());
    }

    let editor = std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| "open".to_string());

    if !config_path.exists() {
        config::write_default_config(&config_path)?;
    }

    std::process::Command::new(&editor)
        .arg(&config_path)
        .status()?;

    Ok(())
}
