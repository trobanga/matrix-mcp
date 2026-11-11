use std::os::unix::fs::PermissionsExt;

use matrix_mcp::session::{login, restore};
use matrix_sdk::test_utils::mocks::MatrixMockServer;

#[tokio::test]
async fn login_saves_private_session_that_restore_loads() {
    let server = MatrixMockServer::new().await;
    server.mock_versions().ok().mount().await;
    server.mock_login().ok().mount().await;
    let dir = tempfile::tempdir().unwrap();

    login(dir.path(), &server.uri(), "cheeky_monkey", "pw")
        .await
        .unwrap();
    let client = restore(dir.path()).await.unwrap();

    assert_eq!(client.user_id().unwrap(), "@cheeky_monkey:matrix.org");
    assert_eq!(client.device_id().unwrap(), "GHTYAJCE");
    assert_eq!(client.homeserver().as_str(), format!("{}/", server.uri()));
    let mode = std::fs::metadata(dir.path().join("session.json"))
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o600);
}

#[tokio::test]
async fn login_refuses_to_replace_existing_session_without_contacting_server() {
    let server = MatrixMockServer::new().await;
    server.mock_login().ok().expect(0).mount().await;
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("session.json"), "{}").unwrap();

    let err = login(dir.path(), &server.uri(), "cheeky_monkey", "pw")
        .await
        .unwrap_err();

    assert!(err.to_string().contains("already"), "{err}");
}

#[tokio::test]
async fn missing_session_tells_user_to_log_in() {
    let dir = tempfile::tempdir().unwrap();

    let err = restore(dir.path()).await.unwrap_err();

    assert!(err.to_string().contains("matrix-mcp login"), "{err}");
}
