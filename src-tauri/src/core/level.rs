use super::grid::{Cell, Grid};
use super::robot::Robot;
use super::types::{Direction, Pos};

/// ステージ定義。固定の壁・ロボット開始位置・ゴールを持つ。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Level {
    pub width: usize,
    pub height: usize,
    pub walls: Vec<Pos>,
    pub start: Pos,
    pub start_heading: Direction,
    pub goal: Pos,
}

impl Level {
    /// 壁を配置した盤面を作る。スタートとゴールは盤面の外にある。
    pub fn build_grid(&self) -> Grid {
        let mut grid = Grid::new(self.width, self.height);
        for &wall in &self.walls {
            grid.set(wall, Cell::Wall);
        }
        grid
    }

    pub fn spawn_robot(&self) -> Robot {
        Robot::new(self.start, self.start_heading)
    }

    /// 落下ブロックの出現位置 (最上段の `x` 列)。
    pub fn spawn_pos(&self, x: i32) -> Pos {
        Pos::new(x, 0)
    }

    /// 盤面の下 (中央) の駅を出発し、盤面の上 (中央) のゴール駅を目指す最初のステージ。
    pub fn first_stage() -> Self {
        Level {
            width: 7,
            height: 12,
            walls: Vec::new(),
            start: Pos::new(3, 12),
            start_heading: Direction::North,
            goal: Pos::new(3, -1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::grid::Cell;

    #[test]
    fn stations_are_outside_the_field_at_center() {
        let level = Level::first_stage();
        let center = (level.width / 2) as i32;
        assert_eq!(level.start, Pos::new(center, level.height as i32));
        assert_eq!(level.goal, Pos::new(center, -1));
        let grid = level.build_grid();
        assert_eq!(grid.get(level.start), None);
        assert_eq!(grid.get(level.goal), None);
    }

    #[test]
    fn build_grid_is_empty_without_walls() {
        let grid = Level::first_stage().build_grid();
        assert_eq!(grid.width(), 7);
        assert_eq!(grid.height(), 12);
        assert_eq!(grid.get(Pos::new(3, 11)), Some(Cell::Empty));
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
    fn spawn_robot_uses_start_and_heading() {
        let robot = Level::first_stage().spawn_robot();
        assert_eq!(robot.pos, Pos::new(3, 12));
        assert_eq!(robot.heading, Direction::North);
    }

    #[test]
    fn spawn_pos_is_top_row_of_given_column() {
        assert_eq!(Level::first_stage().spawn_pos(5), Pos::new(5, 0));
    }
}
