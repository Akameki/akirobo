use serde::{Deserialize, Serialize};
use serde_json::Number;

use super::types::*;

#[derive(Debug, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "payload",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum BotrisMsg {
    // Join Room
    RoomData { room_data: RoomData },
    Authenticated { session_id: SessionId },

    // WS Messages
    Error(String),
    PlayerJoined {},   // player_data: PlayerData },
    PlayerLeft {},     // session_id: SessionId },
    PlayerBanned {},   // bot_info: BotInfo },
    PlayerUnbanned {}, // bot_info: BotInfo },
    SettingsChanged { room_data: RoomData },

    // HostChanged{ bot_info: BotInfo },

    // Ingame
    GameStarted,
    RoundStarted { starts_at: Number, room_data: RoomData },
    RequestMove { game_state: GameState }, // players: Vec<PlayerData> },
    Action { commands: Vec<Command> },

    PlayerAction {}, // commands: Vec<Command>, game_state: GameState, events: Vec<GameEvent> },
    PlayerDamageReceived {}, // session_id: SessionId, damage: Number, game_state: GameState },
    RoundOver {},    // winner_id: SessionId, winner_info: BotInfo, room_data: RoomData },
    GameOver {},     // winner_id: SessionId, winner_info: BotInfo, room_data: RoomData },
    GameReset { room_data: RoomData },
    // Ping(PingPayload),
}
