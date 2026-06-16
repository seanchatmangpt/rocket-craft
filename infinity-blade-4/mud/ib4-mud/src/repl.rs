use std::io::{self, BufRead, Write};
use crate::{command::Command, session::GameSession};

pub fn run_repl(session: &mut GameSession) -> anyhow::Result<()> {
    println!("\u{2554}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2557}");
    println!("\u{2551}     INFINITY BLADE IV \u{2014} MUD TEXT EDITION      \u{2551}");
    println!("\u{2551}  'help' for commands | 'explore' to begin     \u{2551}");
    println!("\u{255a}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{255d}");
    println!();
    println!(
        "Welcome, {}. Bloodline: {} | Level: {}",
        session.player.name,
        session.player.bloodline_label(),
        session.player.level
    );
    println!("The ancient arena awaits. Type 'look' to survey your surroundings.");
    println!();

    let stdin = io::stdin();
    let stdout = io::stdout();

    loop {
        {
            let mut out = stdout.lock();
            write!(out, "> ")?;
            out.flush()?;
        }

        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => break, // EOF
            Err(_) => break,
            Ok(_) => {}
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        match Command::parse(trimmed) {
            Ok(Command::Quit) => {
                println!(
                    "Farewell, {}. The bloodline remembers.",
                    session.player.name
                );
                break;
            }
            Ok(cmd) => {
                let output = session.dispatch(cmd);
                for line in output {
                    println!("{}", line);
                }
                println!();
            }
            Err(msg) => {
                println!("[?] {}", msg);
                println!();
            }
        }
    }

    Ok(())
}
