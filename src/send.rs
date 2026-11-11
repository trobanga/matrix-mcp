use matrix_sdk::ruma::OwnedEventId;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use matrix_sdk::{Client, RoomState};

use crate::config::Whitelist;

#[derive(Debug, thiserror::Error)]
pub enum SendError {
    #[error("room {room} is not on the whitelist; allowed rooms: {allowed}")]
    NotWhitelisted { room: String, allowed: String },
    #[error("the account is not a member of room {0}")]
    NotJoined(String),
    #[error("sending failed: {0}")]
    Failed(matrix_sdk::Error),
}

pub async fn send_message(
    client: &Client,
    whitelist: &Whitelist,
    room: &str,
    text: &str,
) -> Result<OwnedEventId, SendError> {
    let room_id = whitelist
        .resolve(room)
        .ok_or_else(|| SendError::NotWhitelisted {
            room: room.to_owned(),
            allowed: whitelist.names().join(", "),
        })?;
    let room = client
        .get_room(room_id)
        .filter(|room| room.state() == RoomState::Joined)
        .ok_or_else(|| SendError::NotJoined(room_id.to_string()))?;
    let result = room
        .send(RoomMessageEventContent::text_plain(text))
        .await
        .map_err(SendError::Failed)?;
    Ok(result.response.event_id)
}
