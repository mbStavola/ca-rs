extern crate ws;
extern crate serde_json;

use ws::{Message, Result, Error, ErrorKind};

#[serde(tag = "type")]
#[derive(Debug, Deserialize, PartialEq)]
pub enum ClientAction {
    Join,
    Leave,
    Register { name: String },
    Chat { message: String },
    Submission { card_id: u8 },
    Discard { card_id: u8 },
    Vote { card_id: u8 },
    Interact { player_id: u8, action: AdminAction },
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum AdminAction {
    Ban,
    Unban,
}

impl ClientAction {
    pub fn parse(message: &Message) -> Result<ClientAction> {
        message.as_text().and_then(|text| {
            serde_json::from_str(text).map_err(|e| {
                let error = format!("Could not parse ClientAction: {}", text);
                Error::new(ErrorKind::Internal, error)
            })
        })
    }
}