pub mod bot;
pub mod botris;
pub mod game;

// use bot::bot::Bot;
use bot::akirobo::Akirobo;
use botris::{api_messages::BotrisMsg, types::Command, websocket::BotrisWebSocket};
use dotenv::{dotenv, var};
use game::frame::Frame;
use owo_colors::OwoColorize;

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
            match message {
                BotrisMsg::RequestMove { game_state, .. } => {
                    // println!("{:?}", game_state.garbage_queued);
                    let mut akirobo = Akirobo::new();
                    if game_state.held.is_none() {
                        println!("Holding first piece!");
                        ws.send_actions(vec![Command::Hold])
                    } else {
                        let commands = akirobo.suggest_action(Frame::from_state(&game_state));
                        ws.send_actions(commands);
                    }
                }
                BotrisMsg::PlayerAction { .. } => (),
                BotrisMsg::Error(payload) => println!("BotrisError: {}", payload.magenta()),
                BotrisMsg::RoomData { .. } => (),
                BotrisMsg::Authenticated { session_id } => println!("Authenticated ({session_id})"),
                BotrisMsg::PlayerJoined { .. } => println!("Player Joined"),
                BotrisMsg::PlayerLeft { .. } => println!("Player Left"),
                BotrisMsg::PlayerBanned { .. } => println!("Player banned"),
                BotrisMsg::PlayerUnbanned { .. } => println!("Player unbanned"),
                BotrisMsg::SettingsChanged { .. } => println!("Settings Changed"),
                BotrisMsg::GameStarted => println!("{}", "Game Started".cyan()),
                BotrisMsg::RoundStarted { .. } => println!("{}", "Round Started".cyan()),
                BotrisMsg::Action { .. } => panic!("uhhh"),
                BotrisMsg::PlayerDamageReceived { .. } => (),
                BotrisMsg::RoundOver { .. } => println!("{}", "Round Over".cyan()),
                BotrisMsg::GameOver { .. } => println!("{}", "Game Over".cyan()),
                BotrisMsg::GameReset { .. } => println!("{}", "Game Reset".cyan()),
            }
        }
    }
}
