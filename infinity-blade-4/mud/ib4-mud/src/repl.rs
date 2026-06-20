use crate::{command::Command, session::GameSession};
use std::io::{self, BufRead, Write};

pub fn run_repl(session: &mut GameSession) -> anyhow::Result<()> {
    // Premium gold/yellow for the welcome banner
    tracing::info!(target: "game", "\x1b[1;33m\u{2554}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2557}\x1b[0m");
    tracing::info!(target: "game", "\x1b[1;33m\u{2551}     INFINITY BLADE IV \u{2014} MUD TEXT EDITION      \u{2551}\x1b[0m");
    tracing::info!(target: "game", "\x1b[1;33m\u{2551}  'help' for commands | 'explore' to begin     \u{2551}\x1b[0m");
    tracing::info!(target: "game", "\x1b[1;33m\u{255a}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{255d}\x1b[0m");
    tracing::info!(target: "game", "");

    // Bold cyan and green for status
    tracing::info!(
        target: "game",
        "Welcome, \x1b[1;32m{}\x1b[0m. Bloodline: \x1b[1;36m{}\x1b[0m | Level: \x1b[1;36m{}\x1b[0m",
        session.player.name,
        session.player.bloodline_label(),
        session.player.level
    );
    tracing::info!(target: "game", "\x1b[3mThe ancient arena awaits. Type 'look' to survey your surroundings.\x1b[0m");
    tracing::info!(target: "game", "");

    let stdin = io::stdin();
    let stdout = io::stdout();

    loop {
        {
            let mut out = stdout.lock();
            write!(out, "\x1b[1;32m> \x1b[0m")?;
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
                tracing::info!(
                    target: "game",
                    "\x1b[1;33mFarewell, {}. The bloodline remembers.\x1b[0m",
                    session.player.name
                );
                break;
            }
            Ok(cmd) => {
                let output = session.dispatch(cmd);
                for line in output {
                    // Check if it is a system log or gameplay log
                    if line.starts_with("[SYSTEM]") {
                        tracing::info!(target: "game", "\x1b[90m{}\x1b[0m", line);
                    } else if line.contains("DEFEATED") || line.contains("slain") {
                        tracing::info!(target: "game", "\x1b[1;31m{}\x1b[0m", line);
                    } else if line.contains("LEVEL UP") {
                        tracing::info!(target: "game", "\x1b[1;32;5m{}\x1b[0m", line);
                    } else {
                        tracing::info!(target: "game", "{}", line);
                    }
                }
                tracing::info!(target: "game", "");
            }
            Err(msg) => {
                // Styled error output
                tracing::info!(target: "game", "\x1b[1;31m[?] {}\x1b[0m", msg);
                tracing::info!(target: "game", "");
            }
        }
    }

    Ok(())
}
