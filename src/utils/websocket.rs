use std::env::var;
use dotenv::dotenv;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream, tungstenite::Message};
use futures_util::{stream::{SplitSink, SplitStream}, SinkExt, StreamExt};
use serde_json::{from_str, json};
use crossterm::style::Stylize;

use crate::utils::event_types::{BotrisMsg, RequestMovePayload, ActionPayload};

/// Prints errors in the format of
/// Error: Description
/// $args
///
/// $args can be any number of string literals
/// which will each be printed on a seperate line
///
/// # Example
/// ```
/// error!("Description",
///     "You got an error"
/// );
/// ```
///
/// Output:
/// """
/// Error: Descriptions
///     You got an error
/// """
macro_rules! error {
    ($err:literal$(, )?$($args:literal$(,)?)*) => {
        eprintln!("{}: {}", "Error".red().bold(), $err);
        $(eprintln!("    {}", $args);)*
    };
}



/// Represents a websocket that is connected
/// to the botrisbattle room
pub struct BotrisWebSocket {
    pub read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    pub write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
}

impl BotrisWebSocket {
    pub async fn new() -> Self {
        dotenv().ok();
        let token = var("TOKEN").expect("Set TOKEN in .env");
        let room_key = var("ROOMKEY").expect("Set ROOMKEY in .env");
        let url = format!("wss://botrisbattle.com/ws?token={}&roomKey={}",token, room_key);

        println!("Connecting to {}", url);
        let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
        println!("{}", "Connected!!! :3".green().bold());
        let (write, read) = ws_stream.split();
        BotrisWebSocket { read, write }
    }

    /// Reads a message from the botrisbattle server and returns 
    /// a parsed struct BotrisMsg representing its contents.
    /// It will return None on an error.
    pub async fn read (&mut self) -> Option<BotrisMsg> {
        let message = self.read.next().await?;
        let message = message.expect("Failed to read message").into_text().expect("Failed to convert message to text");
        // println!("{:#}", message); // Uncomment this from debugging
        match from_str(&message) {
            Ok(msg) => {
                // println!("{:#?}", msg); // Uncomment this for debugging
                msg
            },
            Err(err) => {
                error!("Failed to parse message");
                eprintln!("{}\n{}", message, err);
                None
            }
        }
    }

    /// Reads messages from the botrisbattle server and returns the next
    /// move request parsed into a RequestMovePayload struct representing
    /// its contents. It will skip past any non request move type messages
    // pub async fn read_next_move_request(&mut self) -> RequestMovePayload {
    //     loop {
    //         if let Some(message) = self.read().await {
    //             match message {
    //                 BotrisMsg::RequestMove { payload } => return payload,
    //                 _ => continue
    //             }
    //         }
    //     }
    // }

    /// Sends the server an action type message with the moves you want to 
    /// make.
    ///
    /// # Example
    /// ```
    /// let mut actions = ActionPayload::new();
    /// actions.push(Command::MoveLeft);
    /// actions.push(Command::MoveRight);
    /// ws.send_actions(actions).await;
    /// ```
    pub async fn send_actions(&mut self, actions: ActionPayload) {
        let event = BotrisMsg::Action(actions);
        let msg = Message::Text(json!(event).to_string());
        
        self.write.send(msg).await.unwrap();
        self.write.flush().await.unwrap();
    }
}
