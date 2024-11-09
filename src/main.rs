pub mod botris;
pub mod bot;
pub mod game;

use std::io::Write;

use crossterm::{
    cursor, queue, style::{Stylize, PrintStyledContent},
    terminal::{self, ClearType::*}, ExecutableCommand,
};

use botris::websocket::BotrisWebSocket;
use botris::api_messages::BotrisMsg;
use bot::bot::Bot;

#[tokio::main]
async fn main() {
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        std::io::stdout().execute(cursor::Show).unwrap();
        std::process::exit(0);
    });

    let mut stdout = std::io::stdout();

    queue!(stdout, terminal::Clear(All), cursor::MoveTo(0,0), cursor::Hide, PrintStyledContent("\n\nAkirobo\n\n".blue().bold()),).unwrap();
    stdout.flush().unwrap();
    
    let mut ws = BotrisWebSocket::new().await;
    let mut session_id = String::new();

    queue!(stdout,
        cursor::MoveToNextLine(3),
        cursor::SavePosition
    ).unwrap();

    let mut bot = Bot::new();

    loop {
        if let Some(message) = ws.read().await {
            match message {
                BotrisMsg::RequestMove(payload) => {
                    let actions=bot.suggest_action(&payload.game_state).await;
                    ws.send_actions(actions).await;
                },
                BotrisMsg::PlayerAction(payload) => {
                    // println!("> {} {}","PlayerAction",payload);
                    // println!("PlayerAction");
                },
                BotrisMsg::Error(payload) => println!("> {} {}", "BotrisError", payload),
                BotrisMsg::RoomData(_) => (),
                BotrisMsg::Authenticated(payload) => {
                    session_id = payload.session_id;
                    println!("Authenticated with SId: {}", session_id);
                },
                BotrisMsg::PlayerJoined(_) => (),
                BotrisMsg::PlayerLeft(_) => (),
                BotrisMsg::PlayerBanned(_) => (),
                BotrisMsg::PlayerUnbanned(_) => (),
                BotrisMsg::SettingsChanged(_) => (),
                BotrisMsg::GameStarted => println!("Game Started"),
                BotrisMsg::RoundStarted(_) => println!("Round Started"),
                BotrisMsg::Action(action_payload) => panic!("uhhh"),
                BotrisMsg::PlayerDamageReceived(player_damage_received_payload) => (),
                BotrisMsg::RoundOver(end_payload) => println!("Round Over"),
                BotrisMsg::GameOver(end_payload) => println!("Game Over"),
                BotrisMsg::GameReset(room_data_payload) => println!("Game Reset"),
            }
        }
    }

    // loop {
    //     let mv_req = ws.read_next_move_request().await;
    //     let actions = bot.request_moves(&mv_req).await;
    //     ws.send_actions(actions).await;
    // }
}

