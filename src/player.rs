extern crate serde_redis;

use self::serde_redis::RedisDeserialize;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Player {
    pub addr: String,
    pub name: Option<String>,
    pub state: PlayerState
}

impl Player {
    pub fn new(addr: String, name: Option<String>) -> Player {
        Player {
            addr: addr,
            name: name,
            state: PlayerState::Watching
        }
    }

    pub fn format_state_key(addr: &str) -> String {
        format!("STATE:{}", addr)
    }

    pub fn format_hand_key(addr: &str, game_id: &str) -> String {
        format!("HAND:{}:{}", addr, game_id)
    }

    pub fn state_key(&self) -> String {
        Player::format_state_key(&self.addr)
    }

    pub fn hand_key(&self, game_id: &str) -> String {
       Player::format_hand_key(&self.addr, game_id)
    }
}

// States
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type")]
pub enum PlayerState {
    Watching,
    Playing,
    Judging,
    TimeOut,
    Banned,
}

// Transitions