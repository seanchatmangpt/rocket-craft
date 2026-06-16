use anyhow::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;
use colored::*;

#[derive(Serialize)]
pub struct EmptyResponse {}

/// Optimize PWA assets and generate manifest
#[verb("optimize")]
pub fn cmd_pwa(
    /// Directory containing PWA assets [default: pwa-staff]
    #[arg(short, long, default_value = "pwa-staff")]
    dir: String,
    /// Output minified worker to a different file
    #[arg(short, long)]
    output: Option<String>,
) -> clap_noun_verb::Result<EmptyResponse> {
    crate::run_pwa(&dir, output).map_err(|e| clap_noun_verb::Error::from(e.to_string()))?;
    Ok(EmptyResponse {})
}
