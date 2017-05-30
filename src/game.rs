extern crate ws;
extern crate redis;
extern crate log;
extern crate serde;
extern crate serde_json;
extern crate serde_redis;

use player::{Player, PlayerState};
use action::ClientAction;

use ws::{Sender, Handler, Message, Error, ErrorKind, Handshake, CloseCode};
use redis::Commands;
use self::serde::{Serialize, Serializer};
use self::serde_redis::RedisDeserialize;

use std::sync::{Arc, Mutex, MutexGuard};

#[derive(Serialize)]
pub struct Game<'a> {
    #[serde(skip_serializing)]
    out: Sender,
    #[serde(skip_serializing)]
    client: &'a redis::Client,
    #[serde(skip_serializing)]
    config: &'a Config,
    state: SharedGameState,
    player: Option<Player>,
    watching: SharedPlayerList,
    playing: SharedPlayerList,
}

impl<'a> Game<'a> {
    pub fn new(out: Sender, client: &'a redis::Client, config: &'a Config, state: SharedGameState,
               watching: SharedPlayerList, playing: SharedPlayerList) -> Game<'a> {
        Game {
            out: out,
            client: client,
            config: config,
            state: state,
            player: None,
            watching: watching,
            playing: playing,
        }
    }

    fn met_player_quota(&self) -> bool {
        let playing_guard: MutexGuard<Vec<Player>> = match self.playing.as_ref().lock() {
            Ok(guard) => guard,
            Err(poison) => poison.into_inner()
        };

        playing_guard.len() as u8 >= self.config.min_players
    }

    fn hit_player_limit(&self) -> bool {
        let playing_guard: MutexGuard<Vec<Player>> = match self.playing.as_ref().lock() {
            Ok(guard) => guard,
            Err(poison) => poison.into_inner()
        };

        playing_guard.len() as u8 == self.config.max_players
    }
}

impl<'a> Handler for Game<'a> {
    fn on_open(&mut self, shake: Handshake) -> ws::Result<()> {
        if let Some(addr) = shake.remote_addr()? {
            let message = format!("Player at {} joined", addr);

            let redis_conn: redis::Connection = self.client.get_connection().expect("Could not make redis connection");
            let state_key: String = Player::format_state_key(&addr);

            self.player = redis_conn.get(state_key).ok()
                .map(|it: redis::Value| {
                    it.deserialize().unwrap_or(Player::new(addr, None))
                })
                .and_then(|mut player| {
                    match player.state {
                        PlayerState::Watching => {
                            let mut watching_guard: MutexGuard<Vec<Player>> = match self.watching.as_ref().lock() {
                                Ok(guard) => guard,
                                Err(poison) => poison.into_inner()
                            };

                            (*watching_guard).push(player.clone());
                        }
                        PlayerState::Playing => {
                            if self.hit_player_limit() {
                                player.state = PlayerState::Watching;

                                let mut watching_guard: MutexGuard<Vec<Player>> = match self.watching.as_ref().lock() {
                                    Ok(guard) => guard,
                                    Err(poison) => poison.into_inner()
                                };

                                (*watching_guard).push(player.clone());
                            } else {
                                let mut playing_guard: MutexGuard<Vec<Player>> = match self.playing.as_ref().lock() {
                                    Ok(guard) => guard,
                                    Err(poison) => poison.into_inner()
                                };

                                (*playing_guard).push(player.clone());
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

    fn on_message(&mut self, msg: Message) -> ws::Result<()> {
        if let Ok(action) = ClientAction::parse(&msg) {
            let redis_conn = self.client.get_connection().expect("Could not make redis connection");

            let mut watching_guard: MutexGuard<Vec<Player>> = match self.watching.as_ref().lock() {
                Ok(guard) => guard,
                Err(poison) => poison.into_inner()
            };

            let mut playing_guard: MutexGuard<Vec<Player>> = match self.playing.as_ref().lock() {
                Ok(guard) => guard,
                Err(poison) => poison.into_inner()
            };

            if let ClientAction::Register { name } = action {
                let name_exists = (*playing_guard).iter()
                    .chain((*watching_guard).iter())
                    .find(|item| {
                        if let Some(ref other_name) = item.name {
                            return &name == other_name;
                        }

                        false
                    }).is_some();

                if !name_exists {
                    let mut taken_player = self.player.take().unwrap();

                    let current_name = taken_player.name.unwrap_or("Anonymous".to_string());
                    let name_change = format!("{} is now known as {}.", current_name, name);

                    taken_player.name = Some(name);
                    self.player = Some(taken_player);

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

            // Once we've exhausted non-gamestate related actions, we need to acquire a gamestate lock
            let mut state_guard: MutexGuard<GameState> = match self.state.as_ref().lock() {
                Ok(guard) => guard,
                Err(poison) => poison.into_inner()
            };

            if !self.met_player_quota() {
                *state_guard = GameState::Inactive;
            } else if self.met_player_quota() && *state_guard == GameState::Inactive {
                *state_guard = GameState::Nominating;
            }

            // Interpret action wrt current gamestate
            match *state_guard {
                GameState::Inactive => {
                    if self.met_player_quota() {
                        *state_guard = GameState::Nominating
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
        }).expect("Could not broadcast JSON");

        Ok(())
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        if let Some(ref player) = self.player {
            match player.state {
                PlayerState::Watching => {
                    let mut watching_guard: MutexGuard<Vec<Player>> = match self.watching.as_ref().lock() {
                        Ok(guard) => guard,
                        Err(poison) => poison.into_inner()
                    };

                    watching_guard.retain(|ref watching_player| {
                        watching_player.addr != player.addr
                    })
                }
                PlayerState::Playing => {
                    let mut playing_guard: MutexGuard<Vec<Player>> = match self.playing.as_ref().lock() {
                        Ok(guard) => guard,
                        Err(poison) => poison.into_inner()
                    };

                    playing_guard.retain(|ref playing_player| {
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

#[derive(Debug, Serialize, PartialEq)]
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

pub struct SharedPlayerList(pub Arc<Mutex<Vec<Player>>>);

impl SharedPlayerList {
    pub fn new() -> SharedPlayerList {
        SharedPlayerList(Arc::new(Mutex::new(vec![])))
    }
}

impl Clone for SharedPlayerList {
    fn clone(&self) -> Self {
        SharedPlayerList(self.0.clone())
    }
}

impl AsRef<Mutex<Vec<Player>>> for SharedPlayerList {
    fn as_ref(&self) -> &Mutex<Vec<Player>> {
        self.0.as_ref()
    }
}

impl Serialize for SharedPlayerList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where
        S: Serializer {
        let guard: MutexGuard<Vec<Player>> = match self.as_ref().lock() {
            Ok(guard) => guard,
            Err(poison) => poison.into_inner()
        };


        guard.serialize(serializer)
    }
}

pub struct SharedGameState(pub Arc<Mutex<GameState>>);

impl SharedGameState {
    pub fn new() -> SharedGameState {
        SharedGameState(Arc::new(Mutex::new(GameState::Inactive)))
    }
}

impl Clone for SharedGameState {
    fn clone(&self) -> Self {
        SharedGameState(self.0.clone())
    }
}

impl AsRef<Mutex<GameState>> for SharedGameState {
    fn as_ref(&self) -> &Mutex<GameState> {
        self.0.as_ref()
    }
}

impl Serialize for SharedGameState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where
        S: Serializer {
        let guard: MutexGuard<GameState> = match self.as_ref().lock() {
            Ok(guard) => guard,
            Err(poison) => poison.into_inner()
        };

        guard.serialize(serializer)
    }
}

struct WrapRedisError(redis::RedisError);

struct WrapSerdeError(serde_json::Error);

impl Into<WrapRedisError> for redis::RedisError {
    fn into(self) -> WrapRedisError {
        WrapRedisError(self)
    }
}

impl From<WrapRedisError> for ws::Error {
    fn from(_: WrapRedisError) -> Self {
        Error::new(ErrorKind::Internal, "Could not get redis connection.")
    }
}

impl Into<WrapSerdeError> for serde_json::Error {
    fn into(self) -> WrapSerdeError {
        WrapSerdeError(self)
    }
}

impl From<WrapSerdeError> for ws::Error {
    fn from(_: WrapSerdeError) -> Self {
        Error::new(ErrorKind::Internal, "Could not get redis connection.")
    }
}