use rmcp::ServiceExt;

use tauri_plugin_instruckt::mcp::server::InstrucktMcpServer;
use tauri_plugin_instruckt::store::Store;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // All logging must go to stderr — stdout is the MCP JSON-RPC transport
    eprintln!("[instruckt-mcp] Starting MCP server...");

    let data_dir = Store::default_data_dir()?;
    eprintln!("[instruckt-mcp] Data directory: {}", data_dir.display());

    let store = Store::new(data_dir)?;
    let server = InstrucktMcpServer::new(store);

    let service = server.serve(rmcp::transport::io::stdio()).await?;

    eprintln!("[instruckt-mcp] Server running, waiting for requests...");
    service.waiting().await?;

    eprintln!("[instruckt-mcp] Server shutting down");
    Ok(())
}
