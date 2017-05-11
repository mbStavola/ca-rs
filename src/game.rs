extern crate ws;
extern crate redis;
extern crate log;
extern crate serde_json;

use ws::{Sender, Handler, Message, Result, Error, ErrorKind, Handshake, CloseCode};
use redis::Commands;

use player::{Player, PlayerState};
use action::ClientAction;

#[derive(Serialize)]
pub struct Game<'a> {
    #[serde(skip_serializing)]
    out: Sender,
    #[serde(skip_serializing)]
    client: &'a redis::Client,
    #[serde(skip_serializing)]
    config: &'a Config,
    state: GameState,
    player: Option<Player>,
    watching: Vec<&'a Player>,
    playing: Vec<&'a Player>,
}

impl<'a> Game<'a> {
    pub fn new(out: Sender, client: &'a redis::Client, config: &'a Config) -> Game<'a> {
        Game {
            out: out,
            client: client,
            config: config,
            state: GameState::Inactive,
            player: None,
            watching: vec![],
            playing: vec![],
        }
    }

    fn met_player_quota(&self) -> bool {
        self.playing.len() as u8 >= self.config.min_players
    }

    fn hit_player_limit(&self) -> bool {
        self.playing.len() as u8 == self.config.max_players
    }
}

impl<'a> Handler for Game<'a> {
    fn on_open(&mut self, shake: Handshake) -> Result<()> {
        if let Some(addr) = shake.remote_addr()? {
            let message = format!("Player at {} joined", addr);

            let redis_conn = self.client.get_connection()
                .map_err(|err| Error::new(ErrorKind::Internal, "Could not get redis connection."))?;

            let state_key = Player::format_state_key(addr);

            self.player = redis_conn.get(state_key).ok()
                .or(Some(Player::new(addr, None)))
                .and_then(|player| {
                    match player.state {
                        PlayerState::Watching => self.watching.push(&player),
                        PlayerState::Playing => {
                            if self.hit_player_limit() {
                                player.state = PlayerState::Watching;
                                self.watching.push(&player);
                            } else {
                                self.playing.push(&player);
                            }
                        }
                        PlayerState::Judging => {}
                        PlayerState::TimeOut => {}
                        PlayerState::Banned => {}
                    }

                    Some(player)
                });

            self.out.broadcast(Message::text(message));
        }

        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        if let Ok(action) = ClientAction::parse(&msg) {
            let redis_conn = self.client.get_connection()
                .map_err(|err| Error::new(ErrorKind::Internal, "Could not get redis connection."))?;

            if let ClientAction::Register(name) = action {
                let name_exists = self.playing.iter()
                    .chain(self.watching.iter())
                    .find(|item| {
                        if let Some(ref other_name) = item.name {
                            return &name == other_name;
                        }

                        false
                    }).is_some();

                if !name_exists {
                    let _: () = redis_conn.set("my_name", name)
                        .map_err(|err| Error::new(ErrorKind::Internal, "Could not execute redis set command."))?;

                    let current_name = self.player.and_then(|player| Some(player.name))
                        .unwrap_or(Some("Anonymous".to_string())).unwrap();
                    let name_change = format!("{} is now known as {}.", current_name, name);

                    self.out.broadcast(Message::text(name_change))?;
                } else {
                    self.out.send(Message::text("Name already being used."))?;
                }

                return Ok(())
            }

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
        } else {
            debug!("Invalid message: {}", msg);
        }

        // Broadcast game state to client
        serde_json::to_string(self).and_then(|game_json| {
            self.out.broadcast(Message::text(game_json));
            Ok(())
        }).map_err(|err| Error::new(ErrorKind::Internal, "Could not execute redis set command."))?;

        Ok(())
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        if let Some(player) = self.player {
            match player.state {
                PlayerState::Watching => {
                    self.watching.retain(|&watching_player| {
                            watching_player.addr != player.addr
                        })
                },
                PlayerState::Playing => {
                    self.playing.retain(|&playing_player| {
                            playing_player.addr != player.addr
                        })
                }
                PlayerState::Judging => {}
                PlayerState::TimeOut => {}
                PlayerState::Banned => {}
            }
        }
    }
}

#[derive(Debug, Serialize)]
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