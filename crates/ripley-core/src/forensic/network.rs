use serde::{Deserialize, Serialize};

use crate::types::Severity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConnection {
    pub process: String,
    pub pid: u32,
    pub protocol: String,
    pub remote_addr: String,
    pub remote_port: u16,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct C2Finding {
    pub connection: NetworkConnection,
    pub matched_indicator: String,
    pub severity: Severity,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct C2Database {
    pub domains: Vec<C2Indicator>,
    pub ip_ranges: Vec<C2Indicator>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct C2Indicator {
    pub value: String,
    pub description: String,
    pub severity: Severity,
}

impl C2Database {
    pub fn load_compiled() -> Self {
        toml::from_str(include_str!("../../../../c2/known_c2.toml"))
            .expect("compiled C2 database must parse")
    }
}

pub fn collect_active_connections() -> Result<Vec<NetworkConnection>, std::io::Error> {
    #[cfg(target_os = "macos")]
    {
        collect_lsof()
    }

    #[cfg(target_os = "linux")]
    {
        collect_ss()
    }

    #[cfg(target_os = "windows")]
    {
        collect_netstat()
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Ok(Vec::new())
    }
}

#[cfg(target_os = "macos")]
fn collect_lsof() -> Result<Vec<NetworkConnection>, std::io::Error> {
    let output = std::process::Command::new("lsof")
        .args(["-i", "-n", "-P"])
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_lsof_output(&stdout))
}

#[cfg(target_os = "linux")]
fn collect_ss() -> Result<Vec<NetworkConnection>, std::io::Error> {
    let output = std::process::Command::new("ss").args(["-tunap"]).output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_ss_output(&stdout))
}

#[cfg(target_os = "windows")]
fn collect_netstat() -> Result<Vec<NetworkConnection>, std::io::Error> {
    let output = std::process::Command::new("netstat")
        .args(["-ano"])
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_netstat_output(&stdout))
}

pub fn parse_lsof_output(output: &str) -> Vec<NetworkConnection> {
    let mut connections = Vec::new();

    for line in output.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 9 {
            continue;
        }

        let process = parts[0].to_string();
        let pid: u32 = match parts[1].parse() {
            Ok(p) => p,
            Err(_) => continue,
        };

        let name_field = parts[parts.len() - 1];
        if !name_field.contains("->") {
            continue;
        }

        let arrow_parts: Vec<&str> = name_field.split("->").collect();
        if arrow_parts.len() < 2 {
            continue;
        }

        let remote = arrow_parts[1];
        let (remote_addr, remote_port) = match remote.rfind(':') {
            Some(pos) => (&remote[..pos], &remote[pos + 1..]),
            None => continue,
        };

        let port: u16 = match remote_port.parse() {
            Ok(p) => p,
            Err(_) => continue,
        };

        let protocol = if parts.len() > 7 {
            parts[7].to_lowercase()
        } else {
            "tcp".to_string()
        };

        connections.push(NetworkConnection {
            process,
            pid,
            protocol,
            remote_addr: remote_addr.to_string(),
            remote_port: port,
            state: "ESTABLISHED".to_string(),
        });
    }

    connections
}

pub fn parse_ss_output(output: &str) -> Vec<NetworkConnection> {
    let mut connections = Vec::new();

    for line in output.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 6 {
            continue;
        }

        let state = parts[0].to_string();
        let peer = parts[4];

        let (remote_addr, remote_port) = match peer.rfind(':') {
            Some(pos) => (&peer[..pos], &peer[pos + 1..]),
            None => continue,
        };

        let port: u16 = match remote_port.parse() {
            Ok(p) => p,
            Err(_) => continue,
        };

        let process_info = if parts.len() > 5 { parts[5] } else { "" };
        let process = extract_process_name(process_info);
        let pid = extract_pid(process_info);

        connections.push(NetworkConnection {
            process,
            pid,
            protocol: "tcp".to_string(),
            remote_addr: remote_addr.to_string(),
            remote_port: port,
            state,
        });
    }

    connections
}

fn extract_process_name(info: &str) -> String {
    if let Some(start) = info.find('"') {
        if let Some(end) = info[start + 1..].find('"') {
            return info[start + 1..start + 1 + end].to_string();
        }
    }
    "unknown".to_string()
}

fn extract_pid(info: &str) -> u32 {
    if let Some(start) = info.find("pid=") {
        let rest = &info[start + 4..];
        let end = rest.find(',').unwrap_or(rest.len());
        rest[..end].parse().unwrap_or(0)
    } else {
        0
    }
}

pub fn check_c2_connections(
    connections: &[NetworkConnection],
    c2_db: &C2Database,
) -> Vec<C2Finding> {
    let mut findings = Vec::new();

    for conn in connections {
        for domain in &c2_db.domains {
            if conn.remote_addr.contains(&domain.value) {
                findings.push(C2Finding {
                    connection: conn.clone(),
                    matched_indicator: domain.value.clone(),
                    severity: domain.severity,
                    description: domain.description.clone(),
                });
            }
        }

        for ip_range in &c2_db.ip_ranges {
            if conn.remote_addr.starts_with(&ip_range.value) {
                findings.push(C2Finding {
                    connection: conn.clone(),
                    matched_indicator: ip_range.value.clone(),
                    severity: ip_range.severity,
                    description: ip_range.description.clone(),
                });
            }
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lsof_output() {
        let output = "COMMAND     PID USER   FD   TYPE  DEVICE SIZE/OFF NODE NAME\n\
                      node      1234 jack   22u  IPv4 0x1234      0t0  TCP 127.0.0.1:3000->93.184.216.34:443\n\
                      chrome    5678 jack   33u  IPv4 0x5678      0t0  TCP 127.0.0.1:5555->142.250.80.46:443\n";

        let connections = parse_lsof_output(output);
        assert_eq!(connections.len(), 2);
        assert_eq!(connections[0].process, "node");
        assert_eq!(connections[0].pid, 1234);
        assert_eq!(connections[0].remote_addr, "93.184.216.34");
        assert_eq!(connections[0].remote_port, 443);
    }

    #[test]
    fn test_parse_lsof_skips_listening() {
        let output = "COMMAND     PID USER   FD   TYPE  DEVICE SIZE/OFF NODE NAME\n\
                      node      1234 jack   22u  IPv4 0x1234      0t0  TCP *:3000 (LISTEN)\n";

        let connections = parse_lsof_output(output);
        assert!(connections.is_empty());
    }

    #[test]
    fn test_c2_match_found() {
        let connections = vec![NetworkConnection {
            process: "node".to_string(),
            pid: 1234,
            protocol: "tcp".to_string(),
            remote_addr: "evil.c2server.example.com".to_string(),
            remote_port: 443,
            state: "ESTABLISHED".to_string(),
        }];

        let c2_db = C2Database {
            domains: vec![C2Indicator {
                value: "evil.c2server.example.com".to_string(),
                description: "Known C2 domain".to_string(),
                severity: Severity::Critical,
            }],
            ip_ranges: vec![],
        };

        let findings = check_c2_connections(&connections, &c2_db);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Critical);
    }

    #[test]
    fn test_c2_no_match() {
        let connections = vec![NetworkConnection {
            process: "chrome".to_string(),
            pid: 5678,
            protocol: "tcp".to_string(),
            remote_addr: "142.250.80.46".to_string(),
            remote_port: 443,
            state: "ESTABLISHED".to_string(),
        }];

        let c2_db = C2Database {
            domains: vec![C2Indicator {
                value: "evil.example.com".to_string(),
                description: "Bad".to_string(),
                severity: Severity::Critical,
            }],
            ip_ranges: vec![C2Indicator {
                value: "10.99.".to_string(),
                description: "Suspicious range".to_string(),
                severity: Severity::High,
            }],
        };

        let findings = check_c2_connections(&connections, &c2_db);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_c2_ip_range_match() {
        let connections = vec![NetworkConnection {
            process: "python".to_string(),
            pid: 9999,
            protocol: "tcp".to_string(),
            remote_addr: "10.99.1.5".to_string(),
            remote_port: 8080,
            state: "ESTABLISHED".to_string(),
        }];

        let c2_db = C2Database {
            domains: vec![],
            ip_ranges: vec![C2Indicator {
                value: "10.99.".to_string(),
                description: "Known C2 IP range".to_string(),
                severity: Severity::High,
            }],
        };

        let findings = check_c2_connections(&connections, &c2_db);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::High);
    }

    #[test]
    fn test_load_c2_database() {
        let db = C2Database::load_compiled();
        assert!(!db.domains.is_empty() || !db.ip_ranges.is_empty());
    }

    #[test]
    fn test_parse_ss_output() {
        let output = "State  Recv-Q Send-Q Local Address:Port   Peer Address:Port Process\n\
                      ESTAB  0      0      192.168.1.5:42000     93.184.216.34:443  users:((\"node\",pid=1234,fd=22))\n";

        let connections = parse_ss_output(output);
        assert_eq!(connections.len(), 1);
        assert_eq!(connections[0].remote_addr, "93.184.216.34");
        assert_eq!(connections[0].remote_port, 443);
        assert_eq!(connections[0].process, "node");
        assert_eq!(connections[0].pid, 1234);
    }
}
