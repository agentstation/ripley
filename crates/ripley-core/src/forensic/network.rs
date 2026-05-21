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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct C2Database {
    pub domains: Vec<C2Indicator>,
    pub ip_ranges: Vec<C2Indicator>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

fn split_addr_port(s: &str) -> Option<(&str, &str)> {
    if let Some(bracket_end) = s.find("]:") {
        Some((&s[1..bracket_end], &s[bracket_end + 2..]))
    } else {
        s.rfind(':').map(|pos| (&s[..pos], &s[pos + 1..]))
    }
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

        let (name_field, state) = if let Some(paren_start) = name_field.rfind('(') {
            if name_field.ends_with(')') {
                let s = &name_field[paren_start + 1..name_field.len() - 1];
                (&name_field[..paren_start], s.to_string())
            } else {
                (name_field, "ESTABLISHED".to_string())
            }
        } else {
            (name_field, "ESTABLISHED".to_string())
        };

        if !name_field.contains("->") {
            continue;
        }

        let arrow_parts: Vec<&str> = name_field.split("->").collect();
        if arrow_parts.len() < 2 {
            continue;
        }

        let remote = arrow_parts[1];
        let (remote_addr, remote_port) = match split_addr_port(remote) {
            Some(pair) => pair,
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
            state,
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

        let (remote_addr, remote_port) = match split_addr_port(peer) {
            Some(pair) => pair,
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

pub fn parse_netstat_output(output: &str) -> Vec<NetworkConnection> {
    let mut connections = Vec::new();

    for line in output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 {
            continue;
        }

        let proto = parts[0].to_uppercase();
        if !proto.starts_with("TCP") && !proto.starts_with("UDP") {
            continue;
        }

        let foreign = parts[2];
        let state = if parts.len() > 3 {
            parts[3].to_string()
        } else {
            String::new()
        };

        let (remote_addr, remote_port) = match split_addr_port(foreign) {
            Some(pair) => pair,
            None => continue,
        };

        let port: u16 = match remote_port.parse() {
            Ok(p) => p,
            Err(_) => continue,
        };

        let pid_idx = if parts.len() > 4 { 4 } else { parts.len() - 1 };
        let pid: u32 = parts[pid_idx].parse().unwrap_or(0);

        connections.push(NetworkConnection {
            process: "unknown".to_string(),
            pid,
            protocol: proto.to_lowercase(),
            remote_addr: remote_addr.to_string(),
            remote_port: port,
            state,
        });
    }

    connections
}

fn extract_process_name(info: &str) -> String {
    if let Some(start) = info.find('"')
        && let Some(end) = info[start + 1..].find('"')
    {
        return info[start + 1..start + 1 + end].to_string();
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
            if conn.remote_addr == domain.value
                || conn.remote_addr.ends_with(&format!(".{}", domain.value))
            {
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
    fn test_parse_lsof_with_state() {
        let output = "COMMAND     PID USER   FD   TYPE  DEVICE SIZE/OFF NODE NAME\n\
                      node      1234 jack   22u  IPv4 0x1234      0t0  TCP 127.0.0.1:3000->93.184.216.34:443(CLOSE_WAIT)\n";

        let connections = parse_lsof_output(output);
        assert_eq!(connections.len(), 1);
        assert_eq!(connections[0].state, "CLOSE_WAIT");
    }

    #[test]
    fn test_parse_lsof_ipv6() {
        let output = "COMMAND     PID USER   FD   TYPE  DEVICE SIZE/OFF NODE NAME\n\
                      node      1234 jack   22u  IPv6 0x1234      0t0  TCP [::1]:3000->[::1]:443\n";

        let connections = parse_lsof_output(output);
        assert_eq!(connections.len(), 1);
        assert_eq!(connections[0].remote_addr, "::1");
        assert_eq!(connections[0].remote_port, 443);
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

    #[test]
    fn test_parse_ss_ipv6() {
        let output = "State  Recv-Q Send-Q Local Address:Port   Peer Address:Port Process\n\
                      ESTAB  0      0      [::1]:42000           [::1]:443  users:((\"node\",pid=1234,fd=22))\n";

        let connections = parse_ss_output(output);
        assert_eq!(connections.len(), 1);
        assert_eq!(connections[0].remote_addr, "::1");
        assert_eq!(connections[0].remote_port, 443);
    }

    #[test]
    fn test_parse_netstat_output() {
        let output = "  Proto  Local Address          Foreign Address        State           PID\n\
                      TCP    192.168.1.5:49742     93.184.216.34:443      ESTABLISHED     1234\n\
                      TCP    0.0.0.0:80            0.0.0.0:0              LISTENING       4\n";

        let connections = parse_netstat_output(output);
        assert_eq!(connections.len(), 2);
        assert_eq!(connections[0].remote_addr, "93.184.216.34");
        assert_eq!(connections[0].remote_port, 443);
        assert_eq!(connections[0].pid, 1234);
        assert_eq!(connections[0].state, "ESTABLISHED");
        assert_eq!(connections[0].protocol, "tcp");
    }

    #[test]
    fn test_parse_netstat_ipv6() {
        let output = "  TCP    [::1]:49742           [::1]:443              ESTABLISHED     1234\n";

        let connections = parse_netstat_output(output);
        assert_eq!(connections.len(), 1);
        assert_eq!(connections[0].remote_addr, "::1");
        assert_eq!(connections[0].remote_port, 443);
    }

    #[test]
    fn test_c2_exact_domain_match() {
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
    fn test_c2_subdomain_match() {
        let connections = vec![NetworkConnection {
            process: "node".to_string(),
            pid: 1234,
            protocol: "tcp".to_string(),
            remote_addr: "sub.evil.example.com".to_string(),
            remote_port: 443,
            state: "ESTABLISHED".to_string(),
        }];

        let c2_db = C2Database {
            domains: vec![C2Indicator {
                value: "evil.example.com".to_string(),
                description: "Known C2 domain".to_string(),
                severity: Severity::Critical,
            }],
            ip_ranges: vec![],
        };

        let findings = check_c2_connections(&connections, &c2_db);
        assert_eq!(findings.len(), 1);
    }

    #[test]
    fn test_c2_no_partial_match() {
        let connections = vec![NetworkConnection {
            process: "chrome".to_string(),
            pid: 5678,
            protocol: "tcp".to_string(),
            remote_addr: "notevil.example.com".to_string(),
            remote_port: 443,
            state: "ESTABLISHED".to_string(),
        }];

        let c2_db = C2Database {
            domains: vec![C2Indicator {
                value: "evil.example.com".to_string(),
                description: "Bad".to_string(),
                severity: Severity::Critical,
            }],
            ip_ranges: vec![],
        };

        let findings = check_c2_connections(&connections, &c2_db);
        assert!(findings.is_empty());
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
    fn test_split_addr_port_ipv4() {
        assert_eq!(split_addr_port("1.2.3.4:443"), Some(("1.2.3.4", "443")));
    }

    #[test]
    fn test_split_addr_port_ipv6() {
        assert_eq!(split_addr_port("[::1]:443"), Some(("::1", "443")));
    }

    #[test]
    fn test_split_addr_port_no_port() {
        assert_eq!(split_addr_port("1.2.3.4"), None);
    }
}
