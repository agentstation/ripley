use std::path::PathBuf;

use ripley_core::feed::Advisory;
use ripley_core::lockfile::InstalledPackage;
use ripley_core::matcher::Match;
use ripley_ipc::{Request, Response};
use tokio::sync::oneshot;

#[derive(Debug)]
#[allow(dead_code)]
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
#[allow(dead_code)]
pub enum Action {
    View(String),
    Fix(String),
    Dismiss(String),
    Contain(String),
}
