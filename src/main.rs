pub mod bot;
pub mod botris;
pub mod game;

// use bot::bot::Bot;
use bot::akirobo::{self, Akirobo};
use botris::{api_messages::BotrisMsg, websocket::BotrisWebSocket};
use owo_colors::OwoColorize;

#[tokio::main]
async fn main() {
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        std::process::exit(0);
    });

    println!("{}", "Akirobo".blue().bold().on_white());

    let mut ws = BotrisWebSocket::new().await;
    let mut my_session_id;

    loop {
        if let Some(message) = ws.read().await {
            match message {
                BotrisMsg::RequestMove { game_state, .. } => {
                    // let mut bot = Bot::new();
                    let mut akirobo = Akirobo::new();
                    let commands = akirobo.suggest_action(&game_state).await;
                    ws.send_actions(commands).await;
                }
                BotrisMsg::PlayerAction { .. } => (),
                BotrisMsg::Error(payload) => println!("> BotrisError {}", payload),
                BotrisMsg::RoomData { .. } => (),
                BotrisMsg::Authenticated { session_id } => {
                    my_session_id = session_id;
                    println!("Authenticated with SId: {}", my_session_id);
                }
                BotrisMsg::PlayerJoined { .. } => (),
                BotrisMsg::PlayerLeft { .. } => (),
                BotrisMsg::PlayerBanned { .. } => (),
                BotrisMsg::PlayerUnbanned { .. } => (),
                BotrisMsg::SettingsChanged { .. } => (),
                BotrisMsg::GameStarted => println!("Game Started"),
                BotrisMsg::RoundStarted { .. } => println!("Round Started"),
                BotrisMsg::Action { .. } => panic!("uhhh"),
                BotrisMsg::PlayerDamageReceived { .. } => (),
                BotrisMsg::RoundOver { .. } => println!("Round Over"),
                BotrisMsg::GameOver { .. } => println!("Game Over"),
                BotrisMsg::GameReset { .. } => println!("Game Reset"),
            }
        }
    }
}
