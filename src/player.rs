pub struct Player {
    state: PlayerState
}

// States
enum PlayerState {
    WATCHING,
    PLAYING,
    JUDGING,
    TIMEOUT,
    BANNED,
}

// Transitions