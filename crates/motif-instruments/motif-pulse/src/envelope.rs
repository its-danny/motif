/// ADSR stage. Progresses linearly: Idle → Attack → Decay → Sustain → Release → Idle.
#[derive(Debug, Default, PartialEq)]
pub enum State {
    #[default]
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

/// Per-voice linear ADSR envelope. Each voice owns one; the state machine
/// runs per-sample via tick(). Not SmoothedParam — this is amplitude shaping,
/// not parameter interpolation.
#[derive(Debug, Default)]
pub struct Envelope {
    state: State,
    level: f64,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    release_start: f64,
}

impl Envelope {
    pub fn new() -> Self {
        Self::default()
    }

    /// Start the envelope from zero. Called on NoteOn (including voice steal).
    pub fn trigger(&mut self) {
        self.state = State::Attack;
        self.level = 0.0;
    }

    /// Begin release from current level. Captures level so release ramp
    /// works correctly even if triggered during attack/decay.
    pub fn release(&mut self) {
        self.release_start = self.level;
        self.state = State::Release;
    }

    /// Set ADSR parameters. Called after trigger() so the voice uses
    /// the synth's current values (times in seconds, sustain is a level 0–1).
    pub fn adsr(&mut self, attack: f32, decay: f32, sustain: f32, release: f32) {
        self.attack = attack;
        self.decay = decay;
        self.sustain = sustain;
        self.release = release;
    }

    /// Advance one sample, return current amplitude (0.0–1.0).
    /// Times below 1e-5s are treated as instant to avoid division blowup.
    pub fn tick(&mut self, sample_rate: f64) -> f64 {
        match self.state {
            State::Idle => {}
            State::Attack => {
                if self.attack < 1e-5 {
                    self.level = 1.0;
                    self.state = State::Decay;
                } else {
                    self.level += 1.0 / (self.attack as f64 * sample_rate);

                    if self.level > 1.0 {
                        self.level = self.level.clamp(0.0, 1.0);
                        self.state = State::Decay;
                    }
                }
            }
            State::Decay => {
                if self.decay < 1e-5 {
                    self.level = self.sustain as f64;
                    self.state = State::Sustain;
                } else {
                    self.level -= (1.0 - self.sustain as f64) / (self.decay as f64 * sample_rate);

                    if self.level < self.sustain as f64 {
                        self.level = self.level.clamp(0.0, self.sustain as f64);
                        self.state = State::Sustain;
                    }
                }
            }
            State::Sustain => {}
            State::Release => {
                if self.release < 1e-5 {
                    self.level = 0.0;
                    self.state = State::Idle;
                } else {
                    self.level -= self.release_start / (self.release as f64 * sample_rate);

                    if self.level < 0.0 {
                        self.level = 0.0;
                        self.state = State::Idle;
                    }
                }
            }
        }

        self.level
    }

    pub fn is_idle(&self) -> bool {
        self.state == State::Idle
    }

    pub fn is_releasing(&self) -> bool {
        self.state == State::Release
    }

    pub fn reset(&mut self) {
        self.state = State::Idle;
        self.level = 0.0;
    }
}
