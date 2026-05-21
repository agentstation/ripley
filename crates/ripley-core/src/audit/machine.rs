use super::{AuditCategory, AuditError, AuditFinding, CategoryReport, TrafficLight};

pub fn collect_disk_encryption() -> Result<String, AuditError> {
    #[cfg(target_os = "macos")]
    {
        let output = std::process::Command::new("fdesetup")
            .arg("status")
            .output()
            .map_err(|e| AuditError::Command(format!("fdesetup: {e}")))?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    #[cfg(target_os = "linux")]
    {
        let output = std::process::Command::new("lsblk")
            .args(["--fs", "--pairs"])
            .output()
            .map_err(|e| AuditError::Command(format!("lsblk: {e}")))?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("manage-bde")
            .args(["-status", "C:"])
            .output()
            .map_err(|e| AuditError::Command(format!("manage-bde: {e}")))?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Err(AuditError::Command("unsupported platform".to_string()))
    }
}

pub fn evaluate_disk_encryption(output: &str) -> AuditFinding {
    #[cfg(target_os = "macos")]
    {
        if output.contains("FileVault is On") {
            AuditFinding {
                name: "Disk encryption".to_string(),
                status: TrafficLight::Green,
                detail: "FileVault is enabled".to_string(),
                fix_command: None,
            }
        } else if output.contains("FileVault is Off") {
            AuditFinding {
                name: "Disk encryption".to_string(),
                status: TrafficLight::Red,
                detail: "FileVault is disabled".to_string(),
                fix_command: Some("sudo fdesetup enable".to_string()),
            }
        } else {
            AuditFinding {
                name: "Disk encryption".to_string(),
                status: TrafficLight::Yellow,
                detail: "Could not determine FileVault status".to_string(),
                fix_command: None,
            }
        }
    }
    #[cfg(target_os = "linux")]
    {
        if output.contains("crypto_LUKS") || output.contains("crypt") {
            AuditFinding {
                name: "Disk encryption".to_string(),
                status: TrafficLight::Green,
                detail: "LUKS encryption detected".to_string(),
                fix_command: None,
            }
        } else {
            AuditFinding {
                name: "Disk encryption".to_string(),
                status: TrafficLight::Red,
                detail: "No disk encryption detected".to_string(),
                fix_command: None,
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        if output.contains("Protection On") || output.contains("Fully Encrypted") {
            AuditFinding {
                name: "Disk encryption".to_string(),
                status: TrafficLight::Green,
                detail: "BitLocker is enabled".to_string(),
                fix_command: None,
            }
        } else if output.contains("Protection Off") || output.contains("Fully Decrypted") {
            AuditFinding {
                name: "Disk encryption".to_string(),
                status: TrafficLight::Red,
                detail: "BitLocker is disabled".to_string(),
                fix_command: Some("manage-bde -on C:".to_string()),
            }
        } else {
            AuditFinding {
                name: "Disk encryption".to_string(),
                status: TrafficLight::Yellow,
                detail: "Could not determine BitLocker status".to_string(),
                fix_command: None,
            }
        }
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        let _ = output;
        AuditFinding {
            name: "Disk encryption".to_string(),
            status: TrafficLight::Yellow,
            detail: "Unsupported platform".to_string(),
            fix_command: None,
        }
    }
}

pub fn collect_firewall() -> Result<String, AuditError> {
    #[cfg(target_os = "macos")]
    {
        let output = std::process::Command::new("defaults")
            .args(["read", "/Library/Preferences/com.apple.alf", "globalstate"])
            .output()
            .map_err(|e| AuditError::Command(format!("defaults read alf: {e}")))?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    #[cfg(target_os = "linux")]
    {
        let output = std::process::Command::new("ufw")
            .arg("status")
            .output()
            .map_err(|e| AuditError::Command(format!("ufw: {e}")))?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("netsh")
            .args(["advfirewall", "show", "allprofiles", "state"])
            .output()
            .map_err(|e| AuditError::Command(format!("netsh: {e}")))?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Err(AuditError::Command("unsupported platform".to_string()))
    }
}

pub fn evaluate_firewall(output: &str) -> AuditFinding {
    #[cfg(target_os = "macos")]
    {
        let trimmed = output.trim();
        if trimmed == "1" || trimmed == "2" {
            AuditFinding {
                name: "Firewall".to_string(),
                status: TrafficLight::Green,
                detail: "Firewall is active".to_string(),
                fix_command: None,
            }
        } else if trimmed == "0" {
            AuditFinding {
                name: "Firewall".to_string(),
                status: TrafficLight::Red,
                detail: "Firewall is disabled".to_string(),
                fix_command: Some(
                    "sudo defaults write /Library/Preferences/com.apple.alf globalstate -int 1"
                        .to_string(),
                ),
            }
        } else {
            AuditFinding {
                name: "Firewall".to_string(),
                status: TrafficLight::Yellow,
                detail: format!("Unknown firewall state: {trimmed}"),
                fix_command: None,
            }
        }
    }
    #[cfg(target_os = "linux")]
    {
        if output.contains("Status: active") {
            AuditFinding {
                name: "Firewall".to_string(),
                status: TrafficLight::Green,
                detail: "UFW firewall is active".to_string(),
                fix_command: None,
            }
        } else if output.contains("Status: inactive") {
            AuditFinding {
                name: "Firewall".to_string(),
                status: TrafficLight::Red,
                detail: "UFW firewall is inactive".to_string(),
                fix_command: Some("sudo ufw enable".to_string()),
            }
        } else {
            AuditFinding {
                name: "Firewall".to_string(),
                status: TrafficLight::Yellow,
                detail: "Could not determine firewall status".to_string(),
                fix_command: None,
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        if output.contains("ON") {
            AuditFinding {
                name: "Firewall".to_string(),
                status: TrafficLight::Green,
                detail: "Windows Firewall is active".to_string(),
                fix_command: None,
            }
        } else if output.contains("OFF") {
            AuditFinding {
                name: "Firewall".to_string(),
                status: TrafficLight::Red,
                detail: "Windows Firewall is disabled".to_string(),
                fix_command: Some("netsh advfirewall set allprofiles state on".to_string()),
            }
        } else {
            AuditFinding {
                name: "Firewall".to_string(),
                status: TrafficLight::Yellow,
                detail: "Could not determine firewall status".to_string(),
                fix_command: None,
            }
        }
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        let _ = output;
        AuditFinding {
            name: "Firewall".to_string(),
            status: TrafficLight::Yellow,
            detail: "Unsupported platform".to_string(),
            fix_command: None,
        }
    }
}

pub fn collect_os_updates() -> Result<String, AuditError> {
    #[cfg(target_os = "macos")]
    {
        let output = std::process::Command::new("softwareupdate")
            .arg("-l")
            .output()
            .map_err(|e| AuditError::Command(format!("softwareupdate: {e}")))?;
        let combined = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(combined)
    }
    #[cfg(target_os = "linux")]
    {
        let output = std::process::Command::new("apt")
            .args(["list", "--upgradable"])
            .output()
            .or_else(|_| {
                std::process::Command::new("dnf")
                    .arg("check-update")
                    .output()
            })
            .map_err(|e| AuditError::Command(format!("package manager: {e}")))?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("powershell")
            .args(["-Command", "Get-HotFix | Sort-Object -Property InstalledOn -Descending | Select-Object -First 1 | Format-List InstalledOn"])
            .output()
            .map_err(|e| AuditError::Command(format!("powershell: {e}")))?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Err(AuditError::Command("unsupported platform".to_string()))
    }
}

pub fn evaluate_os_updates(output: &str) -> AuditFinding {
    #[cfg(target_os = "macos")]
    {
        if output.contains("No new software available") {
            AuditFinding {
                name: "OS updates".to_string(),
                status: TrafficLight::Green,
                detail: "No pending updates".to_string(),
                fix_command: None,
            }
        } else {
            let count = output.lines().filter(|l| l.contains("*")).count();
            if count == 0 {
                AuditFinding {
                    name: "OS updates".to_string(),
                    status: TrafficLight::Green,
                    detail: "No pending updates".to_string(),
                    fix_command: None,
                }
            } else {
                AuditFinding {
                    name: "OS updates".to_string(),
                    status: TrafficLight::Yellow,
                    detail: format!("{count} update(s) pending"),
                    fix_command: Some("softwareupdate -ia".to_string()),
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    {
        let upgradable = output
            .lines()
            .filter(|l| l.contains("upgradable") || l.contains("Upgrade"))
            .count();
        if upgradable == 0 {
            AuditFinding {
                name: "OS updates".to_string(),
                status: TrafficLight::Green,
                detail: "No pending updates".to_string(),
                fix_command: None,
            }
        } else {
            AuditFinding {
                name: "OS updates".to_string(),
                status: TrafficLight::Yellow,
                detail: format!("{upgradable} package(s) upgradable"),
                fix_command: Some("sudo apt upgrade -y".to_string()),
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        if output.trim().is_empty() {
            AuditFinding {
                name: "OS updates".to_string(),
                status: TrafficLight::Yellow,
                detail: "Could not determine update status".to_string(),
                fix_command: None,
            }
        } else {
            AuditFinding {
                name: "OS updates".to_string(),
                status: TrafficLight::Green,
                detail: "Recent hotfix detected".to_string(),
                fix_command: None,
            }
        }
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        let _ = output;
        AuditFinding {
            name: "OS updates".to_string(),
            status: TrafficLight::Yellow,
            detail: "Unsupported platform".to_string(),
            fix_command: None,
        }
    }
}

pub fn collect_screen_lock() -> Result<String, AuditError> {
    #[cfg(target_os = "macos")]
    {
        let output = std::process::Command::new("defaults")
            .args(["-currentHost", "read", "com.apple.screensaver", "idleTime"])
            .output()
            .map_err(|e| AuditError::Command(format!("defaults read screensaver: {e}")))?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    #[cfg(target_os = "linux")]
    {
        let output = std::process::Command::new("gsettings")
            .args(["get", "org.gnome.desktop.session", "idle-delay"])
            .output()
            .map_err(|e| AuditError::Command(format!("gsettings: {e}")))?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("powershell")
            .args(["-Command", "Get-ItemProperty -Path 'HKCU:\\Control Panel\\Desktop' -Name ScreenSaveTimeOut -ErrorAction SilentlyContinue | Select-Object -ExpandProperty ScreenSaveTimeOut"])
            .output()
            .map_err(|e| AuditError::Command(format!("powershell: {e}")))?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Err(AuditError::Command("unsupported platform".to_string()))
    }
}

pub fn evaluate_screen_lock(output: &str) -> AuditFinding {
    let trimmed = output.trim();

    let seconds: Option<u64> = if trimmed.starts_with("uint32 ") {
        trimmed.strip_prefix("uint32 ").and_then(|s| s.parse().ok())
    } else {
        trimmed.parse().ok()
    };

    match seconds {
        Some(0) => AuditFinding {
            name: "Screen lock".to_string(),
            status: TrafficLight::Red,
            detail: "Screen lock is disabled".to_string(),
            fix_command: None,
        },
        Some(s) if s <= 300 => AuditFinding {
            name: "Screen lock".to_string(),
            status: TrafficLight::Green,
            detail: format!("Screen locks after {s} seconds"),
            fix_command: None,
        },
        Some(s) if s <= 600 => AuditFinding {
            name: "Screen lock".to_string(),
            status: TrafficLight::Yellow,
            detail: format!("Screen locks after {s} seconds (>5 min)"),
            fix_command: None,
        },
        Some(s) => AuditFinding {
            name: "Screen lock".to_string(),
            status: TrafficLight::Red,
            detail: format!("Screen locks after {s} seconds (>10 min)"),
            fix_command: None,
        },
        None => AuditFinding {
            name: "Screen lock".to_string(),
            status: TrafficLight::Yellow,
            detail: "Could not determine screen lock timeout".to_string(),
            fix_command: None,
        },
    }
}

pub fn check_machine_security() -> CategoryReport {
    let mut findings = Vec::new();

    match collect_disk_encryption() {
        Ok(output) => findings.push(evaluate_disk_encryption(&output)),
        Err(_) => findings.push(AuditFinding {
            name: "Disk encryption".to_string(),
            status: TrafficLight::Yellow,
            detail: "Could not check disk encryption".to_string(),
            fix_command: None,
        }),
    }

    match collect_firewall() {
        Ok(output) => findings.push(evaluate_firewall(&output)),
        Err(_) => findings.push(AuditFinding {
            name: "Firewall".to_string(),
            status: TrafficLight::Yellow,
            detail: "Could not check firewall status".to_string(),
            fix_command: None,
        }),
    }

    match collect_os_updates() {
        Ok(output) => findings.push(evaluate_os_updates(&output)),
        Err(_) => findings.push(AuditFinding {
            name: "OS updates".to_string(),
            status: TrafficLight::Yellow,
            detail: "Could not check OS update status".to_string(),
            fix_command: None,
        }),
    }

    match collect_screen_lock() {
        Ok(output) => findings.push(evaluate_screen_lock(&output)),
        Err(_) => findings.push(AuditFinding {
            name: "Screen lock".to_string(),
            status: TrafficLight::Yellow,
            detail: "Could not check screen lock".to_string(),
            fix_command: None,
        }),
    }

    CategoryReport::new(AuditCategory::MachineSecurity, findings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_filevault_on() {
        let finding = evaluate_disk_encryption("FileVault is On.");
        assert_eq!(finding.status, TrafficLight::Green);
        assert!(finding.fix_command.is_none());
    }

    #[test]
    fn test_evaluate_filevault_off() {
        let finding = evaluate_disk_encryption("FileVault is Off.");
        assert_eq!(finding.status, TrafficLight::Red);
        assert!(finding.fix_command.is_some());
    }

    #[test]
    fn test_evaluate_firewall_active() {
        let finding = evaluate_firewall("1\n");
        assert_eq!(finding.status, TrafficLight::Green);
    }

    #[test]
    fn test_evaluate_firewall_off() {
        let finding = evaluate_firewall("0\n");
        assert_eq!(finding.status, TrafficLight::Red);
        assert!(finding.fix_command.is_some());
    }

    #[test]
    fn test_evaluate_screen_lock_5min() {
        let finding = evaluate_screen_lock("300");
        assert_eq!(finding.status, TrafficLight::Green);
    }

    #[test]
    fn test_evaluate_screen_lock_10min() {
        let finding = evaluate_screen_lock("600");
        assert_eq!(finding.status, TrafficLight::Yellow);
    }

    #[test]
    fn test_evaluate_screen_lock_disabled() {
        let finding = evaluate_screen_lock("0");
        assert_eq!(finding.status, TrafficLight::Red);
    }

    #[test]
    fn test_evaluate_screen_lock_gnome_format() {
        let finding = evaluate_screen_lock("uint32 300");
        assert_eq!(finding.status, TrafficLight::Green);
    }

    #[test]
    fn test_evaluate_os_updates_no_updates() {
        let finding = evaluate_os_updates("No new software available.");
        assert_eq!(finding.status, TrafficLight::Green);
    }

    #[test]
    fn test_evaluate_os_updates_pending() {
        let output = "Software Update found the following new or updated software:\n* macOS Sequoia 15.5-15.5\n";
        let finding = evaluate_os_updates(output);
        assert_eq!(finding.status, TrafficLight::Yellow);
        assert!(finding.detail.contains("1"));
    }
}
