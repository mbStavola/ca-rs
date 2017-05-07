extern crate ws;

use ws::{Factory, Sender, Handler, Message, Result};

use player::Player;

pub struct Game {
    out: Sender,
    players: Vec<Player>,
    delegate: Box<StateHandler>
}

impl Handler for Game {
    fn on_message(&mut self, msg: Message) -> Result<()> {
        // Perform action and mutate delegate
        if let Ok(new_state) = self.delegate.on_message(msg) {
            self.delegate = new_state;
        }

        Ok(())
    }
}

pub struct Inactive<'a> {
    players: &'a Vec<Player>,
}

trait StateHandler {
    //    fn on_open(&mut self, shake: Handshake) -> Result<Box<StateHandler>>;
    fn on_message(&mut self, msg: Message) -> Result<Box<StateHandler>>;
    //    fn on_close(&mut self, code: CloseCode, reason: &str) -> Result<Box<StateHandler>>;

    //    #[inline]
    //    fn on_timeout(&mut self, event: Token) -> Result<Box<StateHandler>>;

    //    #[inline]
    //    fn on_new_timeout(&mut self, _: Token, _: Timeout) -> Result<Box<StateHandler>>;
}

impl<'a> StateHandler for Inactive<'a> {
    fn on_message(&mut self, msg: Message) -> Result<Box<StateHandler>> {
        if let Ok(text) = msg.as_text() {
            match text {
                _ => println!("{}", text)
            }
        }

        Ok(Box::new(Nominating::from(*self)))
    }
}

struct Nominating<'a> {
    players: &'a Vec<Player>,
}

impl<'a> StateHandler for Nominating<'a> {
    fn on_message(&mut self, msg: Message) -> Result<Box<StateHandler>> {
        unimplemented!()
    }
}

struct RoundStart<'a> {
    players: &'a Vec<Player>,
}

impl<'a> StateHandler for RoundStart<'a> {
    fn on_message(&mut self, msg: Message) -> Result<Box<StateHandler>> {
        unimplemented!()
    }
}

struct RoundEnd<'a> {
    players: &'a Vec<Player>,
}

impl<'a> StateHandler for RoundEnd<'a> {
    fn on_message(&mut self, msg: Message) -> Result<Box<StateHandler>> {
        unimplemented!()
    }
}

struct Paused<'a> {
    players: &'a Vec<Player>,
}

impl<'a> StateHandler for Paused<'a> {
    fn on_message(&mut self, msg: Message) -> Result<Box<StateHandler>> {
        unimplemented!()
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