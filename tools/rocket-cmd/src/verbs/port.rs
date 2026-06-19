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

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    fn bind_ephemeral() -> (TcpListener, u16) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
        let port = listener.local_addr().unwrap().port();
        (listener, port)
    }

    #[test]
    fn succeeds_immediately_when_port_is_open() {
        let (_listener, port) = bind_ephemeral();
        let result = do_wait_port(port, "127.0.0.1".into(), 5);
        assert!(result.is_ok(), "should connect immediately to open port");
        let val = result.unwrap();
        assert_eq!(val["status"], "ready");
    }

    #[test]
    fn returns_ready_addr_in_json() {
        let (_listener, port) = bind_ephemeral();
        let val = do_wait_port(port, "127.0.0.1".into(), 5).unwrap();
        let addr = val["addr"].as_str().expect("addr field must be string");
        assert!(addr.contains(&port.to_string()), "addr must contain port");
        assert!(addr.starts_with("127.0.0.1"), "addr must start with host");
    }

    #[test]
    fn times_out_when_port_is_closed() {
        // Port 1 is reserved; connecting should be refused immediately on most systems.
        // Use a port that will definitely not have a listener.
        let result = do_wait_port(1, "127.0.0.1".into(), 1);
        assert!(result.is_err(), "should time out for an unbound port");
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("Timed out") || msg.contains("timed out") || msg.contains("1s"),
            "error message must mention timeout: {msg}");
    }

    #[test]
    fn timeout_zero_returns_error_immediately() {
        let result = do_wait_port(1, "127.0.0.1".into(), 0);
        assert!(result.is_err(), "zero timeout should fail immediately");
    }
}
