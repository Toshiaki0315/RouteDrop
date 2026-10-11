use serde::{Deserialize, Serialize};

use super::grid::{Cell, Grid};
use super::level::Level;
use super::piece::Piece;
use super::rng::Rng;
use super::robot::{Robot, StepResult};
use super::types::{Direction, Panel, Pos};

/// 落下ブロックの自然落下間隔 (ms)。
pub const FALL_INTERVAL_MS: u32 = 1000;
/// ロボットの進行間隔 (ms)。
pub const ROBOT_INTERVAL_MS: u32 = 500;

const SCORE_PER_STEP: u32 = 10;
const SCORE_ON_CLEAR: u32 = 100;
const SCORE_PER_LINE: u32 = 50;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Playing,
    Cleared,
    GameOver,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Input {
    MoveLeft,
    MoveRight,
    SoftDrop,
    HardDrop,
    Rotate,
}

/// UI へ渡す描画用の状態。
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Snapshot {
    pub grid: Grid,
    pub piece: Option<Piece>,
    /// 次に降ってくるパネル (プレビュー用)。
    pub next: Panel,
    pub robot: Robot,
    /// 盤面の外にあるスタート駅とゴール駅。
    pub start: Pos,
    pub goal: Pos,
    pub status: Status,
    pub score: u32,
}

#[derive(Clone, Debug)]
pub struct Game {
    level: Level,
    grid: Grid,
    piece: Option<Piece>,
    next: Panel,
    robot: Robot,
    status: Status,
    score: u32,
    rng: Rng,
    fall_acc: u32,
    robot_acc: u32,
}

impl Game {
    pub fn new(level: Level, seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let next = Panel::new(rng.next_panel(), Direction::North);
        let mut game = Game {
            grid: level.build_grid(),
            robot: level.spawn_robot(),
            level,
            piece: None,
            next,
            status: Status::Playing,
            score: 0,
            rng,
            fall_acc: 0,
            robot_acc: 0,
        };
        game.spawn_piece();
        game
    }

    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            grid: self.grid.clone(),
            piece: self.piece,
            next: self.next,
            robot: self.robot,
            start: self.level.start,
            goal: self.level.goal,
            status: self.status,
            score: self.score,
        }
    }

    pub fn status(&self) -> Status {
        self.status
    }

    /// 経過時間を与えて、落下とロボット進行の Tick を必要回数だけ進める。
    pub fn advance(&mut self, dt_ms: u32) {
        if self.status != Status::Playing {
            return;
        }
        self.fall_acc += dt_ms;
        self.robot_acc += dt_ms;
        while self.status == Status::Playing && self.robot_acc >= ROBOT_INTERVAL_MS {
            self.robot_acc -= ROBOT_INTERVAL_MS;
            self.tick_robot();
        }
        while self.status == Status::Playing && self.fall_acc >= FALL_INTERVAL_MS {
            self.fall_acc -= FALL_INTERVAL_MS;
            self.tick_fall();
        }
    }

    pub fn handle_input(&mut self, input: Input) {
        if self.status != Status::Playing {
            return;
        }
        match input {
            Input::MoveLeft => self.shift_piece(-1),
            Input::MoveRight => self.shift_piece(1),
            Input::Rotate => {
                if let Some(piece) = self.piece.as_mut() {
                    piece.rotate();
                }
            }
            Input::SoftDrop => {
                self.fall_acc = 0;
                self.tick_fall();
            }
            Input::HardDrop => {
                self.fall_acc = 0;
                while let Some(piece) = self.piece.as_mut() {
                    if !piece.try_move(&self.grid, 0, 1) {
                        break;
                    }
                }
                self.lock_piece();
            }
        }
    }

    fn shift_piece(&mut self, dx: i32) {
        if let Some(piece) = self.piece.as_mut() {
            piece.try_move(&self.grid, dx, 0);
        }
    }

    fn tick_fall(&mut self) {
        let moved = match self.piece.as_mut() {
            Some(piece) => piece.try_move(&self.grid, 0, 1),
            None => return,
        };
        if !moved {
            self.lock_piece();
        }
    }

    fn lock_piece(&mut self) {
        if let Some(piece) = self.piece.take() {
            self.grid.set(piece.pos, Cell::Panel(piece.panel));
        }
        self.clear_lines();
        self.spawn_piece();
    }

    /// ロボットより下 (通過済み) の行が全部パネルで埋まったら消す。
    /// ロボットの行とそれより上は経路に必要なので消さない。
    /// 消えた行より上にいるロボットは、乗っているパネルと一緒に 1 段下がる。
    fn clear_lines(&mut self) {
        let mut y = self.grid.height() as i32 - 1;
        while y > self.robot.pos.y {
            if self.grid.is_row_full(y as usize) {
                self.grid.remove_row(y as usize);
                self.robot.pos.y += 1;
                self.score += SCORE_PER_LINE;
            } else {
                y -= 1;
            }
        }
    }

    /// 次のパネルを、ランダムな列の最上段に出現させる。
    fn spawn_piece(&mut self) {
        let x = (self.rng.next_u32() % self.level.width as u32) as i32;
        let pos = self.level.spawn_pos(x);
        if self.grid.is_free(pos) {
            self.piece = Some(Piece::new(pos, self.next));
            self.next = Panel::new(self.rng.next_panel(), Direction::North);
        } else {
            self.piece = None;
            self.status = Status::GameOver;
        }
    }

    fn tick_robot(&mut self) {
        match self.robot.step(&self.grid, self.level.goal) {
            StepResult::Moved => self.score += SCORE_PER_STEP,
            StepResult::Waiting => {}
            StepResult::Reached => {
                self.score += SCORE_PER_STEP + SCORE_ON_CLEAR;
                self.status = Status::Cleared;
            }
            StepResult::Crashed => self.status = Status::GameOver,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::PanelKind;

    fn game() -> Game {
        Game::new(Level::first_stage(), 1)
    }

    fn panel(kind: PanelKind, facing: Direction) -> Panel {
        Panel::new(kind, facing)
    }

    fn straight_north() -> Cell {
        Cell::Panel(panel(PanelKind::Straight, Direction::North))
    }

    /// 落下ブロックを指定のパネル・位置に差し替える。
    fn set_piece(game: &mut Game, x: i32, y: i32, panel: Panel) {
        game.piece = Some(Piece::new(Pos::new(x, y), panel));
    }

    /// 落下ブロックを (x, 0) の直進パネルに差し替える。
    fn set_straight_at(game: &mut Game, x: i32) {
        set_piece(game, x, 0, panel(PanelKind::Straight, Direction::North));
    }

    /// 行 `y` の `xs` 列をパネルで埋める。
    fn fill_row(game: &mut Game, y: i32, xs: std::ops::Range<i32>) {
        for x in xs {
            game.grid.set(Pos::new(x, y), straight_north());
        }
    }

    #[test]
    fn new_game_starts_outside_the_field() {
        let g = game();
        let s = g.snapshot();
        assert_eq!(s.status, Status::Playing);
        assert_eq!(s.score, 0);
        assert_eq!(s.robot.pos, Pos::new(3, 12));
        assert_eq!(s.robot.heading, Direction::North);
        assert_eq!(s.start, Pos::new(3, 12));
        assert_eq!(s.goal, Pos::new(3, -1));
        assert_eq!(s.piece.unwrap().pos.y, 0);
        assert_eq!(g.status(), Status::Playing);
    }

    #[test]
    fn spawn_column_is_random_but_inside_the_field() {
        let mut columns = std::collections::HashSet::new();
        for seed in 0..40 {
            let x = Game::new(Level::first_stage(), seed).piece.unwrap().pos.x;
            assert!((0..7).contains(&x));
            columns.insert(x);
        }
        assert!(columns.len() > 1);
    }

    #[test]
    fn next_preview_becomes_the_next_falling_piece() {
        let mut g = game();
        let preview = g.snapshot().next;
        assert_eq!(preview.facing, Direction::North);
        g.handle_input(Input::HardDrop);
        assert_eq!(g.piece.unwrap().panel, preview);
    }

    #[test]
    fn move_left_and_right() {
        let mut g = game();
        set_straight_at(&mut g, 3);
        g.handle_input(Input::MoveRight);
        g.handle_input(Input::MoveRight);
        assert_eq!(g.piece.unwrap().pos, Pos::new(5, 0));
        g.handle_input(Input::MoveLeft);
        assert_eq!(g.piece.unwrap().pos, Pos::new(4, 0));
    }

    #[test]
    fn move_stops_at_board_edge() {
        let mut g = game();
        set_straight_at(&mut g, 3);
        for _ in 0..20 {
            g.handle_input(Input::MoveLeft);
        }
        assert_eq!(g.piece.unwrap().pos.x, 0);
        for _ in 0..20 {
            g.handle_input(Input::MoveRight);
        }
        assert_eq!(g.piece.unwrap().pos.x, 6);
    }

    #[test]
    fn idle_play_survives_many_pieces() {
        let mut g = game();
        for _ in 0..5 {
            g.handle_input(Input::HardDrop);
        }
        assert_eq!(g.status(), Status::Playing);
    }

    #[test]
    fn rotate_turns_piece_clockwise() {
        let mut g = game();
        set_piece(&mut g, 4, 0, panel(PanelKind::TurnRight, Direction::North));
        g.handle_input(Input::Rotate);
        assert_eq!(
            g.piece.unwrap().panel,
            panel(PanelKind::TurnRight, Direction::East)
        );
    }

    #[test]
    fn new_pieces_start_facing_north() {
        assert_eq!(game().piece.unwrap().panel.facing, Direction::North);
    }

    #[test]
    fn soft_drop_moves_down_one() {
        let mut g = game();
        set_straight_at(&mut g, 2);
        g.handle_input(Input::SoftDrop);
        assert_eq!(g.piece.unwrap().pos, Pos::new(2, 1));
    }

    #[test]
    fn soft_drop_on_floor_locks_and_respawns() {
        let mut g = game();
        set_piece(&mut g, 4, 11, panel(PanelKind::TurnLeft, Direction::East));
        g.handle_input(Input::SoftDrop);
        assert_eq!(
            g.grid.get(Pos::new(4, 11)),
            Some(Cell::Panel(panel(PanelKind::TurnLeft, Direction::East)))
        );
        assert_eq!(g.piece.unwrap().pos.y, 0);
    }

    #[test]
    fn hard_drop_lands_on_the_floor_then_stacks() {
        let mut g = game();
        set_straight_at(&mut g, 3);
        g.handle_input(Input::HardDrop);
        assert_eq!(g.grid.get(Pos::new(3, 11)), Some(straight_north()));
        set_straight_at(&mut g, 3);
        g.handle_input(Input::HardDrop);
        assert_eq!(g.grid.get(Pos::new(3, 10)), Some(straight_north()));
    }

    #[test]
    fn hard_drop_without_piece_is_harmless() {
        let mut g = game();
        g.piece = None;
        g.handle_input(Input::HardDrop);
        assert!(g.piece.is_some());
    }

    #[test]
    fn rotate_and_shift_without_piece_are_harmless() {
        let mut g = game();
        g.piece = None;
        g.handle_input(Input::Rotate);
        g.handle_input(Input::MoveLeft);
        g.handle_input(Input::SoftDrop);
        assert!(g.piece.is_none());
    }

    #[test]
    fn piece_falls_naturally_every_second() {
        let mut g = game();
        g.advance(FALL_INTERVAL_MS - 1);
        assert_eq!(g.piece.unwrap().pos.y, 0);
        g.advance(1);
        assert_eq!(g.piece.unwrap().pos.y, 1);
    }

    #[test]
    fn robot_enters_the_field_every_half_second() {
        let mut g = game();
        g.grid.set(Pos::new(3, 11), straight_north());
        g.advance(ROBOT_INTERVAL_MS - 1);
        assert_eq!(g.robot.pos, Pos::new(3, 12));
        g.advance(1);
        assert_eq!(g.robot.pos, Pos::new(3, 11));
        assert_eq!(g.snapshot().score, SCORE_PER_STEP);
    }

    #[test]
    fn robot_waits_at_the_station_when_path_is_empty() {
        let mut g = game();
        g.advance(ROBOT_INTERVAL_MS);
        assert_eq!(g.robot.pos, Pos::new(3, 12));
        assert_eq!(g.status(), Status::Playing);
    }

    #[test]
    fn robot_crash_is_game_over() {
        let mut g = game();
        g.grid.set(Pos::new(3, 11), Cell::Wall);
        g.advance(ROBOT_INTERVAL_MS);
        assert_eq!(g.status(), Status::GameOver);
    }

    #[test]
    fn robot_reaching_goal_station_clears_stage() {
        let mut g = game();
        for y in 0..=11 {
            g.grid.set(Pos::new(3, y), straight_north());
        }
        g.piece = None; // 落下ブロックが経路に干渉しないようにする
        g.advance(ROBOT_INTERVAL_MS * 13);
        assert_eq!(g.status(), Status::Cleared);
        assert_eq!(g.robot.pos, Pos::new(3, -1));
        assert_eq!(g.snapshot().score, SCORE_PER_STEP * 13 + SCORE_ON_CLEAR);
    }

    #[test]
    fn blocked_spawn_is_game_over() {
        let mut g = game();
        fill_row(&mut g, 0, 0..7);
        g.piece = None;
        g.spawn_piece();
        assert_eq!(g.status(), Status::GameOver);
        assert!(g.piece.is_none());
    }

    #[test]
    fn nothing_happens_after_game_ends() {
        let mut g = game();
        g.status = Status::GameOver;
        let before = g.snapshot();
        g.advance(10_000);
        g.handle_input(Input::MoveLeft);
        g.handle_input(Input::HardDrop);
        assert_eq!(g.snapshot(), before);
    }

    #[test]
    fn advance_stops_mid_way_when_game_ends() {
        let mut g = game();
        g.grid.set(Pos::new(3, 11), Cell::Wall);
        g.advance(FALL_INTERVAL_MS * 5);
        assert_eq!(g.status(), Status::GameOver);
    }

    #[test]
    fn same_seed_gives_same_piece_sequence() {
        let mut a = Game::new(Level::first_stage(), 99);
        let mut b = Game::new(Level::first_stage(), 99);
        for _ in 0..5 {
            a.handle_input(Input::HardDrop);
            b.handle_input(Input::HardDrop);
            assert_eq!(a.snapshot(), b.snapshot());
        }
    }

    #[test]
    fn full_row_below_robot_disappears_and_things_above_drop() {
        let mut g = game();
        g.robot.pos = Pos::new(3, 5); // 行 11 はロボットより下 (通過済み)
        fill_row(&mut g, 11, 0..6);
        g.grid.set(Pos::new(1, 10), straight_north());
        set_piece(&mut g, 6, 0, panel(PanelKind::Straight, Direction::North));
        g.handle_input(Input::HardDrop);

        assert_eq!(g.grid.get(Pos::new(1, 11)), Some(straight_north()));
        assert_eq!(g.grid.get(Pos::new(1, 10)), Some(Cell::Empty));
        assert_eq!(g.grid.get(Pos::new(6, 11)), Some(Cell::Empty));
        assert_eq!(g.robot.pos, Pos::new(3, 6));
        assert_eq!(g.snapshot().score, SCORE_PER_LINE);
    }

    #[test]
    fn consecutive_full_rows_all_disappear() {
        let mut g = game();
        g.robot.pos = Pos::new(3, 2);
        fill_row(&mut g, 11, 0..6);
        fill_row(&mut g, 10, 0..7);
        set_piece(&mut g, 6, 11, panel(PanelKind::Straight, Direction::North));
        g.handle_input(Input::SoftDrop);

        assert!(!g.grid.is_row_full(11));
        assert!(!g.grid.is_row_full(10));
        assert_eq!(g.robot.pos, Pos::new(3, 4));
        assert_eq!(g.snapshot().score, SCORE_PER_LINE * 2);
    }

    #[test]
    fn row_the_robot_stands_on_is_kept() {
        let mut g = game();
        g.robot.pos = Pos::new(3, 11);
        fill_row(&mut g, 11, 0..6);
        set_piece(&mut g, 6, 0, panel(PanelKind::Straight, Direction::North));
        g.handle_input(Input::HardDrop);

        assert!(g.grid.is_row_full(11));
        assert_eq!(g.robot.pos, Pos::new(3, 11));
        assert_eq!(g.snapshot().score, 0);
    }

    #[test]
    fn rows_are_kept_while_the_robot_is_still_at_the_station() {
        let mut g = game();
        fill_row(&mut g, 11, 0..6);
        set_piece(&mut g, 6, 0, panel(PanelKind::Straight, Direction::North));
        g.handle_input(Input::HardDrop);

        assert!(g.grid.is_row_full(11));
        assert_eq!(g.snapshot().score, 0);
    }

    #[test]
    fn full_row_above_robot_is_kept() {
        let mut g = game();
        g.robot.pos = Pos::new(3, 5);
        fill_row(&mut g, 3, 0..6);
        g.grid.set(Pos::new(6, 4), Cell::Wall);
        set_piece(&mut g, 6, 3, panel(PanelKind::Straight, Direction::North));
        g.handle_input(Input::SoftDrop);

        assert!(g.grid.is_row_full(3));
        assert_eq!(g.robot.pos, Pos::new(3, 5));
    }

    #[test]
    fn input_deserializes_from_snake_case() {
        let input: Input = serde_json::from_str("\"hard_drop\"").unwrap();
        assert_eq!(input, Input::HardDrop);
    }

    #[test]
    fn snapshot_serializes_for_the_ui() {
        let json = serde_json::to_value(game().snapshot()).unwrap();
        assert_eq!(json["status"], "playing");
        assert_eq!(json["robot"]["heading"], "north");
        assert_eq!(json["grid"]["width"], 7);
        assert_eq!(json["piece"]["panel"]["facing"], "north");
        assert_eq!(json["next"]["facing"], "north");
        assert_eq!(json["goal"]["y"], -1);
        assert_eq!(json["start"]["y"], 12);
    }
}
