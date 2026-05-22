use std::process::ExitCode;

use anyhow::Result;

pub async fn cmd_contain(_target: &str, _format: &str) -> Result<ExitCode> {
    anyhow::bail!("contain command not yet implemented (M16)")
}
