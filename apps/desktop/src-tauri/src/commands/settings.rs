use std::path::Path;

use ripley_core::config::{self, Config};
use ripley_core::types::GuardMode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct SettingsDto {
    pub poll_interval_secs: u32,
    pub launch_at_login: bool,
    pub monitor_enabled: bool,
    pub monitor_watch_processes: bool,
    pub monitor_watch_persistence: bool,
    pub monitor_watch_lockfiles: bool,
    pub guard_mode: String,
    pub guard_timeout_secs: u32,
    pub guard_sandbox: bool,
    pub posture_strict: bool,
    pub posture_require_lockfile: bool,
}

fn config_to_dto(config: &Config) -> SettingsDto {
    SettingsDto {
        poll_interval_secs: config.general.poll_interval_secs.min(u32::MAX as u64) as u32,
        launch_at_login: config.general.launch_at_login,
        monitor_enabled: config.monitor.enabled,
        monitor_watch_processes: config.monitor.watch_processes,
        monitor_watch_persistence: config.monitor.watch_persistence,
        monitor_watch_lockfiles: config.monitor.watch_lockfiles,
        guard_mode: config.guard.mode.to_string(),
        guard_timeout_secs: config.guard.timeout_secs.min(u32::MAX as u64) as u32,
        guard_sandbox: config.guard.sandbox,
        posture_strict: config.posture.strict,
        posture_require_lockfile: config.posture.require_lockfile,
    }
}

fn parse_guard_mode(value: &str) -> Result<GuardMode, String> {
    match value {
        "strict" => Ok(GuardMode::Strict),
        "audit" => Ok(GuardMode::Audit),
        "off" => Ok(GuardMode::Off),
        other => Err(format!("unknown guard mode: {other}")),
    }
}

fn apply_dto(config: &mut Config, dto: &SettingsDto) -> Result<(), String> {
    config.general.poll_interval_secs = dto.poll_interval_secs as u64;
    config.general.launch_at_login = dto.launch_at_login;
    config.monitor.enabled = dto.monitor_enabled;
    config.monitor.watch_processes = dto.monitor_watch_processes;
    config.monitor.watch_persistence = dto.monitor_watch_persistence;
    config.monitor.watch_lockfiles = dto.monitor_watch_lockfiles;
    config.guard.mode = parse_guard_mode(&dto.guard_mode)?;
    config.guard.timeout_secs = dto.guard_timeout_secs as u64;
    config.guard.sandbox = dto.guard_sandbox;
    config.posture.strict = dto.posture_strict;
    config.posture.require_lockfile = dto.posture_require_lockfile;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn read_settings() -> Result<SettingsDto, String> {
    let working_dir = std::env::current_dir().map_err(|e| e.to_string())?;
    let config = config::load_config(&working_dir).map_err(|e| e.to_string())?;
    Ok(config_to_dto(&config))
}

#[tauri::command]
#[specta::specta]
pub fn write_settings(settings: SettingsDto) -> Result<SettingsDto, String> {
    let path = config::config_file_path().map_err(|e| e.to_string())?;
    write_settings_to(&path, &settings)
}

pub(crate) fn write_settings_to(
    path: &Path,
    settings: &SettingsDto,
) -> Result<SettingsDto, String> {
    let mut config = if path.exists() {
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        toml::from_str(&content).unwrap_or_default()
    } else {
        Config::default()
    };
    apply_dto(&mut config, settings)?;
    config::save_config(&config, path).map_err(|e| e.to_string())?;
    Ok(config_to_dto(&config))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn sample_dto() -> SettingsDto {
        SettingsDto {
            poll_interval_secs: 600,
            launch_at_login: true,
            monitor_enabled: true,
            monitor_watch_processes: false,
            monitor_watch_persistence: true,
            monitor_watch_lockfiles: true,
            guard_mode: "audit".to_string(),
            guard_timeout_secs: 60,
            guard_sandbox: true,
            posture_strict: true,
            posture_require_lockfile: true,
        }
    }

    #[test]
    fn round_trips_settings_through_disk() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        let dto = sample_dto();
        let written = write_settings_to(&path, &dto).expect("write");
        assert_eq!(written.guard_mode, "audit");
        assert_eq!(written.poll_interval_secs, 600);

        let content = std::fs::read_to_string(&path).expect("read");
        assert!(content.contains("mode = \"audit\""));
        assert!(content.contains("poll_interval_secs = 600"));
    }

    #[test]
    fn rejects_unknown_guard_mode() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        let mut dto = sample_dto();
        dto.guard_mode = "bogus".to_string();
        let result = write_settings_to(&path, &dto);
        assert!(result.is_err());
        assert!(!path.exists(), "no file written when validation fails");
    }

    #[test]
    fn atomic_write_does_not_leave_tmp_behind() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        write_settings_to(&path, &sample_dto()).expect("write");
        let tmp = path.with_extension("toml.tmp");
        assert!(!tmp.exists(), "temp file should be renamed away");
        assert!(path.exists());
    }

    #[test]
    fn parse_guard_mode_accepts_all_variants() {
        assert_eq!(parse_guard_mode("strict").unwrap(), GuardMode::Strict);
        assert_eq!(parse_guard_mode("audit").unwrap(), GuardMode::Audit);
        assert_eq!(parse_guard_mode("off").unwrap(), GuardMode::Off);
        assert!(parse_guard_mode("nope").is_err());
    }
}
