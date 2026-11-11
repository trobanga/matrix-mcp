use matrix_mcp::config::Whitelist;

use matrix_mcp::send::send_message;
use matrix_sdk::ruma::{event_id, room_id};
use matrix_sdk::test_utils::mocks::MatrixMockServer;

#[tokio::test]
async fn sends_text_to_whitelisted_room_and_returns_event_id() {
    let server = MatrixMockServer::new().await;
    let client = server.client_builder().build().await;
    server.mock_room_state_encryption().plain().mount().await;
    server
        .sync_joined_room(&client, room_id!("!room:localhost"))
        .await;
    server
        .mock_room_send()
        .ok(event_id!("$ev1"))
        .expect(1)
        .mount()
        .await;
    let whitelist = Whitelist::parse("!room:localhost").unwrap();

    let event_id = send_message(&client, &whitelist, "!room:localhost", "hello")
        .await
        .unwrap();

    assert_eq!(event_id, "$ev1");
}

#[tokio::test]
async fn sends_to_room_given_by_label() {
    let server = MatrixMockServer::new().await;
    let client = server.client_builder().build().await;
    server.mock_room_state_encryption().plain().mount().await;
    server
        .sync_joined_room(&client, room_id!("!room:localhost"))
        .await;
    server
        .mock_room_send()
        .ok(event_id!("$ev1"))
        .expect(1)
        .mount()
        .await;
    let whitelist = Whitelist::parse("self=!room:localhost").unwrap();

    let event_id = send_message(&client, &whitelist, "Self", "hello")
        .await
        .unwrap();

    assert_eq!(event_id, "$ev1");
}

#[tokio::test]
async fn reports_whitelisted_room_the_account_has_not_joined() {
    let server = MatrixMockServer::new().await;
    let client = server.client_builder().build().await;
    let whitelist = Whitelist::parse("!room:localhost").unwrap();

    let err = send_message(&client, &whitelist, "!room:localhost", "hello")
        .await
        .unwrap_err();

    assert!(err.to_string().contains("not a member"), "{err}");
}

#[tokio::test]
async fn reports_homeserver_error() {
    let server = MatrixMockServer::new().await;
    let client = server.client_builder().build().await;
    server.mock_room_state_encryption().plain().mount().await;
    server
        .sync_joined_room(&client, room_id!("!room:localhost"))
        .await;
    server.mock_room_send().error500().mount().await;
    let whitelist = Whitelist::parse("!room:localhost").unwrap();

    let err = send_message(&client, &whitelist, "!room:localhost", "hello")
        .await
        .unwrap_err();

    assert!(err.to_string().starts_with("sending failed"), "{err}");
}

#[tokio::test]
async fn refuses_room_not_on_whitelist_without_sending() {
    let server = MatrixMockServer::new().await;
    let client = server.client_builder().build().await;
    server.mock_room_state_encryption().plain().mount().await;
    server
        .sync_joined_room(&client, room_id!("!other:localhost"))
        .await;
    server.mock_room_send().ok(event_id!("$ev1")).expect(0).mount().await;
    let whitelist = Whitelist::parse("!room:localhost").unwrap();

    let err = send_message(&client, &whitelist, "!other:localhost", "hello")
        .await
        .unwrap_err();

    assert!(err.to_string().contains("not on the whitelist"), "{err}");
}

#[tokio::test]
async fn refusal_lists_allowed_rooms() {
    let server = MatrixMockServer::new().await;
    let client = server.client_builder().build().await;
    let whitelist = Whitelist::parse("self=!room:localhost,!bare:localhost").unwrap();

    let err = send_message(&client, &whitelist, "team", "hello")
        .await
        .unwrap_err();

    assert!(
        err.to_string()
            .ends_with("allowed rooms: self, !bare:localhost"),
        "{err}"
    );
}
