use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::dirs;
use crate::types::{GuardMode, LogLevel, Severity};

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

// --- Resolved config (all fields have values) ---

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
    pub min_severity: Severity,
    pub sound: bool,
}

impl Default for NotificationsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_severity: Severity::Low,
            sound: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    pub level: LogLevel,
    pub file: bool,
    pub rotation: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            file: true,
            rotation: "daily".to_string(),
        }
    }
}

// --- Overlay config (all fields optional, for layered merging) ---

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
struct ConfigOverlay {
    general: GeneralOverlay,
    monitoring: MonitoringOverlay,
    guard: GuardOverlay,
    posture: PostureOverlay,
    feeds: FeedsOverlay,
    notifications: NotificationsOverlay,
    logging: LoggingOverlay,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
struct GeneralOverlay {
    poll_interval_secs: Option<u64>,
    harness: Option<String>,
    launch_at_login: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
struct MonitoringOverlay {
    project_roots: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
struct GuardOverlay {
    mode: Option<GuardMode>,
    trust: Option<Vec<String>>,
    timeout_secs: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
struct PostureOverlay {
    strict: Option<bool>,
    require_lockfile: Option<bool>,
    require_exact_versions: Option<bool>,
    require_integrity_hashes: Option<bool>,
    block_exotic_sources: Option<bool>,
    allowed_registries: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
struct FeedsOverlay {
    stale_threshold_secs: Option<u64>,
    socket_api_key: Option<String>,
    osv: Option<bool>,
    ghsa: Option<bool>,
    socket: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
struct NotificationsOverlay {
    enabled: Option<bool>,
    min_severity: Option<Severity>,
    sound: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
struct LoggingOverlay {
    level: Option<LogLevel>,
    file: Option<bool>,
    rotation: Option<String>,
}

fn apply_overlay(base: &mut Config, overlay: &ConfigOverlay) {
    if let Some(v) = overlay.general.poll_interval_secs {
        base.general.poll_interval_secs = v;
    }
    if let Some(ref v) = overlay.general.harness {
        base.general.harness = Some(v.clone());
    }
    if let Some(v) = overlay.general.launch_at_login {
        base.general.launch_at_login = v;
    }

    if let Some(ref roots) = overlay.monitoring.project_roots {
        base.monitoring.project_roots.extend(roots.iter().cloned());
    }

    if let Some(v) = overlay.guard.mode {
        base.guard.mode = v;
    }
    if let Some(ref v) = overlay.guard.trust {
        base.guard.trust.extend(v.iter().cloned());
    }
    if let Some(v) = overlay.guard.timeout_secs {
        base.guard.timeout_secs = v;
    }

    if let Some(v) = overlay.posture.strict {
        base.posture.strict = v;
    }
    if let Some(v) = overlay.posture.require_lockfile {
        base.posture.require_lockfile = v;
    }
    if let Some(v) = overlay.posture.require_exact_versions {
        base.posture.require_exact_versions = v;
    }
    if let Some(v) = overlay.posture.require_integrity_hashes {
        base.posture.require_integrity_hashes = v;
    }
    if let Some(v) = overlay.posture.block_exotic_sources {
        base.posture.block_exotic_sources = v;
    }
    if let Some(ref v) = overlay.posture.allowed_registries {
        base.posture.allowed_registries.clone_from(v);
    }

    if let Some(v) = overlay.feeds.osv {
        base.feeds.osv = v;
    }
    if let Some(v) = overlay.feeds.ghsa {
        base.feeds.ghsa = v;
    }
    if let Some(v) = overlay.feeds.socket {
        base.feeds.socket = v;
    }
    if let Some(ref v) = overlay.feeds.socket_api_key {
        base.feeds.socket_api_key = Some(v.clone());
    }
    if let Some(v) = overlay.feeds.stale_threshold_secs {
        base.feeds.stale_threshold_secs = v;
    }

    if let Some(v) = overlay.notifications.enabled {
        base.notifications.enabled = v;
    }
    if let Some(v) = overlay.notifications.min_severity {
        base.notifications.min_severity = v;
    }
    if let Some(v) = overlay.notifications.sound {
        base.notifications.sound = v;
    }

    if let Some(v) = overlay.logging.level {
        base.logging.level = v;
    }
    if let Some(v) = overlay.logging.file {
        base.logging.file = v;
    }
    if let Some(ref v) = overlay.logging.rotation {
        base.logging.rotation.clone_from(v);
    }
}

fn read_overlay(path: &Path) -> Result<Option<ConfigOverlay>, ConfigError> {
    match std::fs::read_to_string(path) {
        Ok(contents) => {
            let overlay: ConfigOverlay =
                toml::from_str(&contents).map_err(|e| ConfigError::ParseToml {
                    path: path.to_path_buf(),
                    source: e,
                })?;
            Ok(Some(overlay))
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
    if let Ok(val) = std::env::var("RIPLEY_LOG_LEVEL")
        && let Ok(level) = serde_json::from_value(serde_json::Value::String(val))
    {
        config.logging.level = level;
    }
}

pub fn load_config(working_dir: &Path) -> Result<Config, ConfigError> {
    let mut config = Config::default();

    let config_path = dirs::config_dir()?.join("config.toml");
    if let Some(overlay) = read_overlay(&config_path)? {
        apply_overlay(&mut config, &overlay);
    }

    let project_config_path = working_dir.join(".ripley.toml");
    if let Some(overlay) = read_overlay(&project_config_path)? {
        apply_overlay(&mut config, &overlay);
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
    fn test_overlay_only_overrides_specified_fields() {
        let mut config = Config::default();
        let overlay_toml = r#"
[general]
poll_interval_secs = 60
"#;
        let overlay: ConfigOverlay = toml::from_str(overlay_toml).expect("parse");
        apply_overlay(&mut config, &overlay);
        assert_eq!(config.general.poll_interval_secs, 60);
        assert!(!config.general.launch_at_login);
        assert_eq!(config.guard.timeout_secs, 30);
    }

    #[test]
    fn test_overlay_can_set_value_to_default() {
        let mut config = Config::default();
        config.general.poll_interval_secs = 60;

        let overlay_toml = r#"
[general]
poll_interval_secs = 300
"#;
        let overlay: ConfigOverlay = toml::from_str(overlay_toml).expect("parse");
        apply_overlay(&mut config, &overlay);
        assert_eq!(config.general.poll_interval_secs, 300);
    }

    #[test]
    fn test_overlay_can_disable_strict() {
        let mut config = Config::default();
        config.posture.strict = true;

        let overlay_toml = r#"
[posture]
strict = false
"#;
        let overlay: ConfigOverlay = toml::from_str(overlay_toml).expect("parse");
        apply_overlay(&mut config, &overlay);
        assert!(!config.posture.strict);
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
