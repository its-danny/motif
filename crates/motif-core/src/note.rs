use wmidi::{Note, Velocity};

use crate::tick::Tick;

#[derive(Debug)]
pub struct NoteEvent {
    pub start_tick: Tick,
    pub length_ticks: u64,
    pub note: Note,
    pub velocity: Velocity,
}
