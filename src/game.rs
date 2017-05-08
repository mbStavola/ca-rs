extern crate ws;

use ws::{Sender, Handler, Message, Result};

use player::Player;
use action::ClientAction;

pub struct Game {
    out: Sender,
    players: Vec<Player>,
    state: GameState
}

impl Game {
    pub fn new(out: Sender) -> Game {
        Game {
            out: out,
            players: vec![],
            state: GameState::Inactive,
        }
    }

    fn met_player_quota(&self) -> bool {
        return self.players.len() > 3
    }
}

impl Handler for Game {
    fn on_message(&mut self, msg: Message) -> Result<()> {
        let action = ClientAction::parse(msg);

        if !self.met_player_quota() {
            self.state = GameState::Inactive;
        }

        match self.state {
            GameState::Inactive => {
                if self.met_player_quota() {
                    self.state = GameState::Nominating;
                }
            },
            GameState::Nominating => {

            },
            GameState::RoundStart => {

            },
            GameState::RoundEnd => {

            },
            GameState::Paused => {

            },
        }

        self.out.broadcast(Message::text("AH"));

        Ok(())
    }
}

pub enum GameState {
    Inactive,
    Nominating,
    RoundStart,
    RoundEnd,
    Paused,
}

impl GameState {

}