use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::forensic::network::NetworkConnection;

#[derive(Debug, Clone, Serialize)]
pub struct ProcessSnapshot {
    pub pid: u32,
    pub name: String,
    pub cmdline: String,
    pub open_files: Vec<String>,
    pub network_connections: Vec<NetworkConnection>,
    pub env_vars: Vec<(String, String)>,
    pub children: Vec<u32>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContainResult {
    pub snapshot_path: PathBuf,
    pub killed: bool,
    pub snapshot: ProcessSnapshot,
}

pub fn collect_process_snapshot(pid: u32) -> Result<ProcessSnapshot, std::io::Error> {
    #[cfg(target_os = "macos")]
    {
        collect_snapshot_macos(pid)
    }

    #[cfg(target_os = "linux")]
    {
        collect_snapshot_linux(pid)
    }

    #[cfg(target_os = "windows")]
    {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "process snapshot not implemented on Windows",
        ))
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "process snapshot not implemented on this platform",
        ))
    }
}

#[cfg(target_os = "macos")]
fn collect_snapshot_macos(pid: u32) -> Result<ProcessSnapshot, std::io::Error> {
    let lsof_output = std::process::Command::new("lsof")
        .args(["-p", &pid.to_string()])
        .output()?;
    let lsof_str = String::from_utf8_lossy(&lsof_output.stdout);

    let ps_output = std::process::Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "comm=,args="])
        .output()?;
    let ps_str = String::from_utf8_lossy(&ps_output.stdout);

    let pgrep_output = std::process::Command::new("pgrep")
        .args(["-P", &pid.to_string()])
        .output()?;
    let pgrep_str = String::from_utf8_lossy(&pgrep_output.stdout);

    let lsof_net = std::process::Command::new("lsof")
        .args(["-i", "-n", "-P", "-a", "-p", &pid.to_string()])
        .output()?;
    let lsof_net_str = String::from_utf8_lossy(&lsof_net.stdout);

    Ok(evaluate_snapshot_output(
        &lsof_str,
        &ps_str,
        &pgrep_str,
        &lsof_net_str,
        pid,
    ))
}

#[cfg(target_os = "linux")]
fn collect_snapshot_linux(pid: u32) -> Result<ProcessSnapshot, std::io::Error> {
    let cmdline = std::fs::read_to_string(format!("/proc/{pid}/cmdline"))
        .unwrap_or_default()
        .replace('\0', " ")
        .trim()
        .to_string();

    let name = std::fs::read_to_string(format!("/proc/{pid}/comm"))
        .unwrap_or_default()
        .trim()
        .to_string();

    let open_files = match std::fs::read_dir(format!("/proc/{pid}/fd")) {
        Ok(entries) => entries
            .flatten()
            .filter_map(|e| std::fs::read_link(e.path()).ok())
            .map(|p| p.to_string_lossy().to_string())
            .collect(),
        Err(_) => Vec::new(),
    };

    let env_str = std::fs::read_to_string(format!("/proc/{pid}/environ")).unwrap_or_default();
    let env_vars = parse_proc_environ(&env_str);

    let children = match std::process::Command::new("pgrep")
        .args(["-P", &pid.to_string()])
        .output()
    {
        Ok(output) => parse_child_pids(&String::from_utf8_lossy(&output.stdout)),
        Err(_) => Vec::new(),
    };

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    Ok(ProcessSnapshot {
        pid,
        name,
        cmdline,
        open_files,
        network_connections: Vec::new(),
        env_vars,
        children,
        timestamp: now,
    })
}

pub fn evaluate_snapshot_output(
    lsof_output: &str,
    ps_output: &str,
    pgrep_output: &str,
    lsof_net_output: &str,
    pid: u32,
) -> ProcessSnapshot {
    let open_files = parse_lsof_files(lsof_output);
    let (name, cmdline) = parse_ps_output(ps_output);
    let children = parse_child_pids(pgrep_output);
    let network_connections = crate::forensic::network::parse_lsof_output(lsof_net_output);

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    ProcessSnapshot {
        pid,
        name,
        cmdline,
        open_files,
        network_connections,
        env_vars: Vec::new(),
        children,
        timestamp: now,
    }
}

fn parse_lsof_files(output: &str) -> Vec<String> {
    output
        .lines()
        .skip(1)
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            parts.last().map(|s| s.to_string())
        })
        .collect()
}

fn parse_ps_output(output: &str) -> (String, String) {
    let trimmed = output.trim();
    if trimmed.is_empty() {
        return (String::new(), String::new());
    }
    let parts: Vec<&str> = trimmed.splitn(2, char::is_whitespace).collect();
    let name = parts.first().unwrap_or(&"").to_string();
    let cmdline = parts.get(1).unwrap_or(&"").trim().to_string();
    (name, cmdline)
}

pub fn parse_child_pids(output: &str) -> Vec<u32> {
    output
        .lines()
        .filter_map(|line| line.trim().parse::<u32>().ok())
        .collect()
}

pub fn parse_proc_environ(data: &str) -> Vec<(String, String)> {
    data.split('\0')
        .filter(|s| !s.is_empty())
        .filter_map(|entry| {
            entry
                .split_once('=')
                .map(|(k, v)| (k.to_string(), v.to_string()))
        })
        .collect()
}

pub fn contain_process(pid: u32, data_dir: &Path) -> Result<ContainResult, std::io::Error> {
    let snapshot = collect_process_snapshot(pid)?;

    let snapshots_dir = data_dir.join("snapshots");
    std::fs::create_dir_all(&snapshots_dir)?;

    let snapshot_filename = format!("{}_{}.json", pid, snapshot.timestamp);
    let snapshot_path = snapshots_dir.join(&snapshot_filename);

    let json = serde_json::to_string_pretty(&snapshot).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, format!("serialize: {e}"))
    })?;

    let temp_path = snapshot_path.with_extension("json.tmp");
    std::fs::write(&temp_path, json.as_bytes())?;
    std::fs::rename(&temp_path, &snapshot_path)?;

    let killed = kill_process(pid);

    Ok(ContainResult {
        snapshot_path,
        killed,
        snapshot,
    })
}

fn kill_process(pid: u32) -> bool {
    #[cfg(unix)]
    {
        let result = std::process::Command::new("kill")
            .args(["-9", &pid.to_string()])
            .status();
        matches!(result, Ok(status) if status.success())
    }

    #[cfg(not(unix))]
    {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_snapshot_output() {
        let lsof = "COMMAND  PID USER  FD   TYPE DEVICE SIZE/OFF NODE NAME\n\
                     node    1234 jack  txt    REG    1,4  1234567 123 /usr/local/bin/node\n\
                     node    1234 jack   3u   REG    1,4     4096 456 /tmp/data.txt\n";
        let ps = "node /usr/local/bin/node server.js";
        let pgrep = "5678\n5679\n";
        let lsof_net = "";

        let snap = evaluate_snapshot_output(lsof, ps, pgrep, lsof_net, 1234);
        assert_eq!(snap.pid, 1234);
        assert_eq!(snap.name, "node");
        assert_eq!(snap.cmdline, "/usr/local/bin/node server.js");
        assert_eq!(snap.open_files.len(), 2);
        assert_eq!(snap.children, vec![5678, 5679]);
    }

    #[test]
    fn test_parse_lsof_files() {
        let output = "COMMAND  PID USER  FD   TYPE DEVICE SIZE/OFF NODE NAME\n\
                       node    1234 jack  txt    REG    1,4  1234567 123 /usr/local/bin/node\n";
        let files = parse_lsof_files(output);
        assert_eq!(files, vec!["/usr/local/bin/node"]);
    }

    #[test]
    fn test_parse_lsof_files_empty() {
        let files = parse_lsof_files("");
        assert!(files.is_empty());
    }

    #[test]
    fn test_parse_ps_output() {
        let (name, cmdline) = parse_ps_output("node /usr/local/bin/node server.js");
        assert_eq!(name, "node");
        assert_eq!(cmdline, "/usr/local/bin/node server.js");
    }

    #[test]
    fn test_parse_ps_output_empty() {
        let (name, cmdline) = parse_ps_output("");
        assert!(name.is_empty());
        assert!(cmdline.is_empty());
    }

    #[test]
    fn test_parse_child_pids() {
        assert_eq!(parse_child_pids("5678\n5679\n"), vec![5678, 5679]);
    }

    #[test]
    fn test_parse_child_pids_empty() {
        assert!(parse_child_pids("").is_empty());
    }

    #[test]
    fn test_parse_child_pids_with_junk() {
        assert_eq!(parse_child_pids("abc\n1234\n"), vec![1234]);
    }

    #[test]
    fn test_parse_proc_environ() {
        let data = "HOME=/home/user\0PATH=/usr/bin\0SHELL=/bin/bash\0";
        let vars = parse_proc_environ(data);
        assert_eq!(vars.len(), 3);
        assert_eq!(vars[0], ("HOME".to_string(), "/home/user".to_string()));
    }

    #[test]
    fn test_parse_proc_environ_empty() {
        let vars = parse_proc_environ("");
        assert!(vars.is_empty());
    }

    #[test]
    fn test_snapshot_serializes_to_json() {
        let snap = ProcessSnapshot {
            pid: 1234,
            name: "node".to_string(),
            cmdline: "node server.js".to_string(),
            open_files: vec!["/tmp/data.txt".to_string()],
            network_connections: Vec::new(),
            env_vars: vec![("HOME".to_string(), "/home/user".to_string())],
            children: vec![5678],
            timestamp: 1700000000,
        };

        let json = serde_json::to_string(&snap).expect("serialize");
        assert!(json.contains("\"pid\":1234"));
        assert!(json.contains("\"name\":\"node\""));
        assert!(json.contains("\"children\":[5678]"));
    }

    #[test]
    fn test_snapshot_path_format() {
        let dir = tempfile::tempdir().expect("tempdir");
        let snapshots_dir = dir.path().join("snapshots");
        std::fs::create_dir_all(&snapshots_dir).expect("mkdir");

        let filename = format!("{}_{}.json", 1234, 1700000000u64);
        let path = snapshots_dir.join(&filename);
        assert_eq!(
            path.file_name().unwrap().to_str().unwrap(),
            "1234_1700000000.json"
        );
    }

    #[test]
    fn test_contain_result_serializes() {
        let result = ContainResult {
            snapshot_path: PathBuf::from("/data/snapshots/1234_1700000000.json"),
            killed: true,
            snapshot: ProcessSnapshot {
                pid: 1234,
                name: "node".to_string(),
                cmdline: "node evil.js".to_string(),
                open_files: Vec::new(),
                network_connections: Vec::new(),
                env_vars: Vec::new(),
                children: Vec::new(),
                timestamp: 1700000000,
            },
        };

        let json = serde_json::to_string_pretty(&result).expect("serialize");
        assert!(json.contains("\"killed\": true"));
        assert!(json.contains("\"snapshot_path\""));
    }
}
