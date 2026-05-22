#![allow(dead_code)]

use tray_icon::TrayIconBuilder;
use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem};

pub enum TrayAction {
    ShowDashboard,
    ScanNow,
    ToggleMonitor,
    Quit,
}

pub struct RipleyTray {
    _tray: tray_icon::TrayIcon,
    show_id: MenuId,
    scan_id: MenuId,
    monitor_id: MenuId,
    quit_id: MenuId,
}

impl RipleyTray {
    pub fn new() -> anyhow::Result<Self> {
        let menu = Menu::new();

        let show_item = MenuItem::new("Show Dashboard", true, None);
        let scan_item = MenuItem::new("Scan Now...", true, None);
        let monitor_item = MenuItem::new("Monitor: Off", true, None);
        let quit_item = MenuItem::new("Quit Ripley", true, None);

        let show_id = show_item.id().clone();
        let scan_id = scan_item.id().clone();
        let monitor_id = monitor_item.id().clone();
        let quit_id = quit_item.id().clone();

        menu.append(&show_item)?;
        menu.append(&scan_item)?;
        menu.append(&monitor_item)?;
        menu.append(&PredefinedMenuItem::separator())?;
        menu.append(&quit_item)?;

        let icon = load_icon_idle();

        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Ripley")
            .with_icon(icon)
            .build()?;

        Ok(Self {
            _tray: tray,
            show_id,
            scan_id,
            monitor_id,
            quit_id,
        })
    }

    pub fn handle_event(&self, event: &MenuEvent) -> Option<TrayAction> {
        if *event.id() == self.show_id {
            Some(TrayAction::ShowDashboard)
        } else if *event.id() == self.scan_id {
            Some(TrayAction::ScanNow)
        } else if *event.id() == self.monitor_id {
            Some(TrayAction::ToggleMonitor)
        } else if *event.id() == self.quit_id {
            Some(TrayAction::Quit)
        } else {
            None
        }
    }
}

fn load_icon_idle() -> tray_icon::Icon {
    let rgba = create_shield_icon();
    tray_icon::Icon::from_rgba(rgba, 22, 22).expect("valid icon data")
}

fn create_shield_icon() -> Vec<u8> {
    let size = 22usize;
    let mut rgba = vec![0u8; size * size * 4];

    for y in 0..size {
        for x in 0..size {
            let cx = x as f64 - 11.0;
            let cy = y as f64 - 11.0;
            let in_shield = cy > -9.0
                && cy < 9.0
                && cx.abs() < (9.0 - (cy * 0.5).max(0.0))
                && (cx.abs() > (7.0 - (cy * 0.5).max(0.0)) || !(-7.0..=7.0).contains(&cy));
            if in_shield {
                let idx = (y * size + x) * 4;
                rgba[idx] = 0xe6;
                rgba[idx + 1] = 0xed;
                rgba[idx + 2] = 0xf3;
                rgba[idx + 3] = 0xff;
            }
        }
    }
    rgba
}
