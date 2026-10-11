use serde::Serialize;

use super::grid::{Cell, Grid};
use super::types::{Direction, Pos};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Robot {
    pub pos: Pos,
    pub heading: Direction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepResult {
    /// レールに進んだ。
    Moved,
    /// 正面が空きマスなので待機した。
    Waiting,
    /// ゴールに到達した。
    Reached,
    /// 壁・盤面外への衝突、またはレールの脱線。
    Crashed,
}

impl Robot {
    pub fn new(pos: Pos, heading: Direction) -> Self {
        Robot { pos, heading }
    }

    /// 1 ステップ進行する。`goal` は盤面の外にあるゴール駅の位置。
    pub fn step(&mut self, grid: &Grid, goal: Pos) -> StepResult {
        let target = self.pos.step(self.heading);
        if target == goal {
            self.pos = target;
            return StepResult::Reached;
        }
        match grid.get(target) {
            None | Some(Cell::Wall) => StepResult::Crashed,
            Some(Cell::Empty) => StepResult::Waiting,
            Some(Cell::Panel(panel)) => match panel.traverse(self.heading) {
                Some(heading) => {
                    self.pos = target;
                    self.heading = heading;
                    StepResult::Moved
                }
                None => StepResult::Crashed,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{Panel, PanelKind};

    /// 盤面 (3x3) の外、北側にあるゴール。
    const GOAL: Pos = Pos { x: 0, y: -1 };

    fn robot() -> Robot {
        Robot::new(Pos::new(0, 0), Direction::East)
    }

    fn panel_cell(kind: PanelKind, facing: Direction) -> Cell {
        Cell::Panel(Panel::new(kind, facing))
    }

    #[test]
    fn straight_rail_keeps_heading() {
        let mut g = Grid::new(3, 3);
        g.set(
            Pos::new(1, 0),
            panel_cell(PanelKind::Straight, Direction::East),
        );
        let mut r = robot();
        assert_eq!(r.step(&g, GOAL), StepResult::Moved);
        assert_eq!(r.pos, Pos::new(1, 0));
        assert_eq!(r.heading, Direction::East);
    }

    #[test]
    fn curve_rails_turn_the_train() {
        let mut g = Grid::new(3, 3);
        g.set(
            Pos::new(1, 0),
            panel_cell(PanelKind::TurnRight, Direction::East),
        );
        let mut r = robot();
        r.step(&g, GOAL);
        assert_eq!(r.heading, Direction::South);

        let mut g = Grid::new(3, 3);
        g.set(
            Pos::new(1, 0),
            panel_cell(PanelKind::TurnLeft, Direction::East),
        );
        let mut r = robot();
        r.step(&g, GOAL);
        assert_eq!(r.heading, Direction::North);
    }

    #[test]
    fn rails_can_be_ridden_from_the_exit_end() {
        let mut g = Grid::new(3, 3);
        g.set(
            Pos::new(1, 1),
            panel_cell(PanelKind::TurnRight, Direction::North),
        );
        let mut r = Robot::new(Pos::new(2, 1), Direction::West);
        assert_eq!(r.step(&g, GOAL), StepResult::Moved);
        assert_eq!(r.heading, Direction::South);
    }

    #[test]
    fn derails_when_entering_an_unconnected_side() {
        let mut g = Grid::new(3, 3);
        g.set(
            Pos::new(1, 0),
            panel_cell(PanelKind::Straight, Direction::North),
        );
        let mut r = robot();
        assert_eq!(r.step(&g, GOAL), StepResult::Crashed);
        assert_eq!(r.pos, Pos::new(0, 0));
    }

    #[test]
    fn two_rails_facing_each_other_do_not_ping_pong() {
        // 向かい合った曲線でも、機関車は引き返せない (往復ループにならない)。
        let mut g = Grid::new(3, 1);
        g.set(
            Pos::new(1, 0),
            panel_cell(PanelKind::TurnRight, Direction::East),
        );
        let mut r = Robot::new(Pos::new(0, 0), Direction::East);
        assert_eq!(r.step(&g, GOAL), StepResult::Moved);
        assert_eq!(r.heading, Direction::South);
        assert_eq!(r.step(&g, GOAL), StepResult::Crashed);
    }

    #[test]
    fn waits_in_front_of_empty_cell() {
        let g = Grid::new(3, 3);
        let mut r = robot();
        assert_eq!(r.step(&g, GOAL), StepResult::Waiting);
        assert_eq!(r.pos, Pos::new(0, 0));
    }

    #[test]
    fn reaches_goal_station_outside_the_board() {
        let g = Grid::new(3, 3);
        let mut r = Robot::new(Pos::new(0, 0), Direction::North);
        assert_eq!(r.step(&g, GOAL), StepResult::Reached);
        assert_eq!(r.pos, GOAL);
    }

    #[test]
    fn crashes_into_wall() {
        let mut g = Grid::new(3, 3);
        g.set(Pos::new(1, 0), Cell::Wall);
        let mut r = robot();
        assert_eq!(r.step(&g, GOAL), StepResult::Crashed);
        assert_eq!(r.pos, Pos::new(0, 0));
    }

    #[test]
    fn crashes_off_the_board_away_from_the_goal() {
        let g = Grid::new(3, 3);
        let mut r = Robot::new(Pos::new(2, 0), Direction::North);
        assert_eq!(r.step(&g, GOAL), StepResult::Crashed);
    }
}
