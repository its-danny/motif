/// 480 PPQ. Divides cleanly by 2, 3, 4, 5, 8, 16, 32, etc.
/// Covers all standard musical subdivisions including triplets.
pub const TICKS_PER_QUARTER: u64 = 480;

/// Absolute position in musical time. Integer-only to avoid float drift.
/// Durations use raw `u64` instead. Tick is specifically a position.
#[derive(Debug, PartialEq, PartialOrd)]
pub struct Tick(u64);

impl Tick {
    pub const ZERO: Tick = Tick(0);

    pub fn from_quarters(quarters: u64) -> Self {
        Tick(quarters * TICKS_PER_QUARTER)
    }

    /// Construct from a beat fraction (e.g. 1/16 = sixteenth note).
    /// Panics in debug if the division isn't exact.
    pub fn from_beats(numerator: u64, denominator: u64) -> Self {
        let total = numerator * TICKS_PER_QUARTER * 4;

        debug_assert!(
            total.is_multiple_of(denominator),
            "from_beats({numerator}, {denominator}) doesn't divide evenly into PPQ 480"
        );

        Tick(total / denominator)
    }

    pub fn to_quarters(self) -> f64 {
        self.0 as f64 / TICKS_PER_QUARTER as f64
    }

    /// Round to the nearest grid line. Ties round up.
    pub fn snap_to_grid(self, grid: u64) -> Tick {
        Tick(((self.0 + grid / 2) / grid) * grid)
    }

    pub fn saturating_sub(self, other: Tick) -> Tick {
        Tick(self.0.saturating_sub(other.0))
    }
}

impl std::ops::Add for Tick {
    type Output = Tick;

    fn add(self, rhs: Tick) -> Tick {
        Tick(self.0 + rhs.0)
    }
}

impl std::ops::Sub for Tick {
    type Output = Tick;

    fn sub(self, rhs: Tick) -> Tick {
        Tick(self.0 - rhs.0)
    }
}

impl std::ops::AddAssign for Tick {
    fn add_assign(&mut self, rhs: Tick) {
        self.0 += rhs.0;
    }
}

impl std::ops::SubAssign for Tick {
    fn sub_assign(&mut self, rhs: Tick) {
        self.0 -= rhs.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_is_zero() {
        assert_eq!(Tick::ZERO, Tick(0));
    }

    #[test]
    fn from_quarters() {
        assert_eq!(Tick::from_quarters(1), Tick(480));
        assert_eq!(Tick::from_quarters(0), Tick::ZERO);
    }

    #[test]
    fn from_beats() {
        assert_eq!(Tick::from_beats(1, 1), Tick(1920));
        assert_eq!(Tick::from_beats(1, 3), Tick(640));
        assert_eq!(Tick::from_beats(1, 4), Tick(480));
        assert_eq!(Tick::from_beats(1, 8), Tick(240));
        assert_eq!(Tick::from_beats(1, 16), Tick(120));
        assert_eq!(Tick::from_beats(1, 32), Tick(60));
    }

    #[test]
    #[should_panic]
    fn from_beats_rejects_uneven_division() {
        Tick::from_beats(1, 7);
    }

    #[test]
    #[should_panic]
    fn from_beats_rejects_zero_denominator() {
        Tick::from_beats(1, 0);
    }

    #[test]
    fn to_quarters() {
        assert_eq!(Tick::ZERO.to_quarters(), 0.0);
        assert_eq!(Tick(240).to_quarters(), 0.5);
        assert_eq!(Tick::from_quarters(1).to_quarters(), 1.0);
    }

    #[test]
    fn snap_to_grid() {
        assert_eq!(Tick(0).snap_to_grid(120), Tick(0));
        assert_eq!(Tick(50).snap_to_grid(120), Tick(0));
        assert_eq!(Tick(60).snap_to_grid(120), Tick(120));
        assert_eq!(Tick(80).snap_to_grid(120), Tick(120));
        assert_eq!(Tick(130).snap_to_grid(120), Tick(120));
        assert_eq!(Tick(500).snap_to_grid(160), Tick(480));
    }

    #[test]
    fn arithmetic() {
        assert_eq!(Tick(480) + Tick(240), Tick(720));
        assert_eq!(Tick(480) - Tick(240), Tick(240));

        let mut tick = Tick(480);
        tick += Tick(240);
        assert_eq!(tick, Tick(720));

        let mut tick = Tick(480);
        tick -= Tick(240);
        assert_eq!(tick, Tick(240));

        assert_eq!(Tick(200).saturating_sub(Tick(100)), Tick(100));
        assert_eq!(Tick(100).saturating_sub(Tick(200)), Tick::ZERO);
    }

    #[test]
    fn ordering() {
        assert!(Tick(0) < Tick(1));
        assert!(Tick(480) == Tick(480));
    }
}
