use wmidi::{Note, Velocity};

use crate::tick::Tick;

/// A note placed in time within a clip. Positions are relative to the
/// clip's start, not absolute in the arrangement. Baking resolves
/// to absolute positions later.
#[derive(Debug)]
pub struct NoteEvent {
    /// Offset from the clip's start.
    pub start_tick: Tick,
    /// Raw u64, not Tick. This is a duration, not a position.
    pub length_ticks: u64,
    pub note: Note,
    pub velocity: Velocity,
}
