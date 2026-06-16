use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "unify",
    version,
    about = "Unified abstraction layer for ggen, clap-noun-verb, lsp-max, chicago-tdd-tools, unrdf, un-test-utils, wasm4pm-compat"
)]
pub struct Cli {
    /// Output results as JSON
    #[arg(long, global = true)]
    pub json: bool,

    /// Suppress non-essential output
    #[arg(long, global = true)]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Compute a BLAKE3 receipt for the given data
    Receipt {
        #[arg(short, long)]
        label: String,
        data: String,
    },
    /// Verify a receipt chain stored as JSON
    Verify {
        chain_json: String,
    },
    /// Check an admission gate law
    Gate {
        #[arg(short, long)]
        law: String,
        #[arg(short, long)]
        data: String,
    },
    /// Show version info for all unify-rs crates
    Info,
    /// Dispatch a noun-verb command as JSON
    Dispatch {
        #[arg(short, long)]
        namespace: Option<String>,
        noun: String,
        verb: String,
        #[arg(short, long)]
        input: Option<String>,
    },
    /// Run a SPARQL-like triple pattern query
    Query {
        /// Turtle input
        #[arg(short, long)]
        ttl: Option<String>,
        pattern: String,
    },
    /// Show the witness registry
    Witnesses {
        #[arg(short, long)]
        domain: Option<String>,
    },
}
