pub mod credentials;
pub mod exposure;
pub mod ioc;
pub mod network;
pub mod persistence;

pub use credentials::{CredentialFinding, ExposureReport};
pub use exposure::{CveExposureReport, ExposureError};
pub use ioc::{IocFinding, IocProfile, IocProfileSet};
pub use network::{C2Database, C2Finding, NetworkConnection};
pub use persistence::{PersistenceCategory, PersistenceFinding};

pub(crate) fn glob_to_regex(pattern: &str) -> String {
    let mut regex = String::from("^");
    for ch in pattern.chars() {
        match ch {
            '*' => regex.push_str(".*"),
            '?' => regex.push('.'),
            '.' | '(' | ')' | '+' | '|' | '^' | '$' | '@' | '{' | '}' | '[' | ']' => {
                regex.push('\\');
                regex.push(ch);
            }
            _ => regex.push(ch),
        }
    }
    regex.push('$');
    regex
}
