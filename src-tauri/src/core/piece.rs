use serde::Serialize;

use super::grid::Grid;
use super::types::{Panel, Pos};

/// 落下中のパネル。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Piece {
    pub pos: Pos,
    pub panel: Panel,
}

impl Piece {
    pub fn new(pos: Pos, panel: Panel) -> Self {
        Piece { pos, panel }
    }

    /// 移動先が空きなら移動して `true`。
    pub fn try_move(&mut self, grid: &Grid, dx: i32, dy: i32) -> bool {
        let target = Pos::new(self.pos.x + dx, self.pos.y + dy);
        if grid.is_free(target) {
            self.pos = target;
            true
        } else {
            false
        }
    }

    /// 時計回りに 90 度回転する。
    pub fn rotate(&mut self) {
        self.panel = self.panel.rotated();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::grid::Cell;
    use crate::core::types::{Direction, PanelKind};

    fn straight() -> Panel {
        Panel::new(PanelKind::Straight, Direction::North)
    }

    fn piece() -> Piece {
        Piece::new(Pos::new(1, 1), straight())
    }

    #[test]
    fn moves_into_free_cell() {
        let g = Grid::new(3, 3);
        let mut p = piece();
        assert!(p.try_move(&g, 1, 0));
        assert_eq!(p.pos, Pos::new(2, 1));
    }

    #[test]
    fn blocked_by_wall() {
        let mut g = Grid::new(3, 3);
        g.set(Pos::new(0, 1), Cell::Wall);
        let mut p = piece();
        assert!(!p.try_move(&g, -1, 0));
        assert_eq!(p.pos, Pos::new(1, 1));
    }

    #[test]
    fn blocked_by_bounds() {
        let g = Grid::new(3, 3);
        let mut p = Piece::new(Pos::new(1, 2), straight());
        assert!(!p.try_move(&g, 0, 1));
    }

    #[test]
    fn rotate_turns_panel_clockwise() {
        let mut p = piece();
        p.rotate();
        assert_eq!(p.panel, Panel::new(PanelKind::Straight, Direction::East));
    }
}
