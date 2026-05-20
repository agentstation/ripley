mod app;
mod events;
mod notifier;
mod poller;
mod theme;
mod tray;
mod views;
mod watcher;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("ripley=info".parse().expect("valid directive")),
        )
        .init();

    tracing::info!("starting ripley tray app");

    if let Err(e) = app::run() {
        tracing::error!("fatal: {e}");
        std::process::exit(1);
    }
}
