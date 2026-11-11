use matrix_sdk::Client;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{ServerCapabilities, ServerConfig};
use rmcp::{ServerHandler, tool, tool_handler, tool_router};

use crate::config::Whitelist;
use crate::send::send_message;

#[derive(Clone)]
pub struct MatrixServer {
    client: Client,
    whitelist: Whitelist,
    #[expect(dead_code, reason = "read by the tool_handler macro")]
    tool_router: ToolRouter<Self>,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct SendArgs {
    /// Room label from list_rooms, or a whitelisted room ID
    room: String,
    /// Plain-text message body
    text: String,
}

impl MatrixServer {
    pub fn new(client: Client, whitelist: Whitelist) -> Self {
        Self {
            client,
            whitelist,
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router]
impl MatrixServer {
    #[tool(description = "Send a plain-text message to a whitelisted Matrix room, \
                          given by its label or room ID (see list_rooms). \
                          Returns the event ID.")]
    async fn send_message(
        &self,
        Parameters(SendArgs { room, text }): Parameters<SendArgs>,
    ) -> Result<String, String> {
        send_message(&self.client, &self.whitelist, &room, &text)
            .await
            .map(|event_id| event_id.to_string())
            .map_err(|e| e.to_string())
    }

    #[tool(description = "List the rooms send_message may send to, one per line \
                          as `label = room ID`.")]
    async fn list_rooms(&self) -> String {
        self.whitelist
            .entries()
            .map(|(name, id)| format!("{name} = {id}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[tool_handler]
impl ServerHandler for MatrixServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
    }
}
