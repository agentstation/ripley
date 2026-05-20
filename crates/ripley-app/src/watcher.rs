use std::path::PathBuf;

use notify::{Event, EventKind, RecursiveMode, Watcher};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use ripley_core::config::Config;

use crate::events::AppEvent;

pub async fn run(config: Config, tx: mpsc::Sender<AppEvent>, cancel: CancellationToken) {
    let (fs_tx, mut fs_rx) = tokio::sync::mpsc::channel::<PathBuf>(64);

    let _watcher = match setup_watcher(&config, fs_tx) {
        Ok(w) => w,
        Err(e) => {
            tracing::error!("failed to start file watcher: {e}");
            return;
        }
    };

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                tracing::info!("watcher shutting down");
                return;
            }
            Some(path) = fs_rx.recv() => {
                match ripley_core::lockfile::parse_lockfile(&path) {
                    Ok(parsed) => {
                        let _ = tx.send(AppEvent::LockfileChanged {
                            path,
                            packages: parsed.packages,
                        }).await;
                    }
                    Err(e) => {
                        tracing::warn!("failed to reparse {}: {e}", path.display());
                    }
                }
            }
        }
    }
}

fn setup_watcher(
    config: &Config,
    tx: mpsc::Sender<PathBuf>,
) -> anyhow::Result<notify::RecommendedWatcher> {
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        if let Ok(event) = res
            && matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_))
        {
            for path in event.paths {
                if is_lockfile(&path) {
                    let _ = tx.blocking_send(path);
                }
            }
        }
    })?;

    for root in &config.monitoring.project_roots {
        let path = std::path::Path::new(root);
        if path.exists() {
            watcher.watch(path, RecursiveMode::Recursive)?;
        }
    }

    Ok(watcher)
}

fn is_lockfile(path: &std::path::Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|name| name == "package-lock.json")
}
