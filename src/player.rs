pub struct Player {
    state: PlayerState
}

// States
enum PlayerState {
    Watching,
    Playing,
    Judging,
    TimeOut,
    Banned,
}

// Transitions