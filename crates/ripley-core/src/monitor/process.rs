use std::path::PathBuf;

use serde::Serialize;

use crate::config::MonitorConfig;
use crate::forensic::network::{
    C2Database, C2Indicator, NetworkConnection, check_c2_connections, collect_active_connections,
};
use crate::types::Severity;

#[derive(Debug, Clone, Serialize)]
pub struct ProcessAlert {
    pub connection: NetworkConnection,
    pub reason: AlertReason,
    pub severity: Severity,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize)]
pub enum AlertReason {
    C2Connection { indicator: String },
    SuspiciousOutbound,
    PersistenceWrite { path: PathBuf },
    LockfileEdit { path: PathBuf },
    McpConfigChange { path: PathBuf },
    UnknownBinary { path: PathBuf },
}

impl std::fmt::Display for AlertReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::C2Connection { indicator } => write!(f, "C2 connection to {indicator}"),
            Self::SuspiciousOutbound => write!(f, "Suspicious outbound connection"),
            Self::PersistenceWrite { path } => write!(f, "Persistence write to {}", path.display()),
            Self::LockfileEdit { path } => write!(f, "Lockfile edit: {}", path.display()),
            Self::McpConfigChange { path } => write!(f, "MCP config change: {}", path.display()),
            Self::UnknownBinary { path } => write!(f, "Unknown binary: {}", path.display()),
        }
    }
}

pub fn scan_processes(
    config: &MonitorConfig,
    c2_db: &C2Database,
) -> Result<Vec<ProcessAlert>, std::io::Error> {
    let connections = collect_active_connections()?;
    let merged = merge_c2_database(c2_db, config);
    Ok(evaluate_connections(&connections, config, &merged))
}

pub fn evaluate_connections(
    connections: &[NetworkConnection],
    config: &MonitorConfig,
    c2_db: &C2Database,
) -> Vec<ProcessAlert> {
    if !config.watch_processes {
        return Vec::new();
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let dev_connections: Vec<&NetworkConnection> = connections
        .iter()
        .filter(|c| is_developer_process(&c.process))
        .collect();

    let c2_findings = check_c2_connections(
        &dev_connections.iter().copied().cloned().collect::<Vec<_>>(),
        c2_db,
    );

    let mut alerts: Vec<ProcessAlert> = c2_findings
        .into_iter()
        .map(|f| ProcessAlert {
            severity: f.severity,
            reason: AlertReason::C2Connection {
                indicator: f.matched_indicator,
            },
            connection: f.connection,
            timestamp: now,
        })
        .collect();

    for conn in &dev_connections {
        let already_flagged = alerts.iter().any(|a| {
            a.connection.pid == conn.pid
                && a.connection.remote_addr == conn.remote_addr
                && a.connection.remote_port == conn.remote_port
        });
        if already_flagged {
            continue;
        }

        if conn.state == "ESTABLISHED" && is_suspicious_port(conn.remote_port) {
            alerts.push(ProcessAlert {
                connection: (*conn).clone(),
                reason: AlertReason::SuspiciousOutbound,
                severity: Severity::Medium,
                timestamp: now,
            });
        }
    }

    alerts
}

pub fn is_developer_process(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "node"
            | "python"
            | "python3"
            | "ruby"
            | "go"
            | "bun"
            | "deno"
            | "pip"
            | "pip3"
            | "npm"
            | "yarn"
            | "pnpm"
            | "cargo"
            | "gem"
            | "bundler"
            | "poetry"
            | "pipx"
    )
}

fn is_suspicious_port(port: u16) -> bool {
    matches!(
        port,
        4444 | 5555 | 6666 | 7777 | 8888 | 9999 | 1337 | 31337 | 12345 | 54321
    )
}

pub fn merge_c2_database(compiled: &C2Database, config: &MonitorConfig) -> C2Database {
    let mut domains = compiled.domains.clone();
    for domain in &config.c2_domains {
        domains.push(C2Indicator {
            value: domain.clone(),
            description: "User-configured C2 domain".to_string(),
            severity: Severity::Critical,
        });
    }

    let mut ip_ranges = compiled.ip_ranges.clone();
    for ip in &config.c2_ips {
        ip_ranges.push(C2Indicator {
            value: ip.clone(),
            description: "User-configured C2 IP".to_string(),
            severity: Severity::Critical,
        });
    }

    C2Database { domains, ip_ranges }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_connection(
        process: &str,
        remote_addr: &str,
        remote_port: u16,
        state: &str,
    ) -> NetworkConnection {
        NetworkConnection {
            process: process.to_string(),
            pid: 1234,
            protocol: "TCP".to_string(),
            remote_addr: remote_addr.to_string(),
            remote_port,
            state: state.to_string(),
        }
    }

    fn default_config() -> MonitorConfig {
        MonitorConfig::default()
    }

    fn empty_c2_db() -> C2Database {
        C2Database {
            domains: Vec::new(),
            ip_ranges: Vec::new(),
        }
    }

    #[test]
    fn test_is_developer_process_true() {
        assert!(is_developer_process("node"));
        assert!(is_developer_process("python3"));
        assert!(is_developer_process("ruby"));
        assert!(is_developer_process("go"));
        assert!(is_developer_process("bun"));
        assert!(is_developer_process("deno"));
        assert!(is_developer_process("npm"));
        assert!(is_developer_process("yarn"));
        assert!(is_developer_process("pnpm"));
        assert!(is_developer_process("cargo"));
        assert!(is_developer_process("pip"));
    }

    #[test]
    fn test_is_developer_process_false() {
        assert!(!is_developer_process("systemd"));
        assert!(!is_developer_process("sshd"));
        assert!(!is_developer_process("Finder"));
        assert!(!is_developer_process("chrome"));
        assert!(!is_developer_process("launchd"));
    }

    #[test]
    fn test_is_developer_process_case_insensitive() {
        assert!(is_developer_process("Node"));
        assert!(is_developer_process("PYTHON3"));
        assert!(is_developer_process("Ruby"));
    }

    #[test]
    fn test_evaluate_connections_empty() {
        let config = default_config();
        let db = empty_c2_db();
        let alerts = evaluate_connections(&[], &config, &db);
        assert!(alerts.is_empty());
    }

    #[test]
    fn test_evaluate_connections_filters_non_dev() {
        let config = default_config();
        let db = empty_c2_db();
        let connections = vec![
            make_connection("chrome", "142.250.80.46", 443, "ESTABLISHED"),
            make_connection("sshd", "10.0.0.1", 22, "ESTABLISHED"),
        ];
        let alerts = evaluate_connections(&connections, &config, &db);
        assert!(alerts.is_empty());
    }

    #[test]
    fn test_evaluate_connections_c2_match() {
        let config = default_config();
        let db = C2Database {
            domains: vec![C2Indicator {
                value: "evil.example.com".to_string(),
                description: "Known C2".to_string(),
                severity: Severity::Critical,
            }],
            ip_ranges: Vec::new(),
        };
        let connections = vec![make_connection(
            "node",
            "evil.example.com",
            443,
            "ESTABLISHED",
        )];
        let alerts = evaluate_connections(&connections, &config, &db);
        assert_eq!(alerts.len(), 1);
        assert!(
            matches!(&alerts[0].reason, AlertReason::C2Connection { indicator } if indicator == "evil.example.com")
        );
        assert_eq!(alerts[0].severity, Severity::Critical);
    }

    #[test]
    fn test_evaluate_connections_suspicious_port() {
        let config = default_config();
        let db = empty_c2_db();
        let connections = vec![make_connection(
            "node",
            "192.168.1.100",
            4444,
            "ESTABLISHED",
        )];
        let alerts = evaluate_connections(&connections, &config, &db);
        assert_eq!(alerts.len(), 1);
        assert!(matches!(&alerts[0].reason, AlertReason::SuspiciousOutbound));
        assert_eq!(alerts[0].severity, Severity::Medium);
    }

    #[test]
    fn test_evaluate_connections_watch_processes_disabled() {
        let mut config = default_config();
        config.watch_processes = false;
        let db = C2Database {
            domains: vec![C2Indicator {
                value: "evil.com".to_string(),
                description: "C2".to_string(),
                severity: Severity::Critical,
            }],
            ip_ranges: Vec::new(),
        };
        let connections = vec![make_connection("node", "evil.com", 443, "ESTABLISHED")];
        let alerts = evaluate_connections(&connections, &config, &db);
        assert!(alerts.is_empty());
    }

    #[test]
    fn test_evaluate_connections_no_double_alert() {
        let config = default_config();
        let db = C2Database {
            domains: vec![C2Indicator {
                value: "evil.com".to_string(),
                description: "C2".to_string(),
                severity: Severity::Critical,
            }],
            ip_ranges: Vec::new(),
        };
        let connections = vec![make_connection("node", "evil.com", 4444, "ESTABLISHED")];
        let alerts = evaluate_connections(&connections, &config, &db);
        assert_eq!(alerts.len(), 1);
        assert!(matches!(
            &alerts[0].reason,
            AlertReason::C2Connection { .. }
        ));
    }

    #[test]
    fn test_merge_c2_database_adds_config_entries() {
        let compiled = C2Database {
            domains: vec![C2Indicator {
                value: "compiled.com".to_string(),
                description: "Built-in".to_string(),
                severity: Severity::High,
            }],
            ip_ranges: Vec::new(),
        };
        let mut config = default_config();
        config.c2_domains = vec!["user-added.com".to_string()];
        config.c2_ips = vec!["10.0.0.0".to_string()];

        let merged = merge_c2_database(&compiled, &config);
        assert_eq!(merged.domains.len(), 2);
        assert_eq!(merged.domains[0].value, "compiled.com");
        assert_eq!(merged.domains[1].value, "user-added.com");
        assert_eq!(merged.ip_ranges.len(), 1);
        assert_eq!(merged.ip_ranges[0].value, "10.0.0.0");
    }

    #[test]
    fn test_merge_c2_database_empty_config() {
        let compiled = C2Database {
            domains: vec![C2Indicator {
                value: "compiled.com".to_string(),
                description: "Built-in".to_string(),
                severity: Severity::High,
            }],
            ip_ranges: vec![C2Indicator {
                value: "192.168.".to_string(),
                description: "Built-in IP".to_string(),
                severity: Severity::Medium,
            }],
        };
        let config = default_config();
        let merged = merge_c2_database(&compiled, &config);
        assert_eq!(merged.domains.len(), 1);
        assert_eq!(merged.ip_ranges.len(), 1);
    }

    #[test]
    fn test_alert_reason_display() {
        let reason = AlertReason::C2Connection {
            indicator: "evil.com".to_string(),
        };
        assert_eq!(format!("{reason}"), "C2 connection to evil.com");

        let reason = AlertReason::SuspiciousOutbound;
        assert_eq!(format!("{reason}"), "Suspicious outbound connection");
    }
}
