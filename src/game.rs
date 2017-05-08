extern crate ws;
extern crate redis;

use ws::{Sender, Handler, Message, Result};

use player::Player;
use action::ClientAction;

pub struct Game<'a> {
    out: Sender,
    client: &'a redis::Client,
    config: &'a Config,
    waiting: Vec<Player>,
    players: Vec<Player>,
    state: GameState
}

impl <'a> Game<'a> {
    pub fn new(out: Sender, client: &'a redis::Client, config: &'a Config) -> Game<'a> {
        Game {
            out: out,
            client: client,
            config: config,
            waiting: vec![],
            players: vec![],
            state: GameState::Inactive,
        }
    }

    fn met_player_quota(&self) -> bool {
        self.players.len() as u8 >= self.config.min_players
    }

    fn hit_player_limit(&self) -> bool {
        self.players.len() as u8 == self.config.max_players
    }
}

impl <'a> Handler for Game<'a> {
    fn on_message(&mut self, msg: Message) -> Result<()> {
        let action = ClientAction::parse(msg)?;

        if action == ClientAction::Join && !self.hit_player_limit() {
            // Put player in game pool
            return Ok(());
        }

        if !self.met_player_quota() {
            self.state = GameState::Inactive;
        }

        // Interpret action wrt current gamestate
        match self.state {
            GameState::Inactive => {
                if self.met_player_quota() {
                    self.state = GameState::Nominating;
                }
            }
            GameState::Nominating => {}
            GameState::RoundStart => {}
            GameState::RoundEnd => {}
            GameState::Paused => {}
        }

        // Broadcast game state to client
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

pub struct Config {
    min_players: u8,
    max_players: u8,
    timeout: u8
}

impl Config {
    pub fn new(min_players: u8, max_players: u8, timeout: u8) -> Config {
        Config {
            min_players: min_players,
            max_players: max_players,
            timeout: timeout
        }
    }
}