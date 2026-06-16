use clap::Parser;
use unify::app::Cli;
use unify::commands::run;

fn main() {
    let cli = Cli::parse();
    let json = cli.json;
    match run(cli) {
        Ok(output) => {
            if json {
                println!("{}", output.to_json());
            } else {
                println!("{}", output.to_human());
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
