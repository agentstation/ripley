use std::path::PathBuf;

use ripley_core::feed::Advisory;
use ripley_core::lockfile::InstalledPackage;
use ripley_core::matcher::Match;
use ripley_ipc::{Request, Response};
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum AppEvent {
    AdvisoriesUpdated(Vec<Advisory>),
    LockfileChanged {
        path: PathBuf,
        packages: Vec<InstalledPackage>,
    },
    MatchFound(Match),
    UserAction(Action),
    IpcRequest(Request, oneshot::Sender<Response>),
}

#[derive(Debug, Clone)]
pub enum Action {
    View(String),
    Fix(String),
    Dismiss(String),
    Contain(String),
}
