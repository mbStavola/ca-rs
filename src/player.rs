#[derive(Debug, Deserialize, Serialize)]
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

    pub fn format_state_key(addr: String) -> String {
        format!("STATE:{}", addr)
    }

    pub fn format_hand_key(addr: String, game_id: String) -> String {
        format!("HAND:{}:{}", addr, game_id)
    }

    pub fn state_key(&self) -> String {
        Player::format_state_key(self.addr)
    }

    pub fn hand_key(&self, game_id: String) -> String {
       Player::format_hand_key(self.addr, game_id)
    }
}

// States
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum PlayerState {
    Watching,
    Playing,
    Judging,
    TimeOut,
    Banned,
}

// Transitions