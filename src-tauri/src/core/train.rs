use serde::Serialize;

use super::grid::{Cell, Grid};
use super::types::{Direction, Pos};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Train {
    pub pos: Pos,
    pub heading: Direction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepResult {
    /// レールに進んだ。
    Moved,
    /// 正面が空きマスなので待機した (つながるレールが落ちてくるかもしれない)。
    Waiting,
    /// 駅に到着した (向きは変えず、そのまま通り抜ける)。
    Arrived,
    /// 壁・盤面外への衝突、またはつながっていないレールへの進入 (脱線)。
    Crashed,
}

impl Train {
    pub fn new(pos: Pos, heading: Direction) -> Self {
        Train { pos, heading }
    }

    /// 1 ステップ進行する。
    pub fn step(&mut self, grid: &Grid) -> StepResult {
        let target = self.pos.step(self.heading);
        match grid.get(target) {
            None | Some(Cell::Wall) => StepResult::Crashed,
            Some(Cell::Empty) => StepResult::Waiting,
            Some(Cell::Station | Cell::Origin) => {
                self.pos = target;
                StepResult::Arrived
            }
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

    fn train() -> Train {
        Train::new(Pos::new(0, 0), Direction::East)
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
        let mut r = train();
        assert_eq!(r.step(&g), StepResult::Moved);
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
        let mut r = train();
        r.step(&g);
        assert_eq!(r.heading, Direction::South);

        let mut g = Grid::new(3, 3);
        g.set(
            Pos::new(1, 0),
            panel_cell(PanelKind::TurnLeft, Direction::East),
        );
        let mut r = train();
        r.step(&g);
        assert_eq!(r.heading, Direction::North);
    }

    #[test]
    fn rails_can_be_ridden_from_the_exit_end() {
        let mut g = Grid::new(3, 3);
        g.set(
            Pos::new(1, 1),
            panel_cell(PanelKind::TurnRight, Direction::North),
        );
        let mut r = Train::new(Pos::new(2, 1), Direction::West);
        assert_eq!(r.step(&g), StepResult::Moved);
        assert_eq!(r.heading, Direction::South);
    }

    #[test]
    fn derails_when_entering_an_unconnected_side() {
        let mut g = Grid::new(3, 3);
        g.set(
            Pos::new(1, 0),
            panel_cell(PanelKind::Straight, Direction::North),
        );
        let mut r = train();
        assert_eq!(r.step(&g), StepResult::Crashed);
        assert_eq!(r.pos, Pos::new(0, 0));
    }

    #[test]
    fn two_rails_facing_each_other_do_not_ping_pong() {
        // 向かい合った曲線でも、列車は引き返せない (往復ループにならない)。
        let mut g = Grid::new(3, 1);
        g.set(
            Pos::new(1, 0),
            panel_cell(PanelKind::TurnRight, Direction::East),
        );
        let mut r = Train::new(Pos::new(0, 0), Direction::East);
        assert_eq!(r.step(&g), StepResult::Moved);
        assert_eq!(r.heading, Direction::South);
        assert_eq!(r.step(&g), StepResult::Crashed);
    }

    #[test]
    fn waits_in_front_of_empty_cell() {
        let g = Grid::new(3, 3);
        let mut r = train();
        assert_eq!(r.step(&g), StepResult::Waiting);
        assert_eq!(r.pos, Pos::new(0, 0));
    }

    #[test]
    fn arrives_at_a_station_and_keeps_its_heading() {
        let mut g = Grid::new(3, 3);
        g.set(Pos::new(1, 0), Cell::Station);
        let mut r = train();
        assert_eq!(r.step(&g), StepResult::Arrived);
        assert_eq!(r.pos, Pos::new(1, 0));
        assert_eq!(r.heading, Direction::East);
    }

    #[test]
    fn passes_through_the_origin_station_too() {
        let mut g = Grid::new(3, 3);
        g.set(Pos::new(1, 0), Cell::Origin);
        let mut r = train();
        assert_eq!(r.step(&g), StepResult::Arrived);
        assert_eq!(r.heading, Direction::East);
    }

    #[test]
    fn crashes_into_wall() {
        let mut g = Grid::new(3, 3);
        g.set(Pos::new(1, 0), Cell::Wall);
        let mut r = train();
        assert_eq!(r.step(&g), StepResult::Crashed);
        assert_eq!(r.pos, Pos::new(0, 0));
    }

    #[test]
    fn crashes_off_the_board() {
        let g = Grid::new(3, 3);
        let mut r = Train::new(Pos::new(0, 0), Direction::North);
        assert_eq!(r.step(&g), StepResult::Crashed);
    }
}
