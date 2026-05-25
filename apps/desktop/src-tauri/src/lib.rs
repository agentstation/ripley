use tauri_specta::{Builder, collect_commands};

pub mod commands;

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

    tauri::Builder::default()
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
