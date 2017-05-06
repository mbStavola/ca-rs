extern crate ws;

use ws::{Factory, Sender, Handler, Message, Result};

use player::Player;

// We should probably use RefCells so we can borrow the struct and mutate it across threads
pub struct Game<S: Handler> {
    state: S
}

impl<S: Handler> Factory for Game<S> {
    type Handler = S;

    fn connection_made(&mut self, _: Sender) -> Self::Handler {
        self.state
    }

    fn client_connected(&mut self, ws: Sender) -> Self::Handler {
        self.connection_made(ws)
    }
}

struct Inactive<'a> {
    players: &'a mut Vec<Player>,
}

impl<'a> Handler for Inactive<'a> {
    fn on_message(&mut self, msg: Message) -> Result<()> {
        if let Ok(text) = msg.as_text() {
            match text {
                _ => println!("{}", text)
            }
        }

        let new_state: Nominating = Nominating::from(*self);

        Ok(())
    }
}

struct Nominating<'a> {
    players: &'a mut Vec<Player>,
}

impl<'a> Handler for Nominating<'a> {
    fn on_message(&mut self, msg: Message) -> Result<()> {
        Ok(())
    }
}

struct RoundStart<'a> {
    players: &'a mut Vec<Player>,
}

impl<'a> Handler for RoundStart<'a> {
    fn on_message(&mut self, msg: Message) -> Result<()> {
        Ok(())
    }
}

struct RoundEnd<'a> {
    players: &'a mut Vec<Player>,
}

impl<'a> Handler for RoundEnd<'a> {
    fn on_message(&mut self, msg: Message) -> Result<()> {
        Ok(())
    }
}

struct Paused<'a> {
    players: &'a mut Vec<Player>,
}

impl<'a> Handler for Paused<'a> {
    fn on_message(&mut self, msg: Message) -> Result<()> {
        Ok(())
    }
}

// Games start off in the inactive state
impl<'a> Game<Inactive<'a>> {
    pub fn new(players: &'a mut Vec<Player>) -> Self {
        let game = Game {
            state: Inactive {
                players: players,
            }
        };

        game
    }
}

// Transitions

// Transition for when player quota is not met while nominating
impl<'a> From<Nominating<'a>> for Inactive<'a> {
    fn from(prev: Nominating<'a>) -> Inactive<'a> {
        Inactive {
            players: prev.players,
        }
    }
}

// Transition on player join
impl<'a> From<Inactive<'a>> for Nominating<'a> {
    fn from(prev: Inactive<'a>) -> Nominating<'a> {
        Nominating {
            players: prev.players,
        }
    }
}

// Transition when player quota is not met while round is starting
impl<'a> From<RoundStart<'a>> for Nominating<'a> {
    fn from(prev: RoundStart<'a>) -> Nominating<'a> {
        Nominating {
            players: prev.players,
        }
    }
}

// Transition when player quota is met while nominating
impl<'a> From<Nominating<'a>> for RoundStart<'a> {
    fn from(prev: Nominating<'a>) -> RoundStart<'a> {
        RoundStart {
            players: prev.players,
        }
    }
}

// Transition for a completed round
impl<'a> From<RoundEnd<'a>> for RoundStart<'a> {
    fn from(prev: RoundEnd<'a>) -> RoundStart<'a> {
        RoundStart {
            players: prev.players,
        }
    }
}

// Transition when judge chooses or times out
impl<'a> From<RoundStart<'a>> for RoundEnd<'a> {
    fn from(prev: RoundStart<'a>) -> RoundEnd<'a> {
        RoundEnd {
            players: prev.players,
        }
    }
}