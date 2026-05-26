//! End-to-end guard-dialog latency bench.
//!
//! Sends N synthetic GuardPrompt events to the running desktop app via UDS,
//! waiting for the user to Allow each one. After completion it parses the
//! `guard-latency.jsonl` records the desktop process wrote for those events
//! and prints p50/p95 of the event_received → dialog_visible elapsed time.
//!
//! Usage: `guard-bench [N]` (default N=10). The desktop app must be running.

#[cfg(not(unix))]
fn main() {
    eprintln!("guard-bench: only supported on Unix targets (requires UDS)");
    std::process::exit(2);
}

#[cfg(unix)]
use std::io::{BufRead, BufReader};
#[cfg(unix)]
use std::path::PathBuf;
#[cfg(unix)]
use std::time::{Duration, Instant};

#[cfg(unix)]
use ripley_ipc::{
    client,
    protocol::{GuardPromptData, Request, Response},
};

#[cfg(unix)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let iterations: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);

    if !client::is_daemon_running() {
        return Err("desktop app not running (UDS missing)".into());
    }

    let log_path = latency_log_path().ok_or("no data dir available")?;
    let starting_lines = count_lines(&log_path).unwrap_or(0);

    println!("guard-bench: firing {iterations} events; click Allow on each dialog");

    let mut round_trips = Vec::with_capacity(iterations);
    for i in 0..iterations {
        let started = Instant::now();
        let req = Request::GuardPrompt(GuardPromptData {
            package: format!("bench-{i}"),
            version: "0.0.0".into(),
            script: "echo bench".into(),
            risk_level: "low".into(),
            matched_rules: vec!["BENCH".into()],
        });
        match client::send_request(&req).await {
            Ok(Response::GuardDecision(_)) => round_trips.push(started.elapsed()),
            Ok(other) => return Err(format!("unexpected response: {other:?}").into()),
            Err(e) => return Err(format!("send_request: {e}").into()),
        }
    }

    let new_records = read_new_records(&log_path, starting_lines, iterations)?;

    let visible_ms: Vec<u64> = new_records.iter().map(|r| r.elapsed_ms as u64).collect();
    let rt_ms: Vec<u64> = round_trips.iter().map(|d| d.as_millis() as u64).collect();

    print_percentiles("dialog_visible", &visible_ms);
    print_percentiles("round_trip", &rt_ms);
    Ok(())
}

#[cfg(unix)]
fn latency_log_path() -> Option<PathBuf> {
    let data_dir = ripley_core::dirs::data_dir().ok()?;
    Some(data_dir.join("guard-latency.jsonl"))
}

#[cfg(unix)]
fn count_lines(path: &PathBuf) -> std::io::Result<usize> {
    if !path.exists() {
        return Ok(0);
    }
    let file = std::fs::File::open(path)?;
    let count = BufReader::new(file).lines().count();
    Ok(count)
}

#[cfg(unix)]
#[derive(serde::Deserialize)]
struct LatencyRecord {
    #[allow(dead_code)]
    id: String,
    elapsed_ms: u128,
    #[allow(dead_code)]
    recorded_at_ms: u128,
}

#[cfg(unix)]
fn read_new_records(
    path: &PathBuf,
    skip: usize,
    expected: usize,
) -> std::io::Result<Vec<LatencyRecord>> {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let file = std::fs::File::open(path)?;
        let mut new_records = Vec::new();
        for line in BufReader::new(file).lines().skip(skip) {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(rec) = serde_json::from_str::<LatencyRecord>(&line) {
                new_records.push(rec);
            }
        }
        if new_records.len() >= expected || Instant::now() >= deadline {
            return Ok(new_records);
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[cfg(unix)]
fn print_percentiles(label: &str, samples: &[u64]) {
    if samples.is_empty() {
        println!("{label}: no samples");
        return;
    }
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let p = |q: f64| -> u64 {
        let idx = ((sorted.len() as f64 - 1.0) * q).round() as usize;
        sorted[idx]
    };
    println!(
        "{label}: n={} min={}ms p50={}ms p95={}ms max={}ms",
        sorted.len(),
        sorted.first().copied().unwrap_or(0),
        p(0.50),
        p(0.95),
        sorted.last().copied().unwrap_or(0),
    );
}
