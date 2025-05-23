use std::fmt::Display;

use serde::{Deserialize, Serialize};
use serde_json::Number;

pub type SessionId = String;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostType {
    // pub id: String,
    pub display_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomData {
    // pub id: String,
    pub host: HostType,
    // pub private: bool,
    // pub ft: Number,
    // pub pps: Number,
    // pub initial_multiplier: Number,
    // pub final_multiplier: Number,
    // pub start_margin: Number,
    // pub end_margin: Number,
    // pub max_players: Number,
    // pub game_ongoing: bool,
    // pub round_ongoing: bool,
    // pub started_at: Option<Number>,
    // pub ended_at: Option<Number>,
    // pub last_winner: Option<SessionId>,
    // pub players: Vec<PlayerData>,
    // pub banned: Vec<BotInfo>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerData {
    // pub session_id: SessionId,
    pub playing: bool,
    pub info: BotInfo,
    pub wins: Number,
    pub game_state: Option<GameState>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Developer {
    id: String,
    display_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BotInfo {
    pub id: String,
    pub name: String,
    // #[serde(skip)]
    // pub avatar: Bool,
    pub team: Option<String>,
    pub language: Option<String>,
    pub eval: Option<String>,
    pub movegen: Option<String>,
    pub search: Option<String>,
    pub developers: Vec<Developer>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Piece {
    I,
    O,
    J,
    L,
    S,
    Z,
    T,
}

#[derive(Debug, Deserialize, Serialize)]
// may or may not work
pub enum Block {
    I,
    O,
    J,
    L,
    S,
    Z,
    T,
    G,
    Null,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PieceData {
    pub piece: Piece,
    pub x: i16,
    pub y: i16,
    pub rotation: u16,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GarbageLine {
    pub delay: Number,
}

pub type Board = Vec<[Option<Block>; 10]>;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameState {
    pub board: Board,
    pub bag: Vec<Piece>,
    pub queue: Vec<Piece>,
    pub garbage_queued: Vec<GarbageLine>,
    pub held: Option<Piece>,
    pub current: PieceData,
    pub can_hold: bool,
    pub combo: u32,
    pub b2b: bool,
    // pub score: Number,
    pub pieces_placed: u32,
    pub dead: bool,
}

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Command {
    Hold,
    MoveLeft,
    MoveRight,
    SonicLeft,
    SonicRight,
    RotateCw,
    RotateCcw,
    Drop,
    SonicDrop,
    HardDrop,
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let symbol = match self {
            Command::Hold => "Hold",
            Command::MoveLeft => "<",
            Command::MoveRight => ">",
            Command::SonicLeft => "<<",
            Command::SonicRight => ">>",
            Command::RotateCw => "CW",
            Command::RotateCcw => "CCW",
            Command::Drop => "v",
            Command::SonicDrop => "V",
            Command::HardDrop => "!",
        };
        write!(f, "{}", symbol)
    }
}

impl std::fmt::Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ClearName {
    #[serde(rename = "Single")]
    Single,
    #[serde(rename = "Double")]
    Double,
    #[serde(rename = "Triple")]
    Triple,
    #[serde(rename = "Quad")]
    Quad,
    #[serde(rename = "All-Spin Single")]
    ASS,
    #[serde(rename = "All-Spin Double")]
    ASD,
    #[serde(rename = "All-Spin Triple")]
    AST,
    #[serde(rename = "Perfect Clear")]
    PC,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(
    tag = "type",
    content = "payload",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum GameEvent {
    PiecePlaced { initial: PieceData, r#final: PieceData },
    DamageTanked { hole_indices: Vec<Number> },
    QueueAdded {},
    Clear {},
    GameOver,
}

// #[derive(Debug, Deserialize, Serialize)]
// #[serde(rename_all = "camelCase")]
// pub struct ClearedLines {
//     pub height: Number,
//     pub blocks: Vec<Block>,
// }
