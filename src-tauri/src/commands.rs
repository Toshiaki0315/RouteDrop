//! フロントエンドとの通信 I/F。ロジックは `core` に委譲するだけ。
use std::sync::Mutex;

use tauri::State;

use crate::core::game::{Game, Input, Snapshot};
use crate::core::level::Level;

pub struct GameState(pub Mutex<Game>);

impl GameState {
    pub fn new() -> Self {
        GameState(Mutex::new(Game::new(Level::first_stage(), 0)))
    }
}

#[tauri::command]
pub fn new_game(state: State<GameState>, seed: u64) -> Snapshot {
    let mut game = state.0.lock().unwrap();
    *game = Game::new(Level::first_stage(), seed);
    game.snapshot()
}

#[tauri::command]
pub fn get_state(state: State<GameState>) -> Snapshot {
    state.0.lock().unwrap().snapshot()
}

#[tauri::command]
pub fn send_input(state: State<GameState>, input: Input) -> Snapshot {
    let mut game = state.0.lock().unwrap();
    game.handle_input(input);
    game.snapshot()
}

#[tauri::command]
pub fn tick(state: State<GameState>, dt_ms: u32) -> Snapshot {
    let mut game = state.0.lock().unwrap();
    game.advance(dt_ms);
    game.snapshot()
}
