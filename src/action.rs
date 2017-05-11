extern crate ws;
extern crate regex;
extern crate lazy_static;

use ws::{Message, Result, Error, ErrorKind};
use self::regex::Regex;

#[derive(Debug, PartialEq)]
pub enum ClientAction {
    Join,
    Leave,
    Register(String),
    Chat(String),
    Submission(String),
    Discard(String),
    Vote(String),
    Interact(String, String),
}

impl ClientAction {
    pub fn parse(message: &Message) -> Result<ClientAction> {
        lazy_static! {
            static ref JOLE_EXP: Regex = Regex::new(r":(join|leav):").unwrap();
            static ref REGI_EXP: Regex = Regex::new(r":regi:(\w{3,10})").unwrap();
            static ref CHAT_EXP: Regex = Regex::new(r":chat:(.{1,128})").unwrap();
            static ref SDV_EXP: Regex = Regex::new(r":(subm|disc|vote):(\d+)").unwrap();
            static ref INTE_EXP: Regex = Regex::new(r":inte:(\d+) (ban|unban)").unwrap();
        }

        message.as_text().and_then(|text| {
            let ref_text = text.as_ref();

            // Handles a join or leave request for a game
            if let Some(captures) = JOLE_EXP.captures(ref_text) {
                let action = captures.get(1).unwrap().as_str();

                let it = match action {
                    "join" => ClientAction::Join,
                    "leav" => ClientAction::Leave,
                    _ => return Err(Error::new(ErrorKind::Internal, "Could not parse ClientAction."))
                };

                return Ok(it);
            }

            // Handles registering a username
            if let Some(captures) = REGI_EXP.captures(ref_text) {
                let name = captures.get(1).unwrap().as_str().to_owned();

                let it = ClientAction::Register(name);

                return Ok(it);
            }

            // Handles sending a chat message
            if let Some(captures) = CHAT_EXP.captures(ref_text) {
                let chat_message = captures.get(1).unwrap().as_str().to_owned();

                let it = ClientAction::Chat(chat_message);

                return Ok(it);
            }

            // Handles submitting or discarding a card, voting on a submission
            if let Some(captures) = SDV_EXP.captures(ref_text) {
                let action = captures.get(1).unwrap().as_str();
                let id = captures.get(2).unwrap().as_str().to_owned();

                let it = match action {
                    "subm" => ClientAction::Submission(id),
                    "disc" => ClientAction::Discard(id),
                    "vote" => ClientAction::Vote(id),
                    _ => return Err(Error::new(ErrorKind::Internal, "Could not parse ClientAction."))
                };

                return Ok(it);
            }

            // Handles player to player interactions
            if let Some(captures) = INTE_EXP.captures(ref_text) {
                let id = captures.get(1).unwrap().as_str().to_owned();
                let action = captures.get(2).unwrap().as_str().to_owned();

                let it = ClientAction::Interact(id, action);

                return Ok(it);
            }

            Err(Error::new(ErrorKind::Internal, "Could not parse ClientAction."))
        })
    }
}