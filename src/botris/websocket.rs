use std::net::TcpStream;


use owo_colors::OwoColorize;
use robo::botris::types::Command;
use tungstenite::{connect, stream::MaybeTlsStream, WebSocket};

use crate::api_messages::BotrisMsg;

pub struct BotrisWebSocket {
    ws: WebSocket<MaybeTlsStream<TcpStream>>,
}

impl BotrisWebSocket {
    pub fn new(url: String) -> Self {
        println!("Connecting to {}", url);
        let (ws, _) = connect(url).expect("Failed to connect to websocket");
        println!("{}", "Connected".green().bold());
        BotrisWebSocket { ws }
    }

    /// Reads a message from the websocket.
    pub fn read(&mut self) -> Option<BotrisMsg> {
        let message = self.ws.read().expect("Error reading from websocket");
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
    pub fn send_actions(&mut self, commands: Vec<Command>) {
        let event = BotrisMsg::Action { commands };
        let msg = tungstenite::Message::text(serde_json::json!(event).to_string());
        self.ws.send(msg).expect("Failed to send message");
    }
}
