use std::time::Instant;

use serde::Serialize;

use super::profile::SandboxProfile;

#[derive(Debug, Clone, Serialize)]
pub struct SandboxResult {
    pub exit_code: i32,
    pub stderr_output: String,
    pub stdout_output: String,
    pub network_blocked: bool,
    pub duration_ms: u64,
    pub sandbox_violations: Vec<SandboxViolation>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SandboxViolation {
    pub kind: ViolationKind,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ViolationKind {
    NetworkAccess,
    FileWriteOutsideScope,
    ProcessSpawn,
    Other,
}

#[derive(Debug, thiserror::Error)]
pub enum SandboxError {
    #[error("sandbox-exec not available on this system")]
    SandboxExecNotFound,
    #[error("bwrap not found — install with: sudo apt install bubblewrap")]
    BwrapNotFound,
    #[error("sandbox profile generation failed: {0}")]
    ProfileGenerationFailed(String),
    #[error("sandbox execution failed: {0}")]
    ExecutionFailed(#[from] std::io::Error),
    #[error("sandboxing not supported on this platform")]
    Unsupported,
}

pub fn execute_sandboxed(
    script: &str,
    args: &[String],
    profile: &SandboxProfile,
) -> Result<SandboxResult, SandboxError> {
    #[cfg(target_os = "macos")]
    {
        execute_sandboxed_macos(script, args, profile)
    }
    #[cfg(target_os = "linux")]
    {
        execute_sandboxed_linux(script, args, profile)
    }
    #[cfg(target_os = "windows")]
    {
        let _ = (script, args, profile);
        Err(SandboxError::Unsupported)
    }
}

#[cfg(target_os = "macos")]
fn execute_sandboxed_macos(
    script: &str,
    args: &[String],
    profile: &SandboxProfile,
) -> Result<SandboxResult, SandboxError> {
    let profile_content = profile.to_sandbox_exec_profile();

    let tmpdir = std::env::temp_dir();
    let profile_path = tmpdir.join(format!("ripley-sandbox-{}.sb", std::process::id()));
    std::fs::write(&profile_path, &profile_content)?;

    let start = Instant::now();

    let mut cmd = std::process::Command::new("sandbox-exec");
    cmd.arg("-f")
        .arg(&profile_path)
        .arg("/bin/sh")
        .arg("-c")
        .arg(script);

    for arg in args {
        cmd.arg(arg);
    }

    cmd.current_dir(&profile.working_dir);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let output = cmd.output()?;
    let duration_ms = start.elapsed().as_millis() as u64;

    let _ = std::fs::remove_file(&profile_path);

    let stderr_output = String::from_utf8_lossy(&output.stderr).to_string();
    let stdout_output = String::from_utf8_lossy(&output.stdout).to_string();
    let violations = parse_sandbox_exec_stderr(&stderr_output);
    let network_blocked = violations
        .iter()
        .any(|v| v.kind == ViolationKind::NetworkAccess);

    Ok(SandboxResult {
        exit_code: output.status.code().unwrap_or(-1),
        stderr_output,
        stdout_output,
        network_blocked,
        duration_ms,
        sandbox_violations: violations,
    })
}

#[cfg(target_os = "linux")]
fn execute_sandboxed_linux(
    script: &str,
    args: &[String],
    profile: &SandboxProfile,
) -> Result<SandboxResult, SandboxError> {
    if !crate::platform::is_command_available("bwrap") {
        return Err(SandboxError::BwrapNotFound);
    }

    let bwrap_args = profile.to_bwrap_args();
    let start = Instant::now();

    let mut cmd = std::process::Command::new("bwrap");
    for arg in &bwrap_args {
        cmd.arg(arg);
    }
    cmd.arg("/bin/sh").arg("-c").arg(script);

    for arg in args {
        cmd.arg(arg);
    }

    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let output = cmd.output()?;
    let duration_ms = start.elapsed().as_millis() as u64;

    let stderr_output = String::from_utf8_lossy(&output.stderr).to_string();
    let stdout_output = String::from_utf8_lossy(&output.stdout).to_string();
    let violations = parse_bwrap_stderr(&stderr_output);
    let network_blocked = violations
        .iter()
        .any(|v| v.kind == ViolationKind::NetworkAccess);

    Ok(SandboxResult {
        exit_code: output.status.code().unwrap_or(-1),
        stderr_output,
        stdout_output,
        network_blocked,
        duration_ms,
        sandbox_violations: violations,
    })
}

#[cfg(target_os = "macos")]
fn parse_sandbox_exec_stderr(stderr: &str) -> Vec<SandboxViolation> {
    let mut violations = Vec::new();
    for line in stderr.lines() {
        if line.contains("deny(1)") || line.contains("Sandbox:") {
            let kind = if line.contains("network") || line.contains("connect") {
                ViolationKind::NetworkAccess
            } else if line.contains("file-write") {
                ViolationKind::FileWriteOutsideScope
            } else if line.contains("process") {
                ViolationKind::ProcessSpawn
            } else {
                ViolationKind::Other
            };
            violations.push(SandboxViolation {
                kind,
                detail: line.to_string(),
            });
        }
    }
    violations
}

#[cfg(target_os = "linux")]
fn parse_bwrap_stderr(stderr: &str) -> Vec<SandboxViolation> {
    let mut violations = Vec::new();
    for line in stderr.lines() {
        if line.contains("Permission denied") || line.contains("Operation not permitted") {
            let kind = if line.contains("connect")
                || line.contains("Network is unreachable")
                || line.contains("network")
            {
                ViolationKind::NetworkAccess
            } else {
                ViolationKind::FileWriteOutsideScope
            };
            violations.push(SandboxViolation {
                kind,
                detail: line.to_string(),
            });
        } else if line.contains("Network is unreachable") {
            violations.push(SandboxViolation {
                kind: ViolationKind::NetworkAccess,
                detail: line.to_string(),
            });
        }
    }
    violations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_result_serialization() {
        let result = SandboxResult {
            exit_code: 0,
            stderr_output: String::new(),
            stdout_output: "hello\n".to_string(),
            network_blocked: false,
            duration_ms: 42,
            sandbox_violations: Vec::new(),
        };
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(json.contains("\"exit_code\":0"));
        assert!(json.contains("\"duration_ms\":42"));
    }

    #[test]
    fn test_violation_kind_serialization() {
        let v = SandboxViolation {
            kind: ViolationKind::NetworkAccess,
            detail: "denied connect".to_string(),
        };
        let json = serde_json::to_string(&v).expect("serialize");
        assert!(json.contains("\"network_access\""));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_parse_sandbox_exec_stderr_network() {
        let stderr = "sandbox-exec(12345) deny(1) network-outbound connect\n";
        let violations = parse_sandbox_exec_stderr(stderr);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].kind, ViolationKind::NetworkAccess);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_parse_sandbox_exec_stderr_file_write() {
        let stderr = "sandbox-exec(12345) deny(1) file-write* /etc/evil\n";
        let violations = parse_sandbox_exec_stderr(stderr);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].kind, ViolationKind::FileWriteOutsideScope);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_parse_sandbox_exec_stderr_clean() {
        let stderr = "normal script output here\n";
        let violations = parse_sandbox_exec_stderr(stderr);
        assert!(violations.is_empty());
    }

    #[cfg(target_os = "macos")]
    #[test]
    #[ignore] // sandbox-exec requires platform-specific permissions that vary by macOS version
    fn test_sandbox_exec_echo() {
        let profile = SandboxProfile {
            allow_network: false,
            writable_paths: vec![std::env::temp_dir()],
            readable_paths: vec![
                std::path::PathBuf::from("/usr"),
                std::path::PathBuf::from("/bin"),
                std::path::PathBuf::from("/dev/null"),
            ],
            working_dir: std::env::temp_dir(),
        };
        let result = execute_sandboxed("echo hello", &[], &profile);
        match result {
            Ok(r) => {
                assert_eq!(r.exit_code, 0);
                assert!(r.stdout_output.contains("hello"));
            }
            Err(SandboxError::SandboxExecNotFound) => {
                // sandbox-exec not available in this environment
            }
            Err(e) => panic!("unexpected error: {e}"),
        }
    }

    #[test]
    fn test_sandbox_error_display() {
        let err = SandboxError::BwrapNotFound;
        assert!(err.to_string().contains("bwrap not found"));

        let err = SandboxError::Unsupported;
        assert!(err.to_string().contains("not supported"));
    }
}
