use std::time::Duration;

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use ripley_core::config::Config;
use ripley_core::db::AdvisoryDb;
use ripley_core::feed::osv::OsvClient;
use ripley_core::types::Ecosystem;

use crate::events::AppEvent;

#[allow(dead_code)]
pub async fn run(
    config: Config,
    db_path: std::path::PathBuf,
    tx: mpsc::Sender<AppEvent>,
    cancel: CancellationToken,
) {
    let interval = Duration::from_secs(config.general.poll_interval_secs);

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                tracing::info!("poller shutting down");
                return;
            }
            _ = tokio::time::sleep(interval) => {
                if let Err(e) = poll_once(&config, &db_path, &tx).await {
                    tracing::warn!("poll failed: {e}");
                }
            }
        }
    }
}

#[allow(dead_code)]
async fn poll_once(
    config: &Config,
    db_path: &std::path::Path,
    tx: &mpsc::Sender<AppEvent>,
) -> anyhow::Result<()> {
    let cache_dir = ripley_core::dirs::cache_dir()?;
    let client = OsvClient::new(cache_dir)?;

    let db = AdvisoryDb::open(db_path)?;

    let mut all_advisories = Vec::new();

    for root in &config.monitoring.project_roots {
        let lockfiles = ripley_core::lockfile::find_lockfiles(std::path::Path::new(root));
        for lf_path in lockfiles {
            let parsed = ripley_core::lockfile::parse_lockfile(&lf_path)?;
            let queries: Vec<(Ecosystem, String)> = parsed
                .packages
                .iter()
                .map(|p| (Ecosystem::Npm, p.name.clone()))
                .collect();

            let advisories = client.query_batch(&queries).await.unwrap_or_default();
            for advisory in &advisories {
                let pkg_name = &advisory.package;
                db.store_advisories(advisory.ecosystem, pkg_name, std::slice::from_ref(advisory))?;
            }
            all_advisories.extend(advisories);
        }
    }

    if !all_advisories.is_empty() {
        let _ = tx.send(AppEvent::AdvisoriesUpdated(all_advisories)).await;
    }

    db.set_last_poll(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    )?;

    Ok(())
}
