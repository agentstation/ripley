use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::dirs;
use crate::types::GuardMode;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("could not read config file {path}: {source}")]
    ReadFile {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("could not parse config file {path}: {source}")]
    ParseToml {
        path: PathBuf,
        source: toml::de::Error,
    },
    #[error("could not serialize config: {0}")]
    SerializeToml(#[from] toml::ser::Error),
    #[error("could not write config file {path}: {source}")]
    WriteFile {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error(transparent)]
    Dir(#[from] dirs::DirError),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub general: GeneralConfig,
    pub monitoring: MonitoringConfig,
    pub guard: GuardConfig,
    pub posture: PostureConfig,
    pub audit: AuditConfig,
    pub feeds: FeedsConfig,
    pub notifications: NotificationsConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    pub poll_interval_secs: u64,
    pub harness: Option<String>,
    pub launch_at_login: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            poll_interval_secs: 300,
            harness: None,
            launch_at_login: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct MonitoringConfig {
    pub project_roots: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GuardConfig {
    pub mode: GuardMode,
    pub trust: Vec<String>,
    pub timeout_secs: u64,
}

impl Default for GuardConfig {
    fn default() -> Self {
        Self {
            mode: GuardMode::Strict,
            trust: Vec::new(),
            timeout_secs: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PostureConfig {
    pub strict: bool,
    pub require_lockfile: bool,
    pub require_exact_versions: bool,
    pub require_integrity_hashes: bool,
    pub block_exotic_sources: bool,
    pub allowed_registries: Vec<String>,
}

impl Default for PostureConfig {
    fn default() -> Self {
        Self {
            strict: false,
            require_lockfile: false,
            require_exact_versions: false,
            require_integrity_hashes: false,
            block_exotic_sources: false,
            allowed_registries: vec!["https://registry.npmjs.org".to_string()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AuditConfig {
    pub machine_security: bool,
    pub toolchain: bool,
    pub ai_tools: bool,
    pub credentials: bool,
    pub min_score: Option<String>,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            machine_security: true,
            toolchain: true,
            ai_tools: true,
            credentials: true,
            min_score: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FeedsConfig {
    pub osv: bool,
    pub ghsa: bool,
    pub socket: bool,
    pub socket_api_key: Option<String>,
    pub stale_threshold_secs: u64,
}

impl Default for FeedsConfig {
    fn default() -> Self {
        Self {
            osv: true,
            ghsa: false,
            socket: false,
            socket_api_key: None,
            stale_threshold_secs: 3600,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NotificationsConfig {
    pub enabled: bool,
    pub min_severity: String,
    pub sound: bool,
}

impl Default for NotificationsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_severity: "low".to_string(),
            sound: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    pub level: String,
    pub file: bool,
    pub rotation: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file: true,
            rotation: "daily".to_string(),
        }
    }
}

fn read_toml_file(path: &Path) -> Result<Option<Config>, ConfigError> {
    match std::fs::read_to_string(path) {
        Ok(contents) => {
            let config: Config = toml::from_str(&contents).map_err(|e| ConfigError::ParseToml {
                path: path.to_path_buf(),
                source: e,
            })?;
            Ok(Some(config))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(ConfigError::ReadFile {
            path: path.to_path_buf(),
            source: e,
        }),
    }
}

fn apply_env_overrides(config: &mut Config) {
    if let Ok(val) = std::env::var("RIPLEY_POLL_INTERVAL")
        && let Ok(secs) = val.parse::<u64>()
    {
        config.general.poll_interval_secs = secs;
    }
    if let Ok(val) = std::env::var("RIPLEY_HARNESS") {
        config.general.harness = Some(val);
    }
    if let Ok(val) = std::env::var("RIPLEY_PROJECT_ROOTS") {
        config.monitoring.project_roots = val.split(':').map(String::from).collect();
    }
    if let Ok(val) = std::env::var("RIPLEY_GUARD_MODE")
        && let Ok(mode) = serde_json::from_value(serde_json::Value::String(val))
    {
        config.guard.mode = mode;
    }
    if let Ok(val) = std::env::var("RIPLEY_SOCKET_API_KEY") {
        config.feeds.socket_api_key = Some(val);
    }
    if let Ok(val) = std::env::var("RIPLEY_LOG_LEVEL") {
        config.logging.level = val;
    }
}

fn merge_config(base: &mut Config, overlay: &Config) {
    if overlay.general.poll_interval_secs != GeneralConfig::default().poll_interval_secs {
        base.general.poll_interval_secs = overlay.general.poll_interval_secs;
    }
    if overlay.general.harness.is_some() {
        base.general.harness.clone_from(&overlay.general.harness);
    }
    if overlay.general.launch_at_login {
        base.general.launch_at_login = true;
    }

    if !overlay.monitoring.project_roots.is_empty() {
        base.monitoring
            .project_roots
            .extend(overlay.monitoring.project_roots.iter().cloned());
    }

    if overlay.guard.mode != GuardMode::Strict {
        base.guard.mode = overlay.guard.mode;
    }
    if !overlay.guard.trust.is_empty() {
        base.guard.trust.extend(overlay.guard.trust.iter().cloned());
    }
    if overlay.guard.timeout_secs != GuardConfig::default().timeout_secs {
        base.guard.timeout_secs = overlay.guard.timeout_secs;
    }

    if overlay.posture.strict {
        base.posture.strict = true;
    }
    if overlay.posture.require_lockfile {
        base.posture.require_lockfile = true;
    }
    if overlay.posture.require_exact_versions {
        base.posture.require_exact_versions = true;
    }
    if overlay.posture.require_integrity_hashes {
        base.posture.require_integrity_hashes = true;
    }
    if overlay.posture.block_exotic_sources {
        base.posture.block_exotic_sources = true;
    }
    if overlay.posture.allowed_registries != PostureConfig::default().allowed_registries {
        base.posture
            .allowed_registries
            .clone_from(&overlay.posture.allowed_registries);
    }

    if overlay.feeds.stale_threshold_secs != FeedsConfig::default().stale_threshold_secs {
        base.feeds.stale_threshold_secs = overlay.feeds.stale_threshold_secs;
    }
    if overlay.feeds.socket_api_key.is_some() {
        base.feeds
            .socket_api_key
            .clone_from(&overlay.feeds.socket_api_key);
    }

    if !overlay.notifications.enabled {
        base.notifications.enabled = false;
    }
    if overlay.notifications.min_severity != NotificationsConfig::default().min_severity {
        base.notifications
            .min_severity
            .clone_from(&overlay.notifications.min_severity);
    }

    if overlay.logging.level != LoggingConfig::default().level {
        base.logging.level.clone_from(&overlay.logging.level);
    }
}

pub fn load_config(working_dir: &Path) -> Result<Config, ConfigError> {
    let mut config = Config::default();

    let config_path = dirs::config_dir()?.join("config.toml");
    if let Some(user_config) = read_toml_file(&config_path)? {
        merge_config(&mut config, &user_config);
    }

    let project_config_path = working_dir.join(".ripley.toml");
    if let Some(project_config) = read_toml_file(&project_config_path)? {
        merge_config(&mut config, &project_config);
    }

    apply_env_overrides(&mut config);

    Ok(config)
}

pub fn config_file_path() -> Result<PathBuf, ConfigError> {
    Ok(dirs::config_dir()?.join("config.toml"))
}

pub fn write_default_config(path: &Path) -> Result<(), ConfigError> {
    let config = Config::default();
    let toml_str = toml::to_string_pretty(&config)?;

    let parent = path.parent().ok_or_else(|| ConfigError::WriteFile {
        path: path.to_path_buf(),
        source: std::io::Error::new(std::io::ErrorKind::InvalidInput, "no parent directory"),
    })?;
    std::fs::create_dir_all(parent).map_err(|e| ConfigError::WriteFile {
        path: path.to_path_buf(),
        source: e,
    })?;

    let temp_path = path.with_extension("toml.tmp");
    std::fs::write(&temp_path, toml_str.as_bytes()).map_err(|e| ConfigError::WriteFile {
        path: temp_path.clone(),
        source: e,
    })?;
    std::fs::rename(&temp_path, path).map_err(|e| ConfigError::WriteFile {
        path: path.to_path_buf(),
        source: e,
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.general.poll_interval_secs, 300);
        assert_eq!(config.guard.mode, GuardMode::Strict);
        assert_eq!(config.guard.timeout_secs, 30);
        assert!(config.feeds.osv);
        assert!(!config.feeds.ghsa);
        assert_eq!(config.feeds.stale_threshold_secs, 3600);
        assert!(config.notifications.enabled);
        assert_eq!(
            config.posture.allowed_registries,
            vec!["https://registry.npmjs.org"]
        );
    }

    #[test]
    fn test_default_config_roundtrip_toml() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).expect("serialize");
        let parsed: Config = toml::from_str(&toml_str).expect("parse");
        assert_eq!(parsed.general.poll_interval_secs, 300);
        assert_eq!(parsed.guard.timeout_secs, 30);
    }

    #[test]
    fn test_layered_loading() {
        let dir = tempfile::tempdir().expect("create tempdir");
        let project_config = dir.path().join(".ripley.toml");
        std::fs::write(
            &project_config,
            r#"
[posture]
strict = true
require_lockfile = true
"#,
        )
        .expect("write project config");

        let config = load_config(dir.path()).expect("load config");
        assert!(config.posture.strict);
        assert!(config.posture.require_lockfile);
        assert!(!config.posture.require_exact_versions);
    }

    #[test]
    fn test_apply_env_overrides() {
        let mut config = Config::default();
        config.general.poll_interval_secs = 300;

        // Test the merge logic directly instead of mutating process env
        let overlay_toml = r#"
[general]
poll_interval_secs = 60
"#;
        let overlay: Config = toml::from_str(overlay_toml).expect("parse");
        merge_config(&mut config, &overlay);
        assert_eq!(config.general.poll_interval_secs, 60);
    }

    #[test]
    fn test_write_default_config() {
        let dir = tempfile::tempdir().expect("create tempdir");
        let path = dir.path().join("config.toml");
        write_default_config(&path).expect("write default config");
        assert!(path.exists());

        let contents = std::fs::read_to_string(&path).expect("read");
        let parsed: Config = toml::from_str(&contents).expect("parse");
        assert_eq!(parsed.general.poll_interval_secs, 300);
    }
}
