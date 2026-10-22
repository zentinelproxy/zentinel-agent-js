//! Zentinel JavaScript Agent CLI
//!
//! Command-line interface for the JavaScript scripting agent.
//!
//! Serves agent protocol v2 over a Unix socket (`--socket`, the default) or
//! gRPC (`--grpc-address`).

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use tracing::info;

use zentinel_agent_js::JsAgent;
use zentinel_agent_protocol::v2::{GrpcAgentServerV2, UdsAgentServerV2};

/// Command line arguments
#[derive(Parser, Debug)]
#[command(name = "zentinel-js-agent")]
#[command(about = "JavaScript scripting agent for Zentinel reverse proxy (v2 protocol)")]
#[command(version)]
struct Args {
    /// Path to JavaScript script file
    #[arg(long, env = "JS_SCRIPT")]
    script: PathBuf,

    /// Unix socket to listen on (agent protocol v2 over UDS). Used unless
    /// `--grpc-address` is given.
    #[arg(long, default_value = "/tmp/zentinel-js.sock", env = "AGENT_SOCKET")]
    socket: PathBuf,

    /// gRPC address to listen on (e.g. "127.0.0.1:50052"). When given, the
    /// agent serves agent protocol v2 over gRPC instead of the Unix socket.
    #[arg(long, env = "GRPC_ADDRESS")]
    grpc_address: Option<String>,

    /// Enable verbose logging
    #[arg(short, long, env = "JS_VERBOSE")]
    verbose: bool,

    /// Fail open on script errors (allow requests instead of blocking)
    #[arg(long, env = "FAIL_OPEN")]
    fail_open: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize tracing
    let log_level = if args.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(format!(
            "{}={},zentinel_agent_protocol=info",
            env!("CARGO_CRATE_NAME"),
            log_level
        ))
        .json()
        .init();

    info!("Starting Zentinel JavaScript Agent (v2 protocol)");

    // Create agent
    let agent = JsAgent::new(args.script.clone(), args.fail_open)?;

    info!(
        script = ?args.script,
        fail_open = args.fail_open,
        "Agent configured"
    );

    if let Some(grpc_addr) = args.grpc_address {
        let addr = grpc_addr
            .parse()
            .context("Invalid gRPC address format (expected host:port)")?;
        info!(grpc_address = %grpc_addr, "Listening on gRPC (agent protocol v2)");
        GrpcAgentServerV2::new("zentinel-js-agent", Box::new(agent))
            .run(addr)
            .await
            .context("Failed to run JavaScript agent gRPC server")?;
    } else {
        info!(socket = %args.socket.display(), "Listening on Unix socket (agent protocol v2)");
        UdsAgentServerV2::new("zentinel-js-agent", args.socket, Box::new(agent))
            .run()
            .await
            .context("Failed to run JavaScript agent UDS server")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_names_the_binary() {
        let err = Args::try_parse_from(["x", "--version"]).unwrap_err();
        assert_eq!(
            err.to_string().trim(),
            format!("zentinel-js-agent {}", env!("CARGO_PKG_VERSION"))
        );
    }

    #[test]
    fn defaults_to_the_unix_socket() {
        let args = Args::try_parse_from(["x", "--script", "a.js"]).unwrap();
        assert!(args.grpc_address.is_none());
        assert_eq!(args.socket, PathBuf::from("/tmp/zentinel-js.sock"));
        let args = Args::try_parse_from(["x", "--script", "a.js", "--grpc-address", "127.0.0.1:1"])
            .unwrap();
        assert_eq!(args.grpc_address.as_deref(), Some("127.0.0.1:1"));
    }
}
