use clap::Parser;
use lsp_max::{LspService, Server};

mod server;
mod diagnostics;
mod code_actions;
mod ocel;

#[derive(Parser, Debug)]
#[command(name = "ggen-asset-lsp")]
#[command(about = "Language Server Protocol for ggen assets", long_about = None)]
struct Cli {
    #[arg(long, help = "Start the LSP server over stdio")]
    stdio: bool,
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if cli.stdio {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();
        let (service, socket) = LspService::new(server::GgenAssetLspServer::new);
        let _ = Server::new(stdin, stdout, socket).serve(service).await;
    } else {
        eprintln!("Error: --stdio flag is required to run the LSP server");
        std::process::exit(1);
    }

    Ok(())
}
