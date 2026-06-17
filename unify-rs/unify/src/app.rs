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
    /// Genie 26 World Manufacturing Platform subcommands
    Genie {
        #[command(subcommand)]
        subcommand: GenieSubcommands,
    },
    /// Parse natural language intent into a WorldSpec JSON
    WorldParse {
        #[arg(short, long)]
        intent: String,
        #[arg(short, long)]
        output: String,
    },
    /// Validate a WorldSpec JSON against coherence rules
    WorldValidate {
        #[arg(short, long)]
        spec: String,
    },
    /// Generate a UE4 T3D level map from a WorldSpec JSON
    WorldGenerate {
        #[arg(short, long)]
        spec: String,
        #[arg(short, long)]
        output: String,
    },
    /// Deploy a manufactured world spec and start visualizer dashboard
    WorldDeploy {
        #[arg(short, long)]
        spec: String,
        #[arg(short, long)]
        log: String,
    },
    /// Evolve an existing world spec with modification intent
    WorldEvolve {
        #[arg(short, long)]
        spec: String,
        #[arg(short, long)]
        intent: String,
        #[arg(short, long)]
        output: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum GenieSubcommands {
    /// Manufacture a new world from natural language intent
    Manufacture {
        /// Path to the natural language intent file or raw text
        #[arg(short, long)]
        intent: String,
        /// Path to save the generated WorldSpec JSON file
        #[arg(long)]
        out_spec: String,
        /// Path to save the compiled UE4 T3D level map file
        #[arg(long)]
        out_t3d: String,
    },
    /// Evolve an existing world spec using new modification intent
    Evolve {
        /// Path to the existing WorldSpec JSON file
        #[arg(short, long)]
        spec: String,
        /// Path to the modification intent file or raw text
        #[arg(short, long)]
        intent: String,
        /// Path to save the evolved WorldSpec JSON file
        #[arg(long)]
        out_spec: String,
        /// Path to save the evolved UE4 T3D level map file
        #[arg(long)]
        out_t3d: String,
    },
    /// Deploy the manufactured world, registering telemetry log entry
    Deploy {
        /// Path to the WorldSpec JSON file
        #[arg(short, long)]
        spec: String,
        /// Path to write the deployment log file
        #[arg(short, long)]
        log: String,
    },
}
