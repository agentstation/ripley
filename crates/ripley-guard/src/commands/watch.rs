use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use notify::{Event, EventKind, RecursiveMode, Watcher};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use ripley_core::config::{self, Config};
use ripley_core::db::AdvisoryDb;
use ripley_core::feed::osv::OsvClient;
use ripley_core::lockfile;
use ripley_core::matcher;
use ripley_core::types::Ecosystem;

pub async fn cmd_watch(daemon: bool) -> Result<()> {
    let config = config::load_config(&std::env::current_dir()?)?;
    let data_dir = ripley_core::dirs::data_dir()?;

    if daemon {
        let log_dir = data_dir.join("logs");
        std::fs::create_dir_all(&log_dir)?;
        let file_appender = tracing_appender::rolling::daily(&log_dir, "ripley.log");
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
        tracing_subscriber::fmt()
            .with_writer(non_blocking)
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .init();
    }

    let cancel = CancellationToken::new();

    let cancel_clone = cancel.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen for ctrl-c");
        tracing::info!("received shutdown signal");
        cancel_clone.cancel();
    });

    let socket_path = ripley_ipc::protocol::socket_path()
        .ok_or_else(|| anyhow::anyhow!("could not determine socket path"))?;

    let (event_tx, mut event_rx) = mpsc::channel::<WatchEvent>(64);

    let db_path = data_dir.join("advisories.redb");

    let poller_cancel = cancel.clone();
    let poller_config = config.clone();
    let poller_db = db_path.clone();
    let poller_tx = event_tx.clone();
    tokio::spawn(async move {
        run_poller(poller_config, poller_db, poller_tx, poller_cancel).await;
    });

    let watcher_cancel = cancel.clone();
    let watcher_config = config.clone();
    let watcher_tx = event_tx.clone();
    tokio::spawn(async move {
        run_watcher(watcher_config, watcher_tx, watcher_cancel).await;
    });

    let ipc_cancel = cancel.clone();
    tokio::spawn(async move {
        if let Err(e) = ripley_ipc::server::serve(
            &socket_path,
            |req| async move {
                match req {
                    ripley_ipc::Request::Status => {
                        ripley_ipc::Response::Status(ripley_ipc::protocol::StatusData {
                            last_poll: None,
                            alert_count: 0,
                            watching: Vec::new(),
                        })
                    }
                    _ => ripley_ipc::Response::Error("not implemented".into()),
                }
            },
            ipc_cancel,
        )
        .await
        {
            tracing::error!("IPC server error: {e}");
        }
    });

    eprintln!("ripley: watching for changes (Ctrl-C to stop)");

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                eprintln!("ripley: shutting down");
                break;
            }
            Some(event) = event_rx.recv() => {
                match event {
                    WatchEvent::NewAdvisories(count) => {
                        tracing::info!("fetched {count} advisories");
                    }
                    WatchEvent::LockfileChanged(path) => {
                        tracing::info!("lockfile changed: {}", path.display());
                        if let Err(e) = check_lockfile(&path, &db_path) {
                            tracing::warn!("check failed for {}: {e}", path.display());
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

#[derive(Debug)]
enum WatchEvent {
    NewAdvisories(usize),
    LockfileChanged(PathBuf),
}

async fn run_poller(
    config: Config,
    db_path: PathBuf,
    tx: mpsc::Sender<WatchEvent>,
    cancel: CancellationToken,
) {
    let interval = Duration::from_secs(config.general.poll_interval_secs);

    loop {
        tokio::select! {
            _ = cancel.cancelled() => return,
            _ = tokio::time::sleep(interval) => {
                match poll_once(&config, &db_path).await {
                    Ok(count) => {
                        let _ = tx.send(WatchEvent::NewAdvisories(count)).await;
                    }
                    Err(e) => tracing::warn!("poll failed: {e}"),
                }
            }
        }
    }
}

async fn poll_once(config: &Config, db_path: &std::path::Path) -> Result<usize> {
    let cache_dir = ripley_core::dirs::cache_dir()?;
    let client = OsvClient::new(cache_dir)?;
    let db = AdvisoryDb::open(db_path)?;

    let mut total = 0;
    for root in &config.monitoring.project_roots {
        let lockfiles = lockfile::find_lockfiles(std::path::Path::new(root));
        for lf_path in lockfiles {
            let parsed = lockfile::parse_lockfile(&lf_path)?;
            let queries: Vec<(Ecosystem, String)> = parsed
                .packages
                .iter()
                .map(|p| (Ecosystem::Npm, p.name.clone()))
                .collect();

            let advisories = client.query_batch(&queries).await.unwrap_or_default();
            for advisory in &advisories {
                db.store_advisories(
                    advisory.ecosystem,
                    &advisory.package,
                    std::slice::from_ref(advisory),
                )?;
            }
            total += advisories.len();
        }
    }

    db.set_last_poll(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    )?;

    Ok(total)
}

async fn run_watcher(config: Config, tx: mpsc::Sender<WatchEvent>, cancel: CancellationToken) {
    let (fs_tx, mut fs_rx) = mpsc::channel::<PathBuf>(64);

    let mut watcher = match notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        if let Ok(event) = res
            && matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_))
        {
            for path in event.paths {
                if path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n == "package-lock.json")
                {
                    let _ = fs_tx.blocking_send(path);
                }
            }
        }
    }) {
        Ok(w) => w,
        Err(e) => {
            tracing::error!("failed to start watcher: {e}");
            return;
        }
    };

    for root in &config.monitoring.project_roots {
        let path = std::path::Path::new(root);
        if path.exists()
            && let Err(e) = watcher.watch(path, RecursiveMode::Recursive)
        {
            tracing::warn!("failed to watch {root}: {e}");
        }
    }

    loop {
        tokio::select! {
            _ = cancel.cancelled() => return,
            Some(path) = fs_rx.recv() => {
                let _ = tx.send(WatchEvent::LockfileChanged(path)).await;
            }
        }
    }
}

fn check_lockfile(lockfile_path: &std::path::Path, db_path: &std::path::Path) -> Result<()> {
    let parsed = lockfile::parse_lockfile(lockfile_path)?;
    let db = AdvisoryDb::open(db_path)?;
    let all_advisories = db.get_all_advisories()?;

    let project_path = lockfile_path
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .to_path_buf();
    let matches = matcher::find_matches(&all_advisories, &parsed.packages, &project_path);

    if !matches.is_empty() {
        for m in &matches {
            let severity = m
                .advisory
                .severity
                .as_ref()
                .map(|s| format!("{s}"))
                .unwrap_or_else(|| "unknown".into());
            tracing::warn!(
                "[{severity}] {}@{} — {}",
                m.advisory.package,
                m.package.version,
                m.advisory.summary
            );

            let _ = notify_rust::Notification::new()
                .appname("Ripley")
                .summary(&format!(
                    "{}@{} is vulnerable",
                    m.advisory.package, m.package.version
                ))
                .body(&format!("{} · {}", m.advisory.id, m.advisory.summary))
                .show();
        }
    }

    Ok(())
}
