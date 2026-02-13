//! Minimal example: play a C major arpeggio through PulseSynth via cpal.
//! Run with: cargo run -p motif-pulse --example play

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use motif_engine::{
    buffer::AudioBuffer,
    events::{Event, MidiEvent, ScheduledEvent},
    graph::evaluate_node,
};
use motif_pulse::synth::Pulse;

fn main() {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device");
    let config = device.default_output_config().expect("no default config");
    let sample_rate = config.sample_rate() as f64;
    let channels = config.channels() as usize;

    println!("Output: {}Hz, {} ch", sample_rate, channels);

    let mut synth = Pulse::new();
    let mut buffer = AudioBuffer::new(2, 8192);
    let mut samples_elapsed: u64 = 0;

    // Schedule: NoteOn at 0s, 0.5s, 1.0s; all off at 2.0s.
    let sr = sample_rate;
    let schedule: Vec<(u64, Event)> = vec![
        (
            0,
            Event::Midi(MidiEvent::NoteOn {
                note: wmidi::Note::C4,
                velocity: wmidi::Velocity::MAX,
            }),
        ),
        (
            (0.5 * sr) as u64,
            Event::Midi(MidiEvent::NoteOn {
                note: wmidi::Note::E4,
                velocity: wmidi::Velocity::MAX,
            }),
        ),
        (
            (1.0 * sr) as u64,
            Event::Midi(MidiEvent::NoteOn {
                note: wmidi::Note::G4,
                velocity: wmidi::Velocity::MAX,
            }),
        ),
        (
            (2.0 * sr) as u64,
            Event::Midi(MidiEvent::NoteOff {
                note: wmidi::Note::C4,
            }),
        ),
        (
            (2.0 * sr) as u64,
            Event::Midi(MidiEvent::NoteOff {
                note: wmidi::Note::E4,
            }),
        ),
        (
            (2.0 * sr) as u64,
            Event::Midi(MidiEvent::NoteOff {
                note: wmidi::Note::G4,
            }),
        ),
    ];
    let mut schedule_cursor = 0;

    let stream = device
        .build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let frames = data.len() / channels;
                buffer.prepare(frames);

                let mut events = Vec::new();

                // Collect events that fall within this buffer.
                while schedule_cursor < schedule.len() {
                    let (trigger_at, ref event) = schedule[schedule_cursor];

                    if trigger_at < samples_elapsed + frames as u64 {
                        let offset = trigger_at.saturating_sub(samples_elapsed) as u32;

                        events.push(ScheduledEvent {
                            sample_offset: offset,
                            event: event.clone(),
                        });

                        schedule_cursor += 1;
                    } else {
                        break;
                    }
                }

                evaluate_node(&mut synth, &[], &mut buffer, &events, sample_rate);

                // Write to cpal output. If more than 2 channels, extra channels get silence.
                for frame in 0..frames {
                    for ch in 0..channels {
                        data[frame * channels + ch] = if ch < 2 {
                            buffer.channel(ch)[frame]
                        } else {
                            0.0
                        };
                    }
                }

                samples_elapsed += frames as u64;
            },
            |err| eprintln!("audio error: {err}"),
            None,
        )
        .expect("failed to build stream");

    stream.play().expect("failed to start stream");

    println!("Playing C major arpeggio... (3 seconds)");
    std::thread::sleep(std::time::Duration::from_secs(3));

    println!("Done.");
}
