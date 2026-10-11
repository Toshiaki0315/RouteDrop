use super::grid::{Cell, Grid};
use super::train::Train;
use super::types::{Direction, Pos};

/// ステージ定義。固定の壁と列車の開始位置を持つ。駅は実行中に出現する。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Level {
    pub width: usize,
    pub height: usize,
    pub walls: Vec<Pos>,
    pub start: Pos,
    pub start_heading: Direction,
}

impl Level {
    /// 壁と始発駅を配置した盤面を作る。
    pub fn build_grid(&self) -> Grid {
        let mut grid = Grid::new(self.width, self.height);
        for &wall in &self.walls {
            grid.set(wall, Cell::Wall);
        }
        grid.set(self.start, Cell::Origin);
        grid
    }

    pub fn spawn_train(&self) -> Train {
        Train::new(self.start, self.start_heading)
    }

    /// 落下ブロックの出現位置 (最上段の `x` 列)。
    pub fn spawn_pos(&self, x: i32) -> Pos {
        Pos::new(x, 0)
    }

    /// 盤面の最下段中央の始発駅から出発する最初のステージ。
    pub fn first_stage() -> Self {
        Level {
            width: 7,
            height: 12,
            walls: Vec::new(),
            start: Pos::new(3, 11),
            start_heading: Direction::North,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::grid::Cell;

    #[test]
    fn origin_station_is_on_the_field_at_bottom_center() {
        let level = Level::first_stage();
        assert_eq!(level.start, Pos::new(3, 11));
        assert_eq!(level.start.x, (level.width / 2) as i32);
        assert_eq!(level.start.y, level.height as i32 - 1);
        assert_eq!(level.build_grid().get(level.start), Some(Cell::Origin));
    }

    #[test]
    fn build_grid_has_no_walls_by_default() {
        let grid = Level::first_stage().build_grid();
        assert_eq!(grid.width(), 7);
        assert_eq!(grid.height(), 12);
        assert_eq!(grid.get(Pos::new(3, 10)), Some(Cell::Empty));
    }

    #[test]
    fn build_grid_places_walls() {
        let level = Level {
            walls: vec![Pos::new(1, 1)],
            ..Level::first_stage()
        };
        assert_eq!(level.build_grid().get(Pos::new(1, 1)), Some(Cell::Wall));
    }

    #[test]
    fn spawn_train_uses_start_and_heading() {
        let train = Level::first_stage().spawn_train();
        assert_eq!(train.pos, Pos::new(3, 11));
        assert_eq!(train.heading, Direction::North);
    }

    #[test]
    fn spawn_pos_is_top_row_of_given_column() {
        assert_eq!(Level::first_stage().spawn_pos(5), Pos::new(5, 0));
    }
}
