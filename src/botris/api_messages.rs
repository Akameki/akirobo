use super::types::*;
use serde::{Deserialize, Serialize};
use serde_json::Number;


#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum BotrisMsg {
    // Join Room
    RoomData(RoomDataPayload),
    Authenticated(SessionIdPayload),

    // WS Messages
    Error(String),
    PlayerJoined(PlayerDataPayload),
    PlayerLeft(SessionIdPayload),
    PlayerBanned(BotInfoPayload),
    PlayerUnbanned(BotInfoPayload),
    SettingsChanged(RoomDataPayload),

    // HostChanged(BotInfoPayload),

    // Ingame
    GameStarted,
    RoundStarted(RoundStartPayload),
    RequestMove(RequestMovePayload),
    Action(ActionPayload),
    PlayerAction(PlayerActionPayload),
    PlayerDamageReceived(PlayerDamageReceivedPayload),
    RoundOver(EndPayload),
    GameOver(EndPayload),
    GameReset(RoomDataPayload),
    // Ping(PingPayload),
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomDataPayload {
    pub room_data: RoomData,
}

pub type ErrorType = String;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerDataPayload {
    pub player_data: PlayerData,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionIdPayload {
    pub session_id: SessionId,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BotInfoPayload {
    pub bot_info: BotInfo,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoundStartPayload {
    pub starts_at: Number,
    pub room_data: RoomData,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestMovePayload {
    pub game_state: GameState,
    pub players: Vec<PlayerData>
}

/// Represents a collection of moves to be sent to the server for the
/// current piece
// pub type ActionPayload = Vec<Command>;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionPayload {
    pub commands: Vec<Command>
}

impl ActionPayload {
    pub fn empty() -> ActionPayload {
        ActionPayload { commands: Vec::new() }
    }
    pub fn new(commands: Vec<Command>) -> ActionPayload {
        ActionPayload { commands }
    }

    /// Adds a new command to the end of the vector
    /// containing the requested Commands
    pub fn push(&mut self, command: Command) -> &mut Self {
        self.commands.push(command);
        self
    }

    /// Appends all of the Commands in commands to the 
    /// end of the vector containing the requested Commands
    pub fn append(&mut self, commands: &mut Vec<Command>) -> &mut Self {
        self.commands.append(commands);
        self
    }
}


#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerActionPayload {
    // pub session_id: SessionId,
    pub commands: Vec<Command>,
    pub game_state: GameState,
    pub events: Vec<GameEvent>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerDamageReceivedPayload {
    pub session_id: SessionId,
    pub damage: Number,
    pub game_state: GameState,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EndPayload {
    pub winner_id: SessionId,
    pub winner_info: BotInfo,
    pub room_data: RoomData,
}
