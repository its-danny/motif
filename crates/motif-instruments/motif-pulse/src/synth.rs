use std::ops::Range;

use motif_engine::{
    buffer::AudioBuffer,
    events::{Event, MidiEvent},
    node::AudioNode,
};

use crate::voice::Voice;

/// Master output scaling. Prevents clipping when multiple voices are active.
const GAIN: f64 = 0.15;

/// 8-voice polyphonic pulse wave synthesizer. Implements AudioNode —
/// feed it NoteOn/NoteOff events via evaluate_node() and it produces audio.
/// ADSR params are shared; each voice gets a copy on trigger.
#[derive(Debug)]
pub struct Pulse {
    pub voices: [Voice; 8],
    pub duty_cycle: f64,
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
    pub next_age: u64,
}

impl Default for Pulse {
    fn default() -> Self {
        Self::new()
    }
}

impl Pulse {
    pub fn new() -> Self {
        Self {
            voices: std::array::from_fn(|_| Voice::new()),
            duty_cycle: 0.5,
            attack: 0.01,
            decay: 0.1,
            sustain: 0.7,
            release: 0.15,
            next_age: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use motif_engine::{events::ScheduledEvent, graph::evaluate_node};
    use wmidi::{Note, Velocity};

    const SAMPLE_RATE: f64 = 48000.0;

    fn make_synth() -> Pulse {
        Pulse::new()
    }

    fn note_on(offset: u32, note: Note) -> ScheduledEvent {
        ScheduledEvent {
            sample_offset: offset,
            event: Event::Midi(MidiEvent::NoteOn {
                note,
                velocity: Velocity::MAX,
            }),
        }
    }

    fn note_off(offset: u32, note: Note) -> ScheduledEvent {
        ScheduledEvent {
            sample_offset: offset,
            event: Event::Midi(MidiEvent::NoteOff { note }),
        }
    }

    fn has_signal(buf: &AudioBuffer, range: std::ops::Range<usize>) -> bool {
        buf.channel(0)[range].iter().any(|&s| s.abs() > 1e-6)
    }

    fn is_silent(buf: &AudioBuffer, range: std::ops::Range<usize>) -> bool {
        buf.channel(0)[range].iter().all(|&s| s.abs() < 1e-6)
    }

    #[test]
    fn note_on_produces_output() {
        let mut synth = make_synth();
        let mut output = AudioBuffer::new(2, 256);
        output.prepare(256);

        let events = [note_on(0, Note::C4)];
        evaluate_node(&mut synth, &[], &mut output, &events, SAMPLE_RATE);

        assert!(has_signal(&output, 0..256));
    }

    #[test]
    fn no_events_is_silent() {
        let mut synth = make_synth();
        let mut output = AudioBuffer::new(2, 256);
        output.prepare(256);

        evaluate_node(&mut synth, &[], &mut output, &[], SAMPLE_RATE);

        assert!(is_silent(&output, 0..256));
    }

    #[test]
    fn note_off_releases_to_silence() {
        let mut synth = make_synth();
        let mut output = AudioBuffer::new(2, 1024);

        // NoteOn, render a bit.
        output.prepare(512);
        let events = [note_on(0, Note::C4)];
        evaluate_node(&mut synth, &[], &mut output, &events, SAMPLE_RATE);
        assert!(has_signal(&output, 0..512));

        // NoteOff, then render enough for release to finish (0.15s = 7200 samples).
        output.prepare(1024);
        let events = [note_off(0, Note::C4)];
        evaluate_node(&mut synth, &[], &mut output, &events, SAMPLE_RATE);

        // Render more buffers until release finishes.
        for _ in 0..10 {
            output.prepare(1024);
            evaluate_node(&mut synth, &[], &mut output, &[], SAMPLE_RATE);
        }

        assert!(is_silent(&output, 0..1024));
    }

    #[test]
    fn polyphony_three_notes() {
        let mut synth = make_synth();
        let mut output = AudioBuffer::new(2, 256);
        output.prepare(256);

        let events = [
            note_on(0, Note::C4),
            note_on(0, Note::E4),
            note_on(0, Note::G4),
        ];
        evaluate_node(&mut synth, &[], &mut output, &events, SAMPLE_RATE);

        let active = synth.voices.iter().filter(|v| v.is_active()).count();
        assert_eq!(active, 3);
    }

    #[test]
    fn voice_steal_at_capacity() {
        let mut synth = make_synth();
        let mut output = AudioBuffer::new(2, 256);
        output.prepare(256);

        // Fill all 8 voices.
        let notes = [
            Note::C3,
            Note::D3,
            Note::E3,
            Note::F3,
            Note::G3,
            Note::A3,
            Note::B3,
            Note::C4,
        ];
        let events: Vec<_> = notes.iter().map(|&n| note_on(0, n)).collect();
        evaluate_node(&mut synth, &[], &mut output, &events, SAMPLE_RATE);
        assert_eq!(synth.voices.iter().filter(|v| v.is_active()).count(), 8);

        // 9th note should steal the oldest (C3).
        output.prepare(256);
        let events = [note_on(0, Note::D4)];
        evaluate_node(&mut synth, &[], &mut output, &events, SAMPLE_RATE);

        assert_eq!(synth.voices.iter().filter(|v| v.is_active()).count(), 8);
        // Oldest voice (age 0) was stolen — no voice should play C3 anymore.
        assert!(!synth.voices.iter().any(|v| v.note == Some(Note::C3)));
    }

    #[test]
    fn reset_silences_all() {
        let mut synth = make_synth();
        let mut output = AudioBuffer::new(2, 256);
        output.prepare(256);

        let events = [note_on(0, Note::C4), note_on(0, Note::E4)];
        evaluate_node(&mut synth, &[], &mut output, &events, SAMPLE_RATE);
        assert!(has_signal(&output, 0..256));

        synth.reset();

        output.prepare(256);
        evaluate_node(&mut synth, &[], &mut output, &[], SAMPLE_RATE);
        assert!(is_silent(&output, 0..256));
    }

    #[test]
    fn output_is_mono_both_channels_equal() {
        let mut synth = make_synth();
        let mut output = AudioBuffer::new(2, 256);
        output.prepare(256);

        let events = [note_on(0, Note::A4)];
        evaluate_node(&mut synth, &[], &mut output, &events, SAMPLE_RATE);

        assert_eq!(output.channel(0), output.channel(1));
    }

    #[test]
    fn mid_buffer_note_on_silent_before_event() {
        let mut synth = make_synth();
        let mut output = AudioBuffer::new(2, 256);
        output.prepare(256);

        let events = [note_on(128, Note::C4)];
        evaluate_node(&mut synth, &[], &mut output, &events, SAMPLE_RATE);

        assert!(is_silent(&output, 0..128));
        assert!(has_signal(&output, 128..256));
    }
}

impl AudioNode for Pulse {
    fn render(
        &mut self,
        _inputs: &[&AudioBuffer],
        output: &mut AudioBuffer,
        frame_range: Range<usize>,
        sample_rate: f64,
    ) {
        for frame in frame_range {
            let mut sum = 0.0;

            for voice in &mut self.voices {
                if voice.is_active() {
                    sum += voice.render(self.duty_cycle, sample_rate);
                }
            }

            let out = (sum * GAIN) as f32;
            output.channel_mut(0)[frame] = out;
            output.channel_mut(1)[frame] = out;
        }
    }

    fn handle_event(&mut self, event: &Event) {
        match event {
            Event::Midi(event) => match event {
                MidiEvent::NoteOn { note, velocity } => {
                    // Find an inactive voice first.
                    let voice_index = self
                        .voices
                        .iter()
                        .position(|v| !v.is_active())
                        .unwrap_or_else(|| {
                            // If no inactive voice, find the oldest one.
                            self.voices
                                .iter()
                                .enumerate()
                                .min_by_key(|(_, v)| v.age)
                                .map(|(i, _)| i)
                                .unwrap()
                        });

                    let voice = &mut self.voices[voice_index];
                    voice.trigger(*note, *velocity, note.to_freq_f64(), self.next_age);
                    voice
                        .envelope
                        .adsr(self.attack, self.decay, self.sustain, self.release);

                    self.next_age += 1;
                }
                MidiEvent::NoteOff { note } => {
                    // Find the voice(s) matching the note and trigger release.
                    for voice in &mut self.voices {
                        if voice.note == Some(*note) && !voice.envelope.is_releasing() {
                            voice.release();
                        }
                    }
                }
            },
        }
    }

    fn reset(&mut self) {
        for voice in &mut self.voices {
            voice.reset();
        }

        self.next_age = 0;
    }
}
