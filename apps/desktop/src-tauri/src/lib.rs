use tauri_specta::{Builder, collect_commands, collect_events};

pub mod commands;
pub mod ipc_bridge;
pub mod prewarm;
pub mod tray;

use commands::diag::report_visible;
use commands::guard::submit_guard_decision;
use commands::ping::ping;
use ipc_bridge::GuardEventPayload;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type, tauri_specta::Event)]
pub struct GuardEvent(pub GuardEventPayload);

pub fn specta_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            ping,
            submit_guard_decision,
            report_visible
        ])
        .events(collect_events![GuardEvent])
}

pub fn export_bindings() -> Result<(), Box<dyn std::error::Error>> {
    let bindings_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("src")
        .join("lib")
        .join("bindings.ts");
    specta_builder().export(
        specta_typescript::Typescript::default().formatter(specta_typescript::formatter::prettier),
        bindings_path,
    )?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = specta_builder();

    #[cfg(debug_assertions)]
    let _ = export_bindings();

    let mut app = tauri::Builder::default();

    #[cfg(desktop)]
    {
        use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};

        let toggle_shortcut = Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyR);

        app = app.plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_shortcut(toggle_shortcut)
                .expect("failed to register global shortcut")
                .with_handler(move |app, shortcut, event| {
                    if shortcut == &toggle_shortcut
                        && event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed
                    {
                        let _ = prewarm::toggle(app);
                    }
                })
                .build(),
        );
    }

    let pending = ipc_bridge::new_pending();
    let latencies = ipc_bridge::new_latency_map();

    app.invoke_handler(builder.invoke_handler())
        .manage(pending.clone())
        .manage(latencies.clone())
        .setup(move |app| {
            builder.mount_events(app);

            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            tray::build(app.handle())?;

            #[cfg(unix)]
            {
                use tauri_specta::Event as _;

                let socket_path = match ripley_ipc::protocol::socket_path() {
                    Some(p) => p,
                    None => {
                        tracing::warn!("no socket path; guard bridge disabled");
                        return Ok(());
                    }
                };
                let app_handle = app.handle().clone();
                let pending = pending.clone();
                let latencies = latencies.clone();
                let cancel = tokio_util::sync::CancellationToken::new();
                tauri::async_runtime::spawn(async move {
                    let emit = move |payload: GuardEventPayload| {
                        if let Err(e) = prewarm::show_on_event(&app_handle) {
                            tracing::warn!("show_on_event failed: {e}");
                        }
                        let _ = GuardEvent(payload).emit(&app_handle);
                    };
                    if let Err(e) =
                        ipc_bridge::serve(&socket_path, pending, latencies, emit, cancel).await
                    {
                        tracing::error!("guard bridge exited with error: {e}");
                    }
                });
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
