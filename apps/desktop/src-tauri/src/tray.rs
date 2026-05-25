use tauri::{
    AppHandle, Runtime,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};

use crate::prewarm;

pub fn build<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Open Ripley", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &quit])?;

    let _ = TrayIconBuilder::with_id("main-tray")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .icon(
            app.default_window_icon()
                .cloned()
                .unwrap_or_else(|| tauri::image::Image::new(&[], 0, 0).to_owned()),
        )
        .icon_as_template(true)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => {
                let _ = prewarm::show(app);
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}
