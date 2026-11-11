#![recursion_limit = "256"]

use std::os::unix::fs::DirBuilderExt;
use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand};
use matrix_mcp::config::{Config, dir};
use matrix_mcp::mcp::MatrixServer;
use matrix_mcp::session::{login, restore};
use matrix_mcp::verify::verify;
use matrix_sdk::config::SyncSettings;
use rmcp::ServiceExt;

/// Send-only MCP server for Matrix with end-to-end encryption.
#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Log in with your password and create the device `matrix-mcp`.
    Login {
        /// Matrix user, e.g. @you:matrix.org
        #[arg(long)]
        user: String,
        #[arg(long, default_value = "https://matrix-client.matrix.org")]
        homeserver: String,
    },
    /// Verify this device with emojis from another session, e.g. Element.
    Verify,
    /// Run the MCP server on stdio.
    Serve,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let home = PathBuf::from(std::env::var_os("HOME").context("HOME is not set")?);
    match Cli::parse().command {
        Command::Login { user, homeserver } => {
            let dir = dir(&home);
            std::fs::DirBuilder::new()
                .recursive(true)
                .mode(0o700)
                .create(&dir)?;
            let password = rpassword::prompt_password(format!("Password for {user}: "))?;
            login(&dir, &homeserver, &user, &password).await?;
            eprintln!("Logged in. Now run `matrix-mcp verify`.");
        }
        Command::Verify => verify(restore(&dir(&home)).await?).await?,
        Command::Serve => serve(home).await?,
    }
    Ok(())
}

async fn serve(home: PathBuf) -> anyhow::Result<()> {
    let config = Config::from_lookup(|key| std::env::var(key).ok(), home)?;
    let client = restore(&config.dir).await?;
    client.sync_once(SyncSettings::default()).await?;
    let sync_client = client.clone();
    tokio::spawn(async move {
        if let Err(e) = sync_client.sync(SyncSettings::default()).await {
            eprintln!("matrix-mcp: sync stopped: {e}");
        }
    });
    MatrixServer::new(client, config.room_whitelist)
        .serve(rmcp::transport::stdio())
        .await?
        .waiting()
        .await?;
    Ok(())
}
