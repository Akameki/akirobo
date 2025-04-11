use std::env::var;

use dotenv::dotenv;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use owo_colors::OwoColorize;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

use super::types::Command;
use crate::botris::api_messages::BotrisMsg;

pub struct BotrisWebSocket {
    pub read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    pub write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
}

impl BotrisWebSocket {
    pub async fn new() -> Self {
        dotenv().ok();
        let token = var("TOKEN").expect("Set TOKEN in .env");
        let room_key = var("ROOMKEY").expect("Set ROOMKEY in .env");
        let url = format!("wss://botrisbattle.com/ws?token={token}&roomKey={room_key}");

        println!("Connecting to {}", url);
        let (ws_stream, _) = connect_async(url).await.expect("Failed to connect to websocket");
        println!("{}", "Connected".green().bold());
        let (write, read) = ws_stream.split();
        BotrisWebSocket { read, write }
    }

    /// Reads a message from the websocket.
    pub async fn read(&mut self) -> Option<BotrisMsg> {
        let message = self.read.next().await?;
        let message = message.expect("Failed to read message");
        let message = message.into_text().expect("Failed to convert message to text");
        match serde_json::from_str(&message) {
            Ok(msg) => msg,
            Err(err) => {
                eprintln!("{}{}\n{}", "Failed to parse message".red().bold(), message, err);
                None
            }
        }
    }

    /// Sends the server an action type message with the moves you want to make
    pub async fn send_actions(&mut self, commands: Vec<Command>) {
        let event = BotrisMsg::Action { commands };
        let msg = Message::Text(serde_json::json!(event).to_string());

        self.write.send(msg).await.unwrap();
        self.write.flush().await.unwrap();
    }
}
