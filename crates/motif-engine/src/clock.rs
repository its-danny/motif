use motif_core::tick::Tick;

pub const TICKS_PER_QUARTER: u64 = 480;

/// Bridge between musical time (ticks) and audio time (samples).
/// sample_position is the single source of truth on the audio thread —
/// ticks are always derived FROM samples, never accumulated independently.
#[derive(Debug)]
pub struct Clock {
    #[allow(dead_code)]
    sample_position: u64,
    sample_rate: f64,
}

impl Clock {
    /// Convert a tick position to the corresponding sample. Uses ceil()
    /// so that sample_to_tick(tick_to_sample(t)) == t for on-grid ticks.
    pub fn tick_to_sample(&self, tick: Tick, bpm: f64) -> u64 {
        let seconds = tick.to_quarters() / (bpm / 60.0);

        (seconds * self.sample_rate).ceil() as u64
    }

    /// Derive the current tick position from a sample position. Used to
    /// report playhead position back to the UI — audio thread never stores ticks.
    pub fn sample_to_tick(&self, sample: u64, bpm: f64) -> Tick {
        let seconds = sample as f64 / self.sample_rate;
        let quarters = seconds * (bpm / 60.0);

        Tick::from_raw((quarters * TICKS_PER_QUARTER as f64) as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clock() -> Clock {
        Clock {
            sample_position: 0,
            sample_rate: 48000.0,
        }
    }

    #[test]
    fn tick_zero_is_sample_zero() {
        let c = clock();

        assert_eq!(c.tick_to_sample(Tick::ZERO, 120.0), 0);
    }

    #[test]
    fn one_beat_at_120bpm() {
        let c = clock();

        // 120 BPM = 2 beats/sec → 1 beat = 0.5s → 24000 samples at 48kHz.
        assert_eq!(c.tick_to_sample(Tick::from_quarters(1), 120.0), 24000);
    }

    #[test]
    fn two_beats_at_120bpm() {
        let c = clock();

        assert_eq!(c.tick_to_sample(Tick::from_quarters(2), 120.0), 48000);
    }

    #[test]
    fn round_trip_on_grid_ticks() {
        let c = clock();

        for &ticks in &[0, 480, 960, 1920] {
            let tick = Tick::from_raw(ticks);
            let sample = c.tick_to_sample(tick, 120.0);
            let back = c.sample_to_tick(sample, 120.0);

            assert_eq!(back, tick, "round-trip failed for tick {ticks}");
        }
    }

    #[test]
    fn round_trip_high_bpm() {
        let c = clock();

        for &ticks in &[0, 480, 960] {
            let tick = Tick::from_raw(ticks);
            let sample = c.tick_to_sample(tick, 300.0);
            let back = c.sample_to_tick(sample, 300.0);

            assert_eq!(back, tick, "round-trip failed at 300 BPM for tick {ticks}");
        }
    }

    #[test]
    fn round_trip_low_bpm() {
        let c = clock();

        for &ticks in &[0, 480, 960] {
            let tick = Tick::from_raw(ticks);
            let sample = c.tick_to_sample(tick, 40.0);
            let back = c.sample_to_tick(sample, 40.0);

            assert_eq!(back, tick, "round-trip failed at 40 BPM for tick {ticks}");
        }
    }
}
