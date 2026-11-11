use std::collections::HashSet;
use std::path::{Path, PathBuf};

use matrix_sdk::ruma::{OwnedRoomId, RoomId};

#[derive(Debug)]
pub struct Config {
    pub room_whitelist: Whitelist,
    pub dir: PathBuf,
}

/// Rooms the server may send to, each under a label chosen by the user.
#[derive(Debug, Clone)]
pub struct Whitelist {
    rooms: Vec<(Option<String>, OwnedRoomId)>,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("MATRIX_ROOM_WHITELIST must list at least one room ID")]
    EmptyWhitelist,
    #[error("MATRIX_ROOM_WHITELIST accepts room IDs (!id:server) only, got {0}")]
    NotARoomId(String),
    #[error("MATRIX_ROOM_WHITELIST has the label {0} more than once")]
    DuplicateLabel(String),
    #[error("MATRIX_ROOM_WHITELIST has an empty label in {0}")]
    EmptyLabel(String),
}

impl Whitelist {
    /// Parses comma-separated `label=!id` or bare `!id` entries.
    pub fn parse(spec: &str) -> Result<Whitelist, ConfigError> {
        let rooms = spec
            .split(',')
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .map(parse_entry)
            .collect::<Result<Vec<_>, _>>()?;
        if rooms.is_empty() {
            return Err(ConfigError::EmptyWhitelist);
        }
        let mut seen = HashSet::new();
        for label in rooms.iter().filter_map(|(label, _)| label.as_deref()) {
            if !seen.insert(label.to_ascii_lowercase()) {
                return Err(ConfigError::DuplicateLabel(label.to_owned()));
            }
        }
        Ok(Whitelist { rooms })
    }

    /// Finds the room for a label (case ignored) or a room ID (exact).
    pub fn resolve(&self, room: &str) -> Option<&OwnedRoomId> {
        self.rooms
            .iter()
            .find(|(label, id)| {
                id.as_str() == room
                    || label
                        .as_deref()
                        .is_some_and(|label| label.eq_ignore_ascii_case(room))
            })
            .map(|(_, id)| id)
    }

    /// Each room as (name, ID) in config order; the name is the label, or
    /// the ID if the room has no label.
    pub fn entries(&self) -> impl Iterator<Item = (&str, &OwnedRoomId)> {
        self.rooms
            .iter()
            .map(|(label, id)| (label.as_deref().unwrap_or(id.as_str()), id))
    }

    pub fn names(&self) -> Vec<&str> {
        self.entries().map(|(name, _)| name).collect()
    }
}

fn parse_entry(entry: &str) -> Result<(Option<String>, OwnedRoomId), ConfigError> {
    let (label, id) = match entry.split_once('=') {
        Some((label, _)) if label.trim().is_empty() => {
            return Err(ConfigError::EmptyLabel(entry.to_owned()));
        }
        Some((label, id)) => (Some(label.trim().to_owned()), id.trim()),
        None => (None, entry),
    };
    let id = RoomId::parse(id).map_err(|_| ConfigError::NotARoomId(entry.to_owned()))?;
    Ok((label, id))
}

impl Config {
    pub fn from_lookup(
        lookup: impl Fn(&str) -> Option<String>,
        home: PathBuf,
    ) -> Result<Config, ConfigError> {
        Ok(Config {
            room_whitelist: Whitelist::parse(&lookup("MATRIX_ROOM_WHITELIST").unwrap_or_default())?,
            dir: dir(&home),
        })
    }
}

pub fn dir(home: &Path) -> PathBuf {
    home.join(".config/matrix-mcp")
}
