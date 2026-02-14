use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use motif_engine::{
    buffer::AudioBuffer,
    control::PlaybackControl,
    events::{RoutedEvent, ScheduledEvent},
    graph::evaluate_node,
};
use motif_pulse::synth::Pulse;
use rtrb::{Consumer, RingBuffer};

struct AudioRuntime {
    // Keep alive.
    _stream: cpal::Stream,
}

fn main() -> iced::Result {
    let (producer, consumer) = RingBuffer::<RoutedEvent>::new(1024);

    let playback = PlaybackControl::new(producer);
    let _audio = start_audio(consumer);

    motif_ui::run(playback)
}

fn start_audio(consumer: Consumer<RoutedEvent>) -> AudioRuntime {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device");
    let config = device.default_output_config().expect("no output config");
    let sample_rate = config.sample_rate() as f64;
    let channels = config.channels() as usize;

    let mut synth = Pulse::new();
    let mut buffer = AudioBuffer::new(2, 8192);
    let mut consumer = consumer;
    let mut scheduled = Vec::<ScheduledEvent>::with_capacity(128);

    let stream = device
        .build_output_stream(
            &config.into(),
            move |out: &mut [f32], _| {
                let frames = out.len() / channels;
                buffer.prepare(frames);

                scheduled.clear();

                while let Ok(routed) = consumer.pop() {
                    // Ignore for now, single synth.
                    let _track = routed.track_id;

                    scheduled.push(ScheduledEvent {
                        sample_offset: 0,
                        event: routed.event,
                    });
                }

                evaluate_node(&mut synth, &[], &mut buffer, &scheduled, sample_rate);

                for f in 0..frames {
                    for ch in 0..channels {
                        out[f * channels + ch] = if ch < 2 { buffer.channel(ch)[f] } else { 0.0 };
                    }
                }
            },
            |err| eprintln!("audio error: {err}"),
            None,
        )
        .expect("build stream failed");

    stream.play().expect("play failed");

    AudioRuntime { _stream: stream }
}
