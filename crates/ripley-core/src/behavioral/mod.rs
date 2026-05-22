pub mod analyzer;
pub mod report;

pub use analyzer::{
    BehavioralError, analyze_behavior, detect_anomalies, extract_declared_behavior,
    sandbox_result_to_observed,
};
pub use report::{
    AnomalyKind, BehavioralAnomaly, BehavioralReport, DeclaredBehavior, FsRead, FsWrite,
    NetworkAttempt, ObservedBehavior, ProcessSpawn,
};
