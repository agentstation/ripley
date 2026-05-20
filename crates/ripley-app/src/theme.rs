use iced::Color;

pub struct Colors;

impl Colors {
    pub const BG: Color = Color::from_rgb(
        0x0d as f32 / 255.0,
        0x11 as f32 / 255.0,
        0x17 as f32 / 255.0,
    );
    pub const SURFACE: Color = Color::from_rgb(
        0x16 as f32 / 255.0,
        0x1b as f32 / 255.0,
        0x22 as f32 / 255.0,
    );
    pub const SURFACE_HOVER: Color = Color::from_rgb(
        0x1c as f32 / 255.0,
        0x21 as f32 / 255.0,
        0x28 as f32 / 255.0,
    );
    pub const SURFACE_ACTIVE: Color = Color::from_rgb(
        0x28 as f32 / 255.0,
        0x2e as f32 / 255.0,
        0x36 as f32 / 255.0,
    );
    pub const BORDER: Color = Color::from_rgb(
        0x30 as f32 / 255.0,
        0x36 as f32 / 255.0,
        0x3d as f32 / 255.0,
    );
    pub const BORDER_SUBTLE: Color = Color::from_rgb(
        0x21 as f32 / 255.0,
        0x26 as f32 / 255.0,
        0x2d as f32 / 255.0,
    );

    pub const TEXT_PRIMARY: Color = Color::from_rgb(
        0xe6 as f32 / 255.0,
        0xed as f32 / 255.0,
        0xf3 as f32 / 255.0,
    );
    pub const TEXT_SECONDARY: Color = Color::from_rgb(
        0x8b as f32 / 255.0,
        0x94 as f32 / 255.0,
        0x9e as f32 / 255.0,
    );
    pub const TEXT_MUTED: Color = Color::from_rgb(
        0x48 as f32 / 255.0,
        0x4f as f32 / 255.0,
        0x58 as f32 / 255.0,
    );

    pub const ACCENT: Color = Color::from_rgb(
        0x58 as f32 / 255.0,
        0xa6 as f32 / 255.0,
        0xff as f32 / 255.0,
    );
    pub const ACCENT_HOVER: Color = Color::from_rgb(
        0x79 as f32 / 255.0,
        0xc0 as f32 / 255.0,
        0xff as f32 / 255.0,
    );

    pub const SEVERITY_CRITICAL: Color = Color::from_rgb(
        0xf8 as f32 / 255.0,
        0x51 as f32 / 255.0,
        0x49 as f32 / 255.0,
    );
    pub const SEVERITY_HIGH: Color = Color::from_rgb(
        0xdb as f32 / 255.0,
        0x6d as f32 / 255.0,
        0x28 as f32 / 255.0,
    );
    pub const SEVERITY_MEDIUM: Color = Color::from_rgb(
        0xd2 as f32 / 255.0,
        0x99 as f32 / 255.0,
        0x22 as f32 / 255.0,
    );
    pub const SEVERITY_LOW: Color = Color::from_rgb(
        0x58 as f32 / 255.0,
        0xa6 as f32 / 255.0,
        0xff as f32 / 255.0,
    );
    pub const SEVERITY_CLEAN: Color = Color::from_rgb(
        0x3f as f32 / 255.0,
        0xb9 as f32 / 255.0,
        0x50 as f32 / 255.0,
    );
}

pub fn severity_color(severity: &ripley_core::types::Severity) -> Color {
    match severity {
        ripley_core::types::Severity::Critical => Colors::SEVERITY_CRITICAL,
        ripley_core::types::Severity::High => Colors::SEVERITY_HIGH,
        ripley_core::types::Severity::Medium => Colors::SEVERITY_MEDIUM,
        ripley_core::types::Severity::Low => Colors::SEVERITY_LOW,
    }
}
