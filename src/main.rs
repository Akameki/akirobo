pub mod utils;
pub mod bot;

use std::io::Write;

use crossterm::{
    cursor, queue, style::{Stylize, PrintStyledContent},
    terminal::{self, ClearType::*}, ExecutableCommand,
};

use utils::websocket::BotrisWebSocket;
use utils::event_types::BotrisMsg;
use bot::bot::Bot;

#[tokio::main]
async fn main() {
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        std::io::stdout().execute(cursor::Show).unwrap();
        std::process::exit(0);
    });

    let mut stdout = std::io::stdout();

    queue!(stdout,
        terminal::Clear(All),
        cursor::MoveTo(0,0),
        cursor::Hide,
        PrintStyledContent("\n\nAkirobo\n\n".blue().bold()),
    ).unwrap();
    stdout.flush().unwrap();
    
    let mut ws = BotrisWebSocket::new().await;

    queue!(stdout,
        cursor::MoveToNextLine(3),
        cursor::SavePosition
    ).unwrap();

    let mut bot = Bot::new();

    loop {
        if let Some(message) = ws.read().await {
            match message {
                BotrisMsg::RequestMove(payload) => {
                    let actions = bot.request_moves(&payload).await;
                    ws.send_actions(actions).await;
                },
                // BotrisMsg::GameReset { payload: _ } => {
                //     println!("Game Reset!");
                //     bot = Bot::new();
                // }
                // BotrisMsg::RoundOver { payload: _ } => {
                //     println!("Round Over!");
                //     bot = Bot::new();
                // }
                // BotrisMsg::GameOver { payload: _ } => {
                //     println!("Game Over!");
                //     bot = Bot::new();
                // }
                BotrisMsg::PlayerAction(payload) => {
                    // println!("received player action");
                },
                BotrisMsg::Error(payload) => {
                    println!("> {} {}", "Error", payload);
                },
                _ => {
                    // println!("> {:#?}", message);
                    println!("> {:?}", message);
                }
            }
        }
    }

    // loop {
    //     let mv_req = ws.read_next_move_request().await;
    //     let actions = bot.request_moves(&mv_req).await;
    //     ws.send_actions(actions).await;
    // }
}

