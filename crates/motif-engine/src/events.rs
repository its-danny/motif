use wmidi::{Note, Velocity};

/// Unscheduled event — what happened, not when. Nodes see these
/// via handle_event(); timing is stripped by evaluate_node().
#[derive(Debug)]
pub enum Event {
    Midi(MidiEvent),
}

#[derive(Debug)]
pub enum MidiEvent {
    NoteOn { note: Note, velocity: Velocity },
    NoteOff { note: Note },
}

/// Event with a sample-accurate position within the current buffer.
/// sample_offset is a buffer index (0..frames), not a musical time value.
pub struct ScheduledEvent {
    pub sample_offset: u32,
    pub event: Event,
}
