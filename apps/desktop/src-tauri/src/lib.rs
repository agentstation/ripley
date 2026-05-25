use tauri_specta::{Builder, collect_commands};

pub mod commands;
pub mod prewarm;
pub mod tray;

use commands::ping::ping;

pub fn specta_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new().commands(collect_commands![ping])
}

pub fn export_bindings() -> Result<(), Box<dyn std::error::Error>> {
    specta_builder().export(
        specta_typescript::Typescript::default().formatter(specta_typescript::formatter::prettier),
        "../src/lib/bindings.ts",
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

        let toggle_shortcut = Shortcut::new(
            Some(Modifiers::SUPER | Modifiers::SHIFT),
            Code::KeyR,
        );

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

    app.invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);

            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            tray::build(app.handle())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
