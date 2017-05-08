extern crate ws;
extern crate regex;
extern crate lazy_static;

use ws::{Message};
use self::regex::Regex;

pub enum ClientAction {
    Join,
    Chat(String),
    Submission(String),
    Discard(String),
    Vote(String),
    Interact(String, String),
}

impl ClientAction {
    pub fn parse(message: Message) -> Result<ClientAction, ()> {
        lazy_static! {
            static ref JOIN_EXP: Regex = Regex::new(r":join:").unwrap();
            static ref CHAT_EXP: Regex = Regex::new(r":chat:(.+)").unwrap();
            static ref SDV_EXP: Regex = Regex::new(r":(subm|disc|vote):(\d+)").unwrap();
            static ref INTE_EXP: Regex = Regex::new(r":inte:(\d+) (ban|unban)").unwrap();
        }

        message.into_text().map_err(|err| ()).and_then(|text| {
            let ref_text = text.as_ref();

            if JOIN_EXP.is_match(ref_text) {
                return Ok(ClientAction::Join);
            }

            if let Some(captures) = CHAT_EXP.captures(ref_text) {
                let chat_message = captures.get(1).unwrap().as_str().to_owned();

                let it = ClientAction::Chat(chat_message);

                return Ok(it);
            }

            if let Some(captures) = SDV_EXP.captures(ref_text) {
                let action = captures.get(1).unwrap().as_str();
                let id = captures.get(2).unwrap().as_str().to_owned();

                let it = match action {
                    "subm" => ClientAction::Submission(id),
                    "disc" => ClientAction::Discard(id),
                    "vote" => ClientAction::Vote(id),
                    _ => return Err(())
                };

                return Ok(it);
            }

            if let Some(captures) = INTE_EXP.captures(ref_text) {
                let id = captures.get(1).unwrap().as_str().to_owned();
                let action = captures.get(2).unwrap().as_str().to_owned();

                let it = ClientAction::Interact(id, action);

                return Ok(it);
            }

            return Err(())
        })
    }
}