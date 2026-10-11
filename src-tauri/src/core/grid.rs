use serde::Serialize;

use super::types::{Panel, Pos};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Cell {
    Empty,
    Wall,
    Panel(Panel),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Grid {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        Grid {
            width,
            height,
            cells: vec![Cell::Empty; width * height],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    fn index(&self, pos: Pos) -> Option<usize> {
        if pos.x < 0 || pos.y < 0 {
            return None;
        }
        let (x, y) = (pos.x as usize, pos.y as usize);
        if x >= self.width || y >= self.height {
            return None;
        }
        Some(y * self.width + x)
    }

    /// 範囲外は `None`。
    pub fn get(&self, pos: Pos) -> Option<Cell> {
        self.index(pos).map(|i| self.cells[i])
    }

    /// 範囲外なら何もせず `false`。
    pub fn set(&mut self, pos: Pos, cell: Cell) -> bool {
        match self.index(pos) {
            Some(i) => {
                self.cells[i] = cell;
                true
            }
            None => false,
        }
    }

    /// 行がすべてパネルで埋まっているか (壁・空きを含む行は false)。
    pub fn is_row_full(&self, y: usize) -> bool {
        (0..self.width).all(|x| matches!(self.cells[y * self.width + x], Cell::Panel(_)))
    }

    /// 行 `y` を消し、それより上のパネルを 1 段下げる。壁は動かない。
    pub fn remove_row(&mut self, y: usize) {
        for row in (0..=y).rev() {
            for x in 0..self.width {
                let i = row * self.width + x;
                if self.cells[i] == Cell::Wall {
                    continue;
                }
                self.cells[i] = if row == 0 {
                    Cell::Empty
                } else {
                    match self.cells[i - self.width] {
                        Cell::Panel(panel) => Cell::Panel(panel),
                        _ => Cell::Empty,
                    }
                };
            }
        }
    }

    /// 範囲内かつ空きマスか。落下ブロックが進入できる条件。
    pub fn is_free(&self, pos: Pos) -> bool {
        self.get(pos) == Some(Cell::Empty)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{Direction, PanelKind};

    #[test]
    fn new_grid_is_empty() {
        let g = Grid::new(3, 2);
        assert_eq!(g.width(), 3);
        assert_eq!(g.height(), 2);
        assert_eq!(g.get(Pos::new(2, 1)), Some(Cell::Empty));
    }

    #[test]
    fn out_of_bounds_get_is_none() {
        let g = Grid::new(3, 2);
        assert_eq!(g.get(Pos::new(-1, 0)), None);
        assert_eq!(g.get(Pos::new(0, -1)), None);
        assert_eq!(g.get(Pos::new(3, 0)), None);
        assert_eq!(g.get(Pos::new(0, 2)), None);
    }

    #[test]
    fn set_then_get() {
        let mut g = Grid::new(3, 2);
        assert!(g.set(Pos::new(1, 1), Cell::Wall));
        assert_eq!(g.get(Pos::new(1, 1)), Some(Cell::Wall));
        assert_eq!(g.get(Pos::new(1, 0)), Some(Cell::Empty));
    }

    #[test]
    fn set_out_of_bounds_returns_false() {
        let mut g = Grid::new(3, 2);
        assert!(!g.set(Pos::new(3, 0), Cell::Wall));
    }

    #[test]
    fn is_free_only_for_empty_in_bounds() {
        let mut g = Grid::new(3, 2);
        g.set(Pos::new(0, 0), Cell::Wall);
        g.set(
            Pos::new(1, 0),
            Cell::Panel(Panel::new(PanelKind::Straight, Direction::North)),
        );
        assert!(!g.is_free(Pos::new(0, 0)));
        assert!(!g.is_free(Pos::new(1, 0)));
        assert!(g.is_free(Pos::new(0, 1)));
        assert!(!g.is_free(Pos::new(-1, 0)));
    }

    fn panel(kind: PanelKind) -> Cell {
        Cell::Panel(Panel::new(kind, Direction::North))
    }

    #[test]
    fn is_row_full_requires_every_cell_to_be_a_panel() {
        let mut g = Grid::new(3, 2);
        assert!(!g.is_row_full(0));
        for x in 0..3 {
            g.set(Pos::new(x, 0), panel(PanelKind::Straight));
        }
        assert!(g.is_row_full(0));
        assert!(!g.is_row_full(1));
        g.set(Pos::new(1, 0), Cell::Wall);
        assert!(!g.is_row_full(0));
    }

    #[test]
    fn remove_row_shifts_panels_down_and_keeps_fixed_cells() {
        let mut g = Grid::new(3, 3);
        for x in 0..3 {
            g.set(Pos::new(x, 2), panel(PanelKind::Straight));
        }
        g.set(Pos::new(0, 1), panel(PanelKind::TurnLeft));
        g.set(Pos::new(1, 1), Cell::Wall);
        g.set(Pos::new(1, 0), panel(PanelKind::TurnRight));

        g.remove_row(2);

        // 上のパネルが 1 段落ちる。壁の下へ落ちる分は壁があるので消える。
        assert_eq!(g.get(Pos::new(0, 2)), Some(panel(PanelKind::TurnLeft)));
        assert_eq!(g.get(Pos::new(1, 2)), Some(Cell::Empty));
        assert_eq!(g.get(Pos::new(2, 2)), Some(Cell::Empty));
        // 壁は動かない。最上段は空になる。
        assert_eq!(g.get(Pos::new(1, 1)), Some(Cell::Wall));
        assert_eq!(g.get(Pos::new(0, 1)), Some(Cell::Empty));
        assert_eq!(g.get(Pos::new(1, 0)), Some(Cell::Empty));
    }
}
