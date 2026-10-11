use serde::Serialize;

/// 方角。座標系は左上原点で、北が y 減少方向。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    pub fn turn_right(self) -> Self {
        match self {
            Direction::North => Direction::East,
            Direction::East => Direction::South,
            Direction::South => Direction::West,
            Direction::West => Direction::North,
        }
    }

    pub fn turn_left(self) -> Self {
        match self {
            Direction::North => Direction::West,
            Direction::West => Direction::South,
            Direction::South => Direction::East,
            Direction::East => Direction::North,
        }
    }

    pub fn opposite(self) -> Self {
        self.turn_right().turn_right()
    }

    /// (dx, dy)
    pub fn delta(self) -> (i32, i32) {
        match self {
            Direction::North => (0, -1),
            Direction::East => (1, 0),
            Direction::South => (0, 1),
            Direction::West => (-1, 0),
        }
    }
}

/// パネルの種類。列車が乗った時の進路変更を決める。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PanelKind {
    Straight,
    TurnRight,
    TurnLeft,
}

impl PanelKind {
    /// パネルの基準方向 `facing` から見た、矢印の指す方向。
    pub fn apply(self, facing: Direction) -> Direction {
        match self {
            PanelKind::Straight => facing,
            PanelKind::TurnRight => facing.turn_right(),
            PanelKind::TurnLeft => facing.turn_left(),
        }
    }
}

/// 種類と向きを持つパネル。列車は乗ると矢印の指す方向へ進む。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Panel {
    pub kind: PanelKind,
    pub facing: Direction,
}

impl Panel {
    pub fn new(kind: PanelKind, facing: Direction) -> Self {
        Panel { kind, facing }
    }

    /// 時計回りに 90 度回転したパネル。
    pub fn rotated(self) -> Self {
        Panel::new(self.kind, self.facing.turn_right())
    }

    /// レールの出口側の辺 (facing 北の直進なら北、右折なら東)。
    pub fn exit_direction(self) -> Direction {
        self.kind.apply(self.facing)
    }

    /// 向き `heading` で進んできた列車が、このレールを通った後の向き。
    /// レールは 2 つの端 (入口側 = facing の反対、出口側) をつなぎ、
    /// どちらの端から入っても反対の端へ抜ける。つながっていない辺から入ると `None` (脱線)。
    pub fn traverse(self, heading: Direction) -> Option<Direction> {
        let entered_from = heading.opposite();
        let (entry, exit) = (self.facing.opposite(), self.exit_direction());
        if entered_from == entry {
            Some(exit)
        } else if entered_from == exit {
            Some(entry)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl Pos {
    pub fn new(x: i32, y: i32) -> Self {
        Pos { x, y }
    }

    pub fn step(self, dir: Direction) -> Self {
        let (dx, dy) = dir.delta();
        Pos::new(self.x + dx, self.y + dy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [Direction; 4] = [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ];

    #[test]
    fn turn_right_cycles_clockwise() {
        assert_eq!(Direction::North.turn_right(), Direction::East);
        assert_eq!(Direction::East.turn_right(), Direction::South);
        assert_eq!(Direction::South.turn_right(), Direction::West);
        assert_eq!(Direction::West.turn_right(), Direction::North);
    }

    #[test]
    fn turn_left_cycles_counter_clockwise() {
        assert_eq!(Direction::North.turn_left(), Direction::West);
        assert_eq!(Direction::West.turn_left(), Direction::South);
        assert_eq!(Direction::South.turn_left(), Direction::East);
        assert_eq!(Direction::East.turn_left(), Direction::North);
    }

    #[test]
    fn opposite_flips_the_direction() {
        assert_eq!(Direction::North.opposite(), Direction::South);
        assert_eq!(Direction::East.opposite(), Direction::West);
        assert_eq!(Direction::South.opposite(), Direction::North);
        assert_eq!(Direction::West.opposite(), Direction::East);
    }

    #[test]
    fn left_and_right_are_inverse() {
        for d in ALL {
            assert_eq!(d.turn_right().turn_left(), d);
        }
    }

    #[test]
    fn delta_matches_screen_coordinates() {
        assert_eq!(Direction::North.delta(), (0, -1));
        assert_eq!(Direction::East.delta(), (1, 0));
        assert_eq!(Direction::South.delta(), (0, 1));
        assert_eq!(Direction::West.delta(), (-1, 0));
    }

    #[test]
    fn panel_kind_apply_gives_arrow_direction() {
        assert_eq!(
            PanelKind::Straight.apply(Direction::North),
            Direction::North
        );
        assert_eq!(
            PanelKind::TurnRight.apply(Direction::North),
            Direction::East
        );
        assert_eq!(PanelKind::TurnLeft.apply(Direction::North), Direction::West);
    }

    #[test]
    fn panel_rotates_clockwise_keeping_kind() {
        let p = Panel::new(PanelKind::TurnRight, Direction::North).rotated();
        assert_eq!(p, Panel::new(PanelKind::TurnRight, Direction::East));
    }

    #[test]
    fn four_rotations_return_to_start() {
        let p = Panel::new(PanelKind::TurnLeft, Direction::South);
        assert_eq!(p.rotated().rotated().rotated().rotated(), p);
    }

    #[test]
    fn exit_direction_follows_kind_and_facing() {
        let exit = |k, f| Panel::new(k, f).exit_direction();
        assert_eq!(exit(PanelKind::Straight, Direction::West), Direction::West);
        assert_eq!(
            exit(PanelKind::TurnRight, Direction::West),
            Direction::North
        );
        assert_eq!(exit(PanelKind::TurnLeft, Direction::West), Direction::South);
    }

    #[test]
    fn pos_step_moves_one_cell() {
        let p = Pos::new(3, 4);
        assert_eq!(p.step(Direction::North), Pos::new(3, 3));
        assert_eq!(p.step(Direction::East), Pos::new(4, 4));
        assert_eq!(p.step(Direction::South), Pos::new(3, 5));
        assert_eq!(p.step(Direction::West), Pos::new(2, 4));
    }

    #[test]
    fn straight_rail_connects_opposite_sides() {
        let p = Panel::new(PanelKind::Straight, Direction::North);
        assert_eq!(p.traverse(Direction::North), Some(Direction::North));
        assert_eq!(p.traverse(Direction::South), Some(Direction::South));
    }

    #[test]
    fn curve_rail_connects_entry_side_to_exit_side() {
        // 北向きの右折: 南の辺と東の辺をつなぐ。
        let p = Panel::new(PanelKind::TurnRight, Direction::North);
        // 北へ進んできた (南の辺から入った) → 東へ抜ける。
        assert_eq!(p.traverse(Direction::North), Some(Direction::East));
        // 西へ進んできた (東の辺から入った) → 南へ抜ける。
        assert_eq!(p.traverse(Direction::West), Some(Direction::South));
    }

    #[test]
    fn entering_from_an_unconnected_side_derails() {
        let p = Panel::new(PanelKind::Straight, Direction::North);
        assert_eq!(p.traverse(Direction::East), None);
        assert_eq!(p.traverse(Direction::West), None);
    }
}
