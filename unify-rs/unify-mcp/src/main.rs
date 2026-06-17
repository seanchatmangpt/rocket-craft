use unify_mcp::anti_llm_tools;
use unify_mcp::rocket_tools;
use unify_mcp::server::{McpServer, ServerInfo};
use unify_mcp::tools;

fn main() {
    let server = McpServer::new(ServerInfo {
        name: "unify-mcp".into(),
        version: env!("CARGO_PKG_VERSION").into(),
    });
    let server = tools::register_server_tools(server);
    let server = rocket_tools::attach_rocket_tools(server);
    let server = anti_llm_tools::attach_anti_llm_tools(server);

    // Read JSON-RPC requests from stdin, one per line; write responses to stdout
    use std::io::BufRead;
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.trim().is_empty() {
            continue;
        }
        let response = server.handle(&line);
        println!("{}", response);
    }
}
