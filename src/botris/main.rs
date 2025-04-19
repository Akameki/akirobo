pub mod api_messages;
pub mod websocket;

use api_messages::BotrisMsg;
use dotenv::{dotenv, var};
use owo_colors::OwoColorize;
use robo::{akirobo::Akirobo, botris::types::Command, tetris_core::snapshot::GameSnapshot};
use websocket::BotrisWebSocket;

// #[tokio::main]
fn main() {
    // tokio::spawn(async move {
    //     tokio::signal::ctrl_c().await.unwrap();
    //     std::process::exit(0);
    // });

    println!("{}", "Akirobo".blue().bold().on_white());

    dotenv().ok();
    let token = var("TOKEN").expect("Set TOKEN in .env");
    let room_key = var("ROOMKEY").expect("Set ROOMKEY in .env");
    let url = format!("wss://botrisbattle.com/ws?token={token}&roomKey={room_key}");

    let mut ws = BotrisWebSocket::new(url);

    loop {
        if let Some(message) = ws.read() {
            use BotrisMsg::*;
            match message {
                RequestMove { game_state, .. } => {
                    let mut akirobo = Akirobo::new();
                    if game_state.held.is_none() {
                        println!("Holding first piece!");
                        ws.send_actions(vec![Command::Hold])
                    } else {
                        let commands = akirobo.suggest_action(&GameSnapshot::from_state(&game_state));
                        ws.send_actions(commands);
                    }
                }
                PlayerAction { .. } => (),
                Error(payload) => println!("BotrisError: {}", payload.magenta()),
                RoomData { .. } => (),
                Authenticated { session_id } => println!("Authenticated ({session_id})"),
                PlayerJoined { .. } => println!("Player Joined"),
                PlayerLeft { .. } => println!("Player Left"),
                PlayerBanned { .. } => println!("Player banned"),
                PlayerUnbanned { .. } => println!("Player unbanned"),
                SettingsChanged { .. } => println!("Settings Changed"),
                GameStarted => println!("{}", "Game Started".cyan()),
                RoundStarted { .. } => println!("{}", "Round Started".cyan()),
                Action { .. } => panic!("uhhh"),
                PlayerDamageReceived { .. } => (),
                RoundOver { .. } => println!("{}", "Round Over".cyan()),
                GameOver { .. } => println!("{}", "Game Over".cyan()),
                GameReset { .. } => println!("{}", "Game Reset".cyan()),
            }
        }
    }
}
