use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

use matrix_sdk::Client;
use matrix_sdk::authentication::matrix::MatrixSession;
use matrix_sdk::store::RoomLoadSettings;
use serde::{Deserialize, Serialize};

const SESSION_FILE: &str = "session.json";
const DEVICE_NAME: &str = "matrix-mcp";

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("no session in {0}; run `matrix-mcp login` first")]
    Missing(String),
    #[error("a session already exists in {0}; remove it to log in again")]
    Exists(String),
    #[error("cannot build client: {0}")]
    Build(#[from] matrix_sdk::ClientBuildError),
    #[error("matrix error: {0}")]
    Matrix(#[from] matrix_sdk::Error),
    #[error("session file: {0}")]
    Io(#[from] std::io::Error),
    #[error("session file is invalid: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Serialize, Deserialize)]
struct StoredSession {
    homeserver: String,
    session: MatrixSession,
}

pub async fn login(
    dir: &Path,
    homeserver: &str,
    user: &str,
    password: &str,
) -> Result<(), SessionError> {
    if dir.join(SESSION_FILE).exists() {
        return Err(SessionError::Exists(dir.display().to_string()));
    }
    let client = build_client(dir, homeserver).await?;
    client
        .matrix_auth()
        .login_username(user, password)
        .initial_device_display_name(DEVICE_NAME)
        .await?;
    let stored = StoredSession {
        homeserver: client.homeserver().to_string(),
        session: client.matrix_auth().session().expect("logged in"),
    };
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(dir.join(SESSION_FILE))?;
    file.write_all(&serde_json::to_vec(&stored)?)?;
    Ok(())
}

pub async fn restore(dir: &Path) -> Result<Client, SessionError> {
    let file = dir.join(SESSION_FILE);
    if !file.exists() {
        return Err(SessionError::Missing(dir.display().to_string()));
    }
    let stored: StoredSession = serde_json::from_slice(&std::fs::read(file)?)?;
    let client = build_client(dir, &stored.homeserver).await?;
    client
        .matrix_auth()
        .restore_session(stored.session, RoomLoadSettings::default())
        .await?;
    Ok(client)
}

async fn build_client(dir: &Path, homeserver: &str) -> Result<Client, SessionError> {
    Ok(Client::builder()
        .homeserver_url(homeserver)
        .sqlite_store(dir.join("store"), None)
        .build()
        .await?)
}
