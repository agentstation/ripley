use tauri_specta::{Builder, collect_commands};

fn main() {
    let builder = Builder::<tauri::Wry>::new().commands(collect_commands![]);

    builder
        .export(
            specta_typescript::Typescript::default()
                .formatter(specta_typescript::formatter::prettier),
            "../src/lib/bindings.ts",
        )
        .expect("failed to export typescript bindings");

    tauri_build::build();
}
