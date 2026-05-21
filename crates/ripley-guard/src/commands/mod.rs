pub mod audit;
pub mod config;
pub mod exposure;
pub mod fix;
pub mod guard;
pub mod harden;
pub mod scan;
pub mod status;
pub mod watch;

use colored::{ColoredString, Colorize};
use ripley_core::audit::TrafficLight;

pub fn traffic_light_indicator(tl: &TrafficLight) -> &'static str {
    match tl {
        TrafficLight::Green => "●",
        TrafficLight::Yellow => "◐",
        TrafficLight::Red => "○",
    }
}

pub fn traffic_light_colored(text: &str, tl: &TrafficLight) -> ColoredString {
    match tl {
        TrafficLight::Green => text.green(),
        TrafficLight::Yellow => text.yellow(),
        TrafficLight::Red => text.red(),
    }
}
