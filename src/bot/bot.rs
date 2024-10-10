use crate::utils::event_types::{ActionPayload, RequestMovePayload};

pub trait BotTrait {
    fn request_moves(event: &RequestMovePayload) -> ActionPayload;
}

pub struct Bot {
    bag: u32,
    piece: u32,
}

impl Bot {
    pub fn new () -> Self {
        Self::default()
    }

    pub fn default() -> Self {
        Bot {
            bag: 0,
            piece: 0,
        }
    }

    pub async fn request_moves(&mut self, event: &RequestMovePayload) -> ActionPayload {
        // nothing .
        // wait one second
        // then return empty action
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        ActionPayload::new()
    }
}