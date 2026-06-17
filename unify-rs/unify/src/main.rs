use clap::Parser;
use unify::app::Cli;
use unify::commands::run;

fn main() {
    let cli = Cli::parse();
    let json = cli.json;
    match run(cli) {
        Ok(output) => {
            if json {
                use std::io::Write;
                let _ = writeln!(std::io::stdout(), "{}", output.to_json());
            } else {
                use std::io::Write;
                let _ = writeln!(std::io::stdout(), "{}", output.to_human());
            }
        }
        Err(e) => {
            use std::io::Write;
            let _ = writeln!(std::io::stderr(), "Error: {}", e);
            std::process::exit(1);
        }
    }
}
