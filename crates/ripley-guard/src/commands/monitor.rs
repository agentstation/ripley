use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::{Result, bail};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use ripley_core::config::{self, Config};
use ripley_core::forensic::network::C2Database;
use ripley_core::monitor::process::ProcessAlert;
use ripley_core::monitor::{
    self, FsEventKind, MonitorEventType, MonitorLogEntry, format_log_entry,
};
use ripley_core::types::Severity;

#[derive(Debug)]
enum MonitorEvent {
    ProcessAlert(ProcessAlert),
    FsAlert(ProcessAlert),
}

pub async fn cmd_monitor(daemon: bool, format: &str) -> Result<()> {
    let config = config::load_config(&std::env::current_dir()?)?;

    if !config.monitor.enabled {
        bail!(
            "Runtime monitoring is disabled. Enable it in config.toml:\n\n\
             [monitor]\n\
             enabled = true\n\n\
             Or set RIPLEY_MONITOR_ENABLED=true"
        );
    }

    let data_dir = ripley_core::dirs::data_dir()?;

    let _log_guard = if daemon {
        let log_dir = data_dir.join("logs");
        std::fs::create_dir_all(&log_dir)?;
        let file_appender = tracing_appender::rolling::daily(&log_dir, "ripley-monitor.log");
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
        tracing_subscriber::fmt()
            .with_writer(non_blocking)
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .init();
        Some(guard)
    } else {
        None
    };

    let cancel = CancellationToken::new();

    let cancel_clone = cancel.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen for ctrl-c");
        tracing::info!("received shutdown signal");
        cancel_clone.cancel();
    });

    let (event_tx, mut event_rx) = mpsc::channel::<MonitorEvent>(64);

    let c2_db = C2Database::load_compiled();
    let merged_c2 = monitor::merge_c2_database(&c2_db, &config.monitor);

    if config.monitor.watch_processes {
        let scanner_cancel = cancel.clone();
        let scanner_config = config.clone();
        let scanner_c2 = merged_c2.clone();
        let scanner_tx = event_tx.clone();
        tokio::spawn(async move {
            run_process_scanner(scanner_config, scanner_c2, scanner_tx, scanner_cancel).await;
        });
    }

    if config.monitor.watch_persistence || config.monitor.watch_lockfiles {
        let watcher_cancel = cancel.clone();
        let watcher_config = config.clone();
        let watcher_tx = event_tx.clone();
        tokio::spawn(async move {
            run_persistence_watcher(watcher_config, watcher_tx, watcher_cancel).await;
        });
    }

    drop(event_tx);

    let guard_log_path = data_dir.join("guard.jsonl");
    let is_json = format == "json";

    log_entry(
        &guard_log_path,
        &MonitorLogEntry::lifecycle(MonitorEventType::MonitorStarted, "Monitor daemon started"),
    );

    if !is_json {
        eprintln!("ripley: monitor active (Ctrl-C to stop)");
    }

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                log_entry(
                    &guard_log_path,
                    &MonitorLogEntry::lifecycle(MonitorEventType::MonitorStopped, "Monitor daemon stopped"),
                );
                if !is_json {
                    eprintln!("ripley: monitor shutting down");
                }
                break;
            }
            event = event_rx.recv() => {
                match event {
                    Some(MonitorEvent::ProcessAlert(alert) | MonitorEvent::FsAlert(alert)) => {
                        print_alert(&alert, is_json);
                        log_entry(&guard_log_path, &MonitorLogEntry::from_alert(&alert));
                    }
                    None => {
                        tracing::warn!("all monitor workers exited");
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

fn print_alert(alert: &ProcessAlert, json: bool) {
    if json {
        if let Ok(line) = serde_json::to_string(alert) {
            println!("{line}");
        }
    } else {
        let severity_label = match alert.severity {
            Severity::Critical => "CRIT",
            Severity::High => "HIGH",
            Severity::Medium => "MED ",
            Severity::Low => "LOW ",
        };
        let severity_icon = match alert.severity {
            Severity::Critical | Severity::High => "●",
            Severity::Medium => "◐",
            Severity::Low => "○",
        };
        let time = format_utc_time(alert.timestamp);
        let process_info = if alert.connection.pid > 0 {
            format!(
                "{} (PID {})",
                alert.connection.process, alert.connection.pid
            )
        } else {
            String::new()
        };

        eprintln!(
            "[{time}] {severity_icon} {severity_label}  {process_info} {reason}",
            reason = alert.reason
        );
    }
}

fn format_utc_time(timestamp: u64) -> String {
    let secs = timestamp % 86400;
    let hours = secs / 3600;
    let mins = (secs % 3600) / 60;
    let s = secs % 60;
    format!("{hours:02}:{mins:02}:{s:02}Z")
}

fn log_entry(path: &std::path::Path, entry: &MonitorLogEntry) {
    if let Ok(line) = format_log_entry(entry) {
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = writeln!(file, "{line}");
        }
    }
}

async fn run_process_scanner(
    config: Config,
    c2_db: C2Database,
    tx: mpsc::Sender<MonitorEvent>,
    cancel: CancellationToken,
) {
    let scan_interval = Duration::from_secs(30);
    let dedup_ttl = Duration::from_secs(300);
    let mut seen: HashMap<String, Instant> = HashMap::new();

    loop {
        tokio::select! {
            _ = cancel.cancelled() => return,
            _ = tokio::time::sleep(scan_interval) => {
                seen.retain(|_, when| when.elapsed() < dedup_ttl);

                let monitor_config = config.monitor.clone();
                let c2 = c2_db.clone();
                let alerts = tokio::task::spawn_blocking(move || {
                    monitor::scan_processes(&monitor_config, &c2)
                }).await;

                match alerts {
                    Ok(Ok(alerts)) => {
                        for alert in alerts {
                            let key = format!(
                                "{}:{}:{}",
                                alert.connection.process,
                                alert.connection.remote_addr,
                                alert.connection.remote_port
                            );
                            if seen.contains_key(&key) {
                                continue;
                            }
                            seen.insert(key, Instant::now());
                            let _ = tx.send(MonitorEvent::ProcessAlert(alert)).await;
                        }
                    }
                    Ok(Err(e)) => {
                        tracing::warn!("process scan failed: {e}");
                    }
                    Err(e) => {
                        tracing::warn!("process scan task panicked: {e}");
                    }
                }
            }
        }
    }
}

async fn run_persistence_watcher(
    config: Config,
    tx: mpsc::Sender<MonitorEvent>,
    cancel: CancellationToken,
) {
    let (fs_tx, mut fs_rx) = mpsc::channel::<(PathBuf, FsEventKind)>(64);

    let _watcher = match setup_persistence_watcher(&config, fs_tx) {
        Ok(w) => w,
        Err(e) => {
            tracing::error!("failed to start persistence watcher: {e}");
            return;
        }
    };

    let mut last_events: HashMap<PathBuf, Instant> = HashMap::new();
    let debounce = Duration::from_millis(100);

    loop {
        tokio::select! {
            _ = cancel.cancelled() => return,
            Some((path, kind)) = fs_rx.recv() => {
                if let Some(last) = last_events.get(&path)
                    && last.elapsed() < debounce
                {
                    continue;
                }
                last_events.insert(path.clone(), Instant::now());

                if let Some(alert) = monitor::evaluate_fs_event(&path, kind) {
                    let _ = tx.send(MonitorEvent::FsAlert(alert)).await;
                }
            }
        }
    }
}

fn setup_persistence_watcher(
    config: &Config,
    tx: mpsc::Sender<(PathBuf, FsEventKind)>,
) -> Result<notify::RecommendedWatcher> {
    let watch_lockfiles = config.monitor.watch_lockfiles;

    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        if let Ok(event) = res {
            let kind = match event.kind {
                EventKind::Create(_) => FsEventKind::Created,
                EventKind::Modify(_) => FsEventKind::Modified,
                EventKind::Remove(_) => FsEventKind::Deleted,
                _ => return,
            };

            for path in event.paths {
                let dominated = monitor::is_persistence_path(&path)
                    || monitor::is_mcp_config(&path)
                    || (watch_lockfiles && monitor::is_lockfile_edit(&path));
                if dominated {
                    let _ = tx.blocking_send((path, kind));
                }
            }
        }
    })?;

    let watch_paths = monitor::persistence_watch_paths();
    for path in &watch_paths {
        if path.exists() {
            if path.is_dir() {
                if let Err(e) = watcher.watch(path, RecursiveMode::Recursive) {
                    tracing::warn!("failed to watch {}: {e}", path.display());
                }
            } else if let Some(parent) = path.parent()
                && parent.exists()
                && let Err(e) = watcher.watch(parent, RecursiveMode::NonRecursive)
            {
                tracing::warn!("failed to watch {}: {e}", parent.display());
            }
        }
    }

    if config.monitor.watch_lockfiles {
        for root in &config.monitoring.project_roots {
            let path = std::path::Path::new(root);
            if path.exists()
                && let Err(e) = watcher.watch(path, RecursiveMode::Recursive)
            {
                tracing::warn!("failed to watch {root}: {e}");
            }
        }
    }

    Ok(watcher)
}
