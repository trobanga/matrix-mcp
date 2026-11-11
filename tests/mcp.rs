use std::time::Duration;

use matrix_mcp::config::Whitelist;
use matrix_mcp::mcp::MatrixServer;
use matrix_sdk::ruma::{event_id, room_id};
use matrix_sdk::test_utils::mocks::MatrixMockServer;
use rmcp::ServiceExt;
use rmcp::model::{CallToolRequestParams, CallToolResult};
use serde_json::{Value, json};

const WHITELIST: &str = "self=!room:localhost,!bare:localhost";

async fn call(tool: &'static str, arguments: Value) -> CallToolResult {
    let server = MatrixMockServer::new().await;
    let client = server.client_builder().build().await;
    server.mock_room_state_encryption().plain().mount().await;
    server
        .sync_joined_room(&client, room_id!("!room:localhost"))
        .await;
    server.mock_room_send().ok(event_id!("$ev1")).mount().await;
    let whitelist = Whitelist::parse(WHITELIST).unwrap();

    let (server_io, client_io) = tokio::io::duplex(64 * 1024);
    tokio::spawn(async move {
        let running = MatrixServer::new(client, whitelist)
            .serve(server_io)
            .await
            .unwrap();
        running.waiting().await.unwrap();
    });
    let mcp = ().serve(client_io).await.unwrap();
    let request =
        CallToolRequestParams::new(tool).with_arguments(arguments.as_object().unwrap().clone());
    let result = tokio::time::timeout(Duration::from_secs(10), mcp.call_tool(request))
        .await
        .expect("tool call timed out")
        .unwrap();
    mcp.cancel().await.unwrap();
    result
}

async fn send_to(room: &str) -> CallToolResult {
    call("send_message", json!({"room": room, "text": "hello"})).await
}

fn text(result: &CallToolResult) -> String {
    result.content[0].as_text().unwrap().text.clone()
}

#[tokio::test]
async fn send_message_tool_returns_event_id() {
    let result = send_to("self").await;

    assert_ne!(result.is_error, Some(true));
    assert_eq!(text(&result), "$ev1");
}

#[tokio::test]
async fn send_message_tool_reports_refusal_as_tool_error() {
    let result = send_to("!other:localhost").await;

    assert_eq!(result.is_error, Some(true));
    assert!(text(&result).contains("not on the whitelist"));
}

#[tokio::test]
async fn list_rooms_tool_returns_whitelist() {
    let result = call("list_rooms", json!({})).await;

    assert_ne!(result.is_error, Some(true));
    assert_eq!(
        text(&result),
        "self = !room:localhost\n!bare:localhost = !bare:localhost"
    );
}
