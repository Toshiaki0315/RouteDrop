use serde::{Deserialize, Serialize};

use super::grid::{Cell, Grid};
use super::level::Level;
use super::piece::Piece;
use super::rng::Rng;
use super::train::{StepResult, Train};
use super::types::{Direction, Panel, Pos};

/// 落下ブロックの自然落下間隔 (ms)。
pub const FALL_INTERVAL_MS: u32 = 1000;
/// 列車の進行間隔 (ms)。
pub const TRAIN_INTERVAL_MS: u32 = 500;

const SCORE_PER_STEP: u32 = 10;
const SCORE_ON_STATION: u32 = 100;
const SCORE_PER_LINE: u32 = 50;
/// 最初の駅までの縦方向の距離 (始発駅から数えたマス)。
pub const FIRST_STATION_DISTANCE: i32 = 15;
/// 駅に着くたびに、次の駅までの距離が増える量 (マス)。
pub const STATION_DISTANCE_STEP: i32 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Playing,
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
    /// 次のパネルが出現する列 (目印の表示用)。
    pub next_column: i32,
    pub train: Train,
    /// 次の駅までの縦方向の距離 (マス)。駅が盤面の上にまだ見えていない間も数える。
    pub rows_to_station: i32,
    /// 到着した駅の数。
    pub stations_reached: u32,
    pub status: Status,
    pub score: u32,
}

#[derive(Clone, Debug)]
pub struct Game {
    level: Level,
    grid: Grid,
    piece: Option<Piece>,
    next: Panel,
    /// 次のパネルが出現する列。出現時に埋まっていたら空いている列から選び直す。
    next_column: i32,
    train: Train,
    /// 次に目指す駅。盤面の上にはみ出している間は `y < 0`。
    /// ライン消去で世界が 1 段下がるたびに `y` が 1 増え、盤面に入ると駅マスが現れる。
    station: Pos,
    stations_reached: u32,
    status: Status,
    score: u32,
    rng: Rng,
    fall_acc: u32,
    train_acc: u32,
}

impl Game {
    pub fn new(level: Level, seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let next = Panel::new(rng.next_panel(), Direction::North);
        let station = Pos::new(
            (rng.next_u32() % level.width as u32) as i32,
            level.start.y - FIRST_STATION_DISTANCE,
        );
        let next_column = (rng.next_u32() % level.width as u32) as i32;
        let mut game = Game {
            grid: level.build_grid(),
            train: level.spawn_train(),
            level,
            piece: None,
            next,
            next_column,
            station,
            stations_reached: 0,
            status: Status::Playing,
            score: 0,
            rng,
            fall_acc: 0,
            train_acc: 0,
        };
        game.place_station_if_visible();
        game.spawn_piece();
        game
    }

    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            grid: self.grid.clone(),
            piece: self.piece,
            next: self.next,
            next_column: self.next_column,
            train: self.train,
            rows_to_station: self.train.pos.y - self.station.y,
            stations_reached: self.stations_reached,
            status: self.status,
            score: self.score,
        }
    }

    pub fn status(&self) -> Status {
        self.status
    }

    /// 経過時間を与えて、落下と列車進行の Tick を必要回数だけ進める。
    pub fn advance(&mut self, dt_ms: u32) {
        if self.status != Status::Playing {
            return;
        }
        self.fall_acc += dt_ms;
        self.train_acc += dt_ms;
        while self.status == Status::Playing && self.train_acc >= TRAIN_INTERVAL_MS {
            self.train_acc -= TRAIN_INTERVAL_MS;
            self.tick_train();
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
            // 駅が降りてきて同じマスを占めていたら、パネルは消える。
            if self.grid.is_free(piece.pos) {
                self.grid.set(piece.pos, Cell::Panel(piece.panel));
            }
        }
        self.clear_lines();
        if self.status == Status::Playing {
            self.spawn_piece();
        }
    }

    /// 列車より下 (通過済み) の行が全部埋まったら消す。
    /// 列車の行とそれより上は経路に必要なので消さない。
    /// 消えた行より上にあるもの (列車・駅を含む) は 1 段下がる。
    /// 列車が最下段にいて、その行が全部埋まったら消せないのでゲームオーバー。
    fn clear_lines(&mut self) {
        let bottom = self.grid.height() as i32 - 1;
        let mut y = bottom;
        while y > self.train.pos.y {
            if self.grid.is_row_full(y as usize) {
                self.grid.remove_row(y as usize);
                self.train.pos.y += 1;
                self.station.y += 1;
                self.place_station_if_visible();
                self.score += SCORE_PER_LINE;
            } else {
                y -= 1;
            }
        }
        if self.train.pos.y == bottom && self.grid.is_row_full(bottom as usize) {
            self.status = Status::GameOver;
        }
    }

    /// 次のパネルを、最上段の空いている列からランダムに選んで出現させる。
    /// 最上段が全部埋まっていたらゲームオーバー。
    fn spawn_piece(&mut self) {
        let free: Vec<i32> = (0..self.level.width as i32)
            .filter(|&x| self.grid.is_free(self.level.spawn_pos(x)))
            .collect();
        if free.is_empty() {
            self.piece = None;
            self.status = Status::GameOver;
        } else {
            // 予告した列に出す。埋まっていたら、空いている列からランダムに選ぶ。
            let x = if free.contains(&self.next_column) {
                self.next_column
            } else {
                free[self.rng.next_u32() as usize % free.len()]
            };
            self.piece = Some(Piece::new(self.level.spawn_pos(x), self.next));
            self.next = Panel::new(self.rng.next_panel(), Direction::North);
            self.next_column = self.random_column();
        }
    }

    fn random_column(&mut self) -> i32 {
        (self.rng.next_u32() % self.level.width as u32) as i32
    }

    /// 目指す駅が盤面に入っていたら、駅マスを置く (すでにあれば何も変わらない)。
    fn place_station_if_visible(&mut self) {
        if self.station.y >= 0 {
            self.grid.set(self.station, Cell::Station);
        }
    }

    /// 駅に着いた。次の駅は、ここから縦に 20, 25, 30, … マス先 (列はランダム)。
    fn reach_station(&mut self) {
        self.score += SCORE_ON_STATION;
        self.stations_reached += 1;
        let distance =
            FIRST_STATION_DISTANCE + STATION_DISTANCE_STEP * self.stations_reached as i32;
        let x = self.random_column();
        self.station = Pos::new(x, self.station.y - distance);
        self.place_station_if_visible();
    }

    fn tick_train(&mut self) {
        match self.train.step(&self.grid) {
            StepResult::Moved => self.score += SCORE_PER_STEP,
            StepResult::Waiting => {}
            StepResult::Arrived => {
                self.score += SCORE_PER_STEP;
                if self.train.pos == self.station {
                    self.reach_station();
                }
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
    fn new_game_starts_on_the_origin_station() {
        let g = game();
        let s = g.snapshot();
        assert_eq!(s.status, Status::Playing);
        assert_eq!(s.score, 0);
        assert_eq!(s.train.pos, Pos::new(3, 11));
        assert_eq!(s.train.heading, Direction::North);
        assert_eq!(s.grid.get(Pos::new(3, 11)), Some(Cell::Origin));
        assert_eq!(s.stations_reached, 0);
        assert_eq!(s.rows_to_station, FIRST_STATION_DISTANCE);
        assert_eq!(g.station.y, 11 - FIRST_STATION_DISTANCE);
        assert!((0..7).contains(&g.station.x));
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
    fn next_piece_spawns_in_the_announced_column() {
        let mut g = game();
        for _ in 0..8 {
            let announced = g.snapshot().next_column;
            assert!((0..7).contains(&announced));
            g.handle_input(Input::HardDrop);
            assert_eq!(g.piece.unwrap().pos.x, announced);
        }
    }

    #[test]
    fn announced_column_changes_between_pieces() {
        let mut g = game();
        let mut columns = std::collections::HashSet::new();
        for _ in 0..20 {
            columns.insert(g.snapshot().next_column);
            g.handle_input(Input::HardDrop);
        }
        assert!(columns.len() > 1);
    }

    #[test]
    fn blocked_announced_column_falls_back_to_a_free_one() {
        let mut g = game();
        let announced = g.snapshot().next_column;
        g.grid.set(Pos::new(announced, 0), straight_north());
        g.piece = None;
        g.spawn_piece();
        assert_eq!(g.status(), Status::Playing);
        assert_ne!(g.piece.unwrap().pos.x, announced);
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
        set_straight_at(&mut g, 2);
        g.handle_input(Input::HardDrop);
        assert_eq!(g.grid.get(Pos::new(2, 11)), Some(straight_north()));
        set_straight_at(&mut g, 2);
        g.handle_input(Input::HardDrop);
        assert_eq!(g.grid.get(Pos::new(2, 10)), Some(straight_north()));
    }

    #[test]
    fn pieces_stack_on_top_of_the_origin_station() {
        let mut g = game();
        set_straight_at(&mut g, 3);
        g.handle_input(Input::HardDrop);
        assert_eq!(g.grid.get(Pos::new(3, 10)), Some(straight_north()));
        assert_eq!(g.grid.get(Pos::new(3, 11)), Some(Cell::Origin));
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
    fn train_leaves_the_origin_every_half_second() {
        let mut g = game();
        g.grid.set(Pos::new(3, 10), straight_north());
        g.advance(TRAIN_INTERVAL_MS - 1);
        assert_eq!(g.train.pos, Pos::new(3, 11));
        g.advance(1);
        assert_eq!(g.train.pos, Pos::new(3, 10));
        assert_eq!(g.snapshot().score, SCORE_PER_STEP);
    }

    #[test]
    fn train_waits_on_the_origin_when_path_is_empty() {
        let mut g = game();
        g.advance(TRAIN_INTERVAL_MS * 4);
        assert_eq!(g.train.pos, Pos::new(3, 11));
        assert_eq!(g.status(), Status::Playing);
    }

    #[test]
    fn empty_cell_ahead_is_not_game_over_a_rail_may_still_fall() {
        let mut g = game();
        g.grid.set(Pos::new(3, 10), straight_north()); // その先 (3,9) は空き
        g.advance(TRAIN_INTERVAL_MS * 4);
        assert_eq!(g.train.pos, Pos::new(3, 10));
        assert_eq!(g.status(), Status::Playing);
    }

    #[test]
    fn unconnected_rail_ahead_is_game_over() {
        let mut g = game();
        // 東西につながる直進レール: 北へ進む列車は入れない
        g.grid.set(
            Pos::new(3, 10),
            Cell::Panel(panel(PanelKind::Straight, Direction::East)),
        );
        g.advance(TRAIN_INTERVAL_MS);
        assert_eq!(g.status(), Status::GameOver);
    }

    #[test]
    fn train_crash_is_game_over() {
        let mut g = game();
        g.grid.set(Pos::new(3, 10), Cell::Wall);
        g.advance(TRAIN_INTERVAL_MS);
        assert_eq!(g.status(), Status::GameOver);
    }

    #[test]
    fn train_arriving_at_the_station_scores_and_aims_for_the_next_one() {
        let mut g = game();
        for y in 1..=10 {
            g.grid.set(Pos::new(3, y), straight_north());
        }
        g.grid.set(Pos::new(3, 0), Cell::Station);
        g.station = Pos::new(3, 0);
        g.piece = None; // 落下ブロックが経路に干渉しないようにする
        g.advance(TRAIN_INTERVAL_MS * 11);

        assert_eq!(g.status(), Status::Playing);
        assert_eq!(g.train.pos, Pos::new(3, 0));
        let s = g.snapshot();
        assert_eq!(s.stations_reached, 1);
        assert_eq!(s.score, SCORE_PER_STEP * 11 + SCORE_ON_STATION);
        // 次の駅は縦に 20 マス先 (盤面の上)。
        assert_eq!(
            g.station.y,
            -(FIRST_STATION_DISTANCE + STATION_DISTANCE_STEP)
        );
        assert_eq!(
            s.rows_to_station,
            FIRST_STATION_DISTANCE + STATION_DISTANCE_STEP
        );
        // 着いた駅はそのまま残り、列車は通り抜けられる。
        assert_eq!(g.grid.get(Pos::new(3, 0)), Some(Cell::Station));
    }

    #[test]
    fn each_next_station_is_five_rows_farther() {
        let mut g = game();
        g.station = Pos::new(3, 0);
        let mut ys = Vec::new();
        for _ in 0..3 {
            g.reach_station();
            ys.push(g.station.y);
        }
        // 距離 20, 25, 30 と、駅の位置から数えて遠くなっていく。
        assert_eq!(ys, vec![-20, -45, -75]);
        assert_eq!(g.snapshot().stations_reached, 3);
    }

    #[test]
    fn next_station_column_is_random() {
        let mut columns = std::collections::HashSet::new();
        for seed in 0..40 {
            let mut g = Game::new(Level::first_stage(), seed);
            g.reach_station();
            columns.insert(g.station.x);
        }
        assert!(columns.len() > 1);
    }

    #[test]
    fn train_passes_through_a_station_that_is_not_the_target() {
        let mut g = game();
        g.train.pos = Pos::new(3, 5);
        g.grid.set(Pos::new(3, 4), Cell::Station); // 通過済みの駅など
        g.advance(TRAIN_INTERVAL_MS);
        assert_eq!(g.train.pos, Pos::new(3, 4));
        assert_eq!(g.train.heading, Direction::North);
        assert_eq!(g.snapshot().stations_reached, 0);
        assert_eq!(g.snapshot().score, SCORE_PER_STEP);
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
    fn station_in_the_top_row_does_not_cause_game_over() {
        // 駅が最上段にあっても、その列を避けて出現する (駅のせいで終わらない)。
        for seed in 0..50 {
            let mut g = Game::new(Level::first_stage(), seed);
            g.grid.set(Pos::new(3, 0), Cell::Station);
            g.station = Pos::new(3, 0);
            for _ in 0..10 {
                g.piece = None;
                g.spawn_piece();
                assert_eq!(g.status(), Status::Playing, "seed {seed}");
                assert_ne!(g.piece.unwrap().pos.x, 3);
            }
        }
    }

    #[test]
    fn piece_spawns_in_the_only_free_column() {
        let mut g = game();
        for x in 0..7 {
            if x != 5 {
                g.grid.set(Pos::new(x, 0), straight_north());
            }
        }
        g.piece = None;
        g.spawn_piece();
        assert_eq!(g.piece.unwrap().pos, Pos::new(5, 0));
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
        g.grid.set(Pos::new(3, 10), Cell::Wall);
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
    fn full_row_below_train_disappears_and_things_above_drop() {
        let mut g = game();
        g.train.pos = Pos::new(3, 5); // 行 11 は列車より下 (通過済み)
        fill_row(&mut g, 11, 0..6);
        g.grid.set(Pos::new(1, 10), straight_north());
        set_piece(&mut g, 6, 0, panel(PanelKind::Straight, Direction::North));
        g.handle_input(Input::HardDrop);

        assert_eq!(g.grid.get(Pos::new(1, 11)), Some(straight_north()));
        assert_eq!(g.grid.get(Pos::new(1, 10)), Some(Cell::Empty));
        assert_eq!(g.grid.get(Pos::new(6, 11)), Some(Cell::Empty));
        assert_eq!(g.train.pos, Pos::new(3, 6));
        assert_eq!(g.snapshot().score, SCORE_PER_LINE);
    }

    #[test]
    fn consecutive_full_rows_all_disappear() {
        let mut g = game();
        g.train.pos = Pos::new(3, 2);
        fill_row(&mut g, 11, 0..6);
        fill_row(&mut g, 10, 0..7);
        set_piece(&mut g, 6, 11, panel(PanelKind::Straight, Direction::North));
        g.handle_input(Input::SoftDrop);

        assert!(!g.grid.is_row_full(11));
        assert!(!g.grid.is_row_full(10));
        assert_eq!(g.train.pos, Pos::new(3, 4));
        assert_eq!(g.snapshot().score, SCORE_PER_LINE * 2);
    }

    #[test]
    fn row_the_train_stands_on_is_kept() {
        let mut g = game();
        g.train.pos = Pos::new(3, 10);
        g.grid.set(Pos::new(6, 11), straight_north()); // 行 11 は埋まっていない
        fill_row(&mut g, 10, 0..6);
        set_piece(&mut g, 6, 10, panel(PanelKind::Straight, Direction::North));
        g.handle_input(Input::SoftDrop);

        assert!(g.grid.is_row_full(10));
        assert_eq!(g.train.pos, Pos::new(3, 10));
        assert_eq!(g.status(), Status::Playing);
        assert_eq!(g.snapshot().score, 0);
    }

    #[test]
    fn full_bottom_row_under_the_train_is_game_over() {
        let mut g = game();
        g.train.pos = Pos::new(3, 11); // 列車が最下段にいる
        fill_row(&mut g, 11, 0..6);
        set_piece(&mut g, 6, 0, panel(PanelKind::Straight, Direction::North));
        g.handle_input(Input::HardDrop);

        assert!(g.grid.is_row_full(11));
        assert_eq!(g.status(), Status::GameOver);
    }

    #[test]
    fn origin_station_row_clears_once_the_train_has_left() {
        let mut g = game();
        g.train.pos = Pos::new(3, 8); // 列車は始発駅を出て先へ進んでいる
        fill_row(&mut g, 11, 0..3);
        fill_row(&mut g, 11, 4..6);
        set_piece(&mut g, 6, 0, panel(PanelKind::Straight, Direction::North));
        g.handle_input(Input::HardDrop);

        // 始発駅 (3,11) も埋まりとして数えられ、行ごと消える。
        assert_eq!(g.grid.get(Pos::new(3, 11)), Some(Cell::Empty));
        assert_eq!(g.snapshot().score, SCORE_PER_LINE);
        assert_eq!(g.train.pos, Pos::new(3, 9));
    }

    #[test]
    fn full_row_above_train_is_kept() {
        let mut g = game();
        g.train.pos = Pos::new(3, 5);
        fill_row(&mut g, 3, 0..6);
        g.grid.set(Pos::new(6, 4), Cell::Wall);
        set_piece(&mut g, 6, 3, panel(PanelKind::Straight, Direction::North));
        g.handle_input(Input::SoftDrop);

        assert!(g.grid.is_row_full(3));
        assert_eq!(g.train.pos, Pos::new(3, 5));
    }

    /// 列車より下の 1 行を完成させて消す。
    fn clear_one_line(g: &mut Game) {
        g.train.pos = Pos::new(3, 5);
        fill_row(g, 11, 0..6);
        set_piece(g, 6, 0, panel(PanelKind::Straight, Direction::North));
        g.handle_input(Input::HardDrop);
    }

    #[test]
    fn station_scrolls_into_the_field_as_lines_are_cleared() {
        let mut g = game();
        g.station = Pos::new(2, -3);
        for expected_y in [-2, -1] {
            clear_one_line(&mut g);
            assert_eq!(g.station.y, expected_y);
            assert!(!(0..12).any(|y| g.grid.get(Pos::new(2, y)) == Some(Cell::Station)));
        }
        clear_one_line(&mut g);
        assert_eq!(g.station, Pos::new(2, 0));
        assert_eq!(g.grid.get(Pos::new(2, 0)), Some(Cell::Station));
        assert_eq!(g.snapshot().rows_to_station, 6); // 列車は消去で 1 段下がって y=6
    }

    #[test]
    fn visible_station_keeps_descending_one_row_per_cleared_line() {
        let mut g = game();
        g.station = Pos::new(2, 0);
        g.grid.set(Pos::new(2, 0), Cell::Station);

        clear_one_line(&mut g);
        assert_eq!(g.station, Pos::new(2, 1));
        assert_eq!(g.grid.get(Pos::new(2, 0)), Some(Cell::Empty));
        assert_eq!(g.grid.get(Pos::new(2, 1)), Some(Cell::Station));

        clear_one_line(&mut g);
        assert_eq!(g.station, Pos::new(2, 2));
        assert_eq!(g.grid.get(Pos::new(2, 2)), Some(Cell::Station));
    }

    #[test]
    fn piece_sharing_a_cell_with_the_station_vanishes_instead_of_erasing_it() {
        let mut g = game();
        g.grid.set(Pos::new(3, 11), Cell::Station);
        set_piece(&mut g, 3, 11, panel(PanelKind::Straight, Direction::North));
        g.handle_input(Input::SoftDrop);
        assert_eq!(g.grid.get(Pos::new(3, 11)), Some(Cell::Station));
        assert!(g.piece.is_some());
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
        assert_eq!(json["train"]["heading"], "north");
        assert_eq!(json["grid"]["width"], 7);
        assert_eq!(json["piece"]["panel"]["facing"], "north");
        assert_eq!(json["next"]["facing"], "north");
        assert_eq!(json["rows_to_station"], FIRST_STATION_DISTANCE);
        assert_eq!(json["stations_reached"], 0);
        assert!(json["next_column"].is_i64());
        assert_eq!(json["grid"]["cells"][11 * 7 + 3], "origin");
    }
}
