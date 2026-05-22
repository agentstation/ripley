pub mod filesystem;
pub mod process;

pub use filesystem::{
    FsEventKind, evaluate_fs_event, is_lockfile_edit, is_mcp_config, is_persistence_path,
    persistence_watch_paths,
};
pub use process::{
    AlertReason, ProcessAlert, evaluate_connections, is_developer_process, merge_c2_database,
    scan_processes,
};
