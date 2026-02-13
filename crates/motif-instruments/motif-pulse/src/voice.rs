use wmidi::{Note, Velocity};

use crate::envelope::Envelope;

/// Single voice of polyphony. Owns a phase accumulator and ADSR envelope.
/// PulseSynth allocates 8 of these; idle voices are skipped during render.
#[derive(Debug, Default)]
pub struct Voice {
    pub phase: f64,
    pub frequency: f64,
    pub velocity: Velocity,
    pub envelope: Envelope,
    pub note: Option<Note>,
    // Monotonic counter for voice-steal ordering (higher = newer).
    pub age: u64,
}

impl Voice {
    pub fn new() -> Self {
        Self::default()
    }

    /// Render one sample. `duty_cycle` (0.0–1.0) controls the fraction of each
    /// wave cycle spent "high" — 0.5 is a square wave, lower values are thinner.
    pub fn render(&mut self, duty_cycle: f64, sample_rate: f64) -> f64 {
        self.phase += self.frequency / sample_rate;

        while self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        let pulse = if self.phase > duty_cycle { 1.0 } else { -1.0 };

        pulse * self.envelope.tick(sample_rate) * (u8::from(self.velocity) as f64 / 127.0)
    }

    pub fn trigger(&mut self, note: Note, velocity: Velocity, frequency: f64, age: u64) {
        self.phase = 0.0;

        self.note = Some(note);
        self.velocity = velocity;
        self.frequency = frequency;
        self.age = age;

        self.envelope.trigger();
    }

    pub fn release(&mut self) {
        self.envelope.release();
    }

    pub fn is_active(&self) -> bool {
        !self.envelope.is_idle()
    }

    pub fn reset(&mut self) {
        self.note = None;
        self.envelope.reset();
    }
}
