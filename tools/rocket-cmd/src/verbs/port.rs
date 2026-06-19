//! Port management commands

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde_json::Value;

fn do_wait_port(port: u16, host: String, timeout_secs: u64) -> Result<Value> {
    use std::net::TcpStream;
    use std::thread;
    use std::time::{Duration, Instant};
    let addr = format!("{}:{}", host, port);
    let deadline = Instant::now() + Duration::from_secs(timeout_secs);
    eprintln!(
        "Waiting for {}:{} (timeout {}s)...",
        host, port, timeout_secs
    );
    loop {
        if TcpStream::connect(&addr).is_ok() {
            println!("READY: {}", addr);
            return Ok(serde_json::json!({"status": "ready", "addr": addr}));
        }
        if Instant::now() >= deadline {
            return Err(clap_noun_verb::NounVerbError::execution_error(format!(
                "Timed out after {}s waiting for {}",
                timeout_secs, addr
            )));
        }
        thread::sleep(Duration::from_millis(500));
    }
}

/// Block until a TCP port accepts connections (server readiness gate)
///
/// # Arguments
/// * `port` - Port number to wait for
/// * `host` - Host to connect to
/// * `timeout` - Timeout in seconds before giving up
#[verb("wait", "port")]
fn wait_port(port: u16, host: Option<String>, timeout: Option<u64>) -> Result<Value> {
    do_wait_port(
        port,
        host.unwrap_or_else(|| "127.0.0.1".to_string()),
        timeout.unwrap_or(60),
    )
}
