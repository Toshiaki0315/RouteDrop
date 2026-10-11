use super::types::PanelKind;

/// 外部クレート不要の決定的乱数 (LCG)。同じシードなら同じ列を返す。
#[derive(Clone, Debug)]
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng { state: seed }
    }

    pub fn next_u32(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.state >> 33) as u32
    }

    pub fn next_panel(&mut self) -> PanelKind {
        match self.next_u32() % 3 {
            0 => PanelKind::Straight,
            1 => PanelKind::TurnRight,
            _ => PanelKind::TurnLeft,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..10 {
            assert_eq!(a.next_u32(), b.next_u32());
        }
    }

    #[test]
    fn different_seed_different_sequence() {
        let mut a = Rng::new(1);
        let mut b = Rng::new(2);
        let xs: Vec<u32> = (0..5).map(|_| a.next_u32()).collect();
        let ys: Vec<u32> = (0..5).map(|_| b.next_u32()).collect();
        assert_ne!(xs, ys);
    }

    #[test]
    fn next_panel_produces_every_kind() {
        let mut rng = Rng::new(7);
        let kinds: Vec<PanelKind> = (0..200).map(|_| rng.next_panel()).collect();
        assert!(kinds.contains(&PanelKind::Straight));
        assert!(kinds.contains(&PanelKind::TurnRight));
        assert!(kinds.contains(&PanelKind::TurnLeft));
    }
}
