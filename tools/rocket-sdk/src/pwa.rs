use anyhow::Result;
use colored::*;

/// Optimize PWA assets and generate manifest
pub fn cmd_pwa(
    dir: String,
    output: Option<String>,
) -> anyhow::Result<()> {
    crate::run_pwa(&dir, output)?;
    Ok(())
}
