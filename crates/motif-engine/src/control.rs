use motif_core::id::TrackId;
use rtrb::Producer;

use crate::{
    error::EngineError,
    events::{Event, MidiEvent, RoutedEvent},
};

/// UI-facing handle for sending real-time events to the audio thread.
///
/// This keeps ring-buffer details out of the UI crate so transport semantics
/// live in one place.
pub struct PlaybackControl {
    producer: Producer<RoutedEvent>,
}

impl PlaybackControl {
    pub fn new(producer: Producer<RoutedEvent>) -> Self {
        Self { producer }
    }

    /// Enqueue a live MIDI event for the next audio callback.
    pub fn send_midi(&mut self, track_id: TrackId, midi: MidiEvent) -> Result<(), EngineError> {
        let routed = RoutedEvent {
            track_id,
            event: Event::Midi(midi),
        };

        self.producer
            .push(routed)
            .map_err(|_| EngineError::BufferFull)
    }
}
