use anti_llm_cheat_lsp::server::AntiLlmServer;
use clap::{Parser, Subcommand};
use tower_lsp::{LspService, Server};

#[derive(Parser)]
#[command(
    name = "anti-llm-cheat-lsp",
    about = "Admissibility LSP: detect LLM stubs and cheats"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the LSP server over stdio
    Serve {
        #[arg(long)]
        stdio: bool,
    },
    /// Scan a directory for LLM cheats and print findings
    Scan {
        #[arg(long, default_value = ".")]
        dir: String,
        #[arg(long, value_name = "DIR")]
        ignore_dirs: Vec<String>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Serve { stdio } => {
            if stdio {
                let stdin = tokio::io::stdin();
                let stdout = tokio::io::stdout();
                let (service, socket) = LspService::new(AntiLlmServer::new);
                Server::new(stdin, stdout, socket).serve(service).await;
            } else {
                eprintln!("Error: --stdio flag required for LSP serve mode");
                std::process::exit(1);
            }
        }
        Commands::Scan { dir, ignore_dirs } => {
            let _ = anti_llm_cheat_lsp::ocel::write_ocel_outputs(&dir);
            let mut obs = anti_llm_cheat_lsp::engine::scan_directory(&dir);
            if !ignore_dirs.is_empty() {
                obs.retain(|o| !ignore_dirs.iter().any(|d| o.file_path.contains(d.as_str())));
            }
            let diags = anti_llm_cheat_lsp::engine::evaluate_diagnostics(&obs);
            let mut diags = diags;
            diags.sort_by(|a, b| a.file_path.cmp(&b.file_path).then(a.line.cmp(&b.line)));
            println!("--- Anti-LLM Admissibility Scan ---");
            println!("Observations: {}", obs.len());
            println!("Diagnostics:  {}", diags.len());
            for d in &diags {
                println!("  [{}] {}:{}: {}", d.code, d.file_path, d.line, d.message);
            }
        }
    }

    Ok(())
}
