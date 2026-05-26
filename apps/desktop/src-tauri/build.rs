use std::path::PathBuf;
use std::process::Command;

fn main() {
    stage_script_shell_sidecar();
    tauri_build::build();
}

/// Stage `ripley-script-shell` into `binaries/<name>-<target-triple>(.exe)`
/// so Tauri's external-binary bundler picks it up. We honor the same profile
/// as the host build (debug vs release) and fail soft: bundling without the
/// sidecar is still useful for `tauri dev`, and `cargo check`/CI can build
/// the lib crate without the workspace producing the binary first.
fn stage_script_shell_sidecar() {
    println!("cargo:rerun-if-changed=../../../crates/ripley-guard/src/bin/ripley-script-shell.rs");
    println!("cargo:rerun-if-env-changed=PROFILE");
    println!("cargo:rerun-if-env-changed=TARGET");
    println!("cargo:rerun-if-env-changed=RIPLEY_SKIP_SIDECAR");

    if std::env::var_os("RIPLEY_SKIP_SIDECAR").is_some() {
        println!("cargo:warning=RIPLEY_SKIP_SIDECAR set; skipping sidecar staging");
        return;
    }

    let target = match host_triple() {
        Some(t) => t,
        None => {
            println!("cargo:warning=could not resolve target triple; skipping sidecar staging");
            return;
        }
    };

    let manifest_dir = PathBuf::from(env_or("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .ancestors()
        .nth(3)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    let exe_name = if cfg!(windows) {
        "ripley-script-shell.exe"
    } else {
        "ripley-script-shell"
    };

    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let candidate = workspace_root.join("target").join(&profile).join(exe_name);

    let dest_dir = manifest_dir.join("binaries");
    if let Err(e) = std::fs::create_dir_all(&dest_dir) {
        println!("cargo:warning=could not create binaries dir: {e}");
        return;
    }

    let suffix = if cfg!(windows) { ".exe" } else { "" };
    let dest = dest_dir.join(format!("ripley-script-shell-{target}{suffix}"));

    if !candidate.exists() {
        // tauri-build validates externalBin resource paths, so we always need
        // the destination to exist. Stage a stub so `cargo build --workspace`
        // and `cargo check` succeed; bundling without a real sidecar still
        // produces a working dev shell because the daemon path is opt-in.
        if !dest.exists() {
            if let Err(e) = std::fs::write(&dest, b"#!/bin/sh\nexit 0\n") {
                println!("cargo:warning=could not write sidecar stub: {e}");
                return;
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(0o755));
            }
        }
        println!(
            "cargo:warning=ripley-script-shell not built at {}; staged a stub \
             sidecar. Run `cargo build -p ripley-guard --bin ripley-script-shell` \
             before `tauri build` to bundle the real binary.",
            candidate.display()
        );
        return;
    }

    if let Err(e) = std::fs::copy(&candidate, &dest) {
        println!(
            "cargo:warning=could not copy sidecar to {}: {e}",
            dest.display()
        );
    }
}

fn host_triple() -> Option<String> {
    if let Ok(t) = std::env::var("TARGET") {
        return Some(t);
    }
    let out = Command::new("rustc").arg("-vV").output().ok()?;
    let stdout = String::from_utf8(out.stdout).ok()?;
    for line in stdout.lines() {
        if let Some(rest) = line.strip_prefix("host: ") {
            return Some(rest.trim().to_string());
        }
    }
    None
}

fn env_or(name: &str) -> String {
    std::env::var(name).unwrap_or_default()
}
