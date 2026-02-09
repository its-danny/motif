use std::ops::Range;

use crate::{buffer::AudioBuffer, events::Event};

/// Universal interface for anything that produces or transforms audio.
/// Nodes never handle event timing — evaluate_node() slices the buffer
/// and calls render/handle_event in the correct interleaved order.
pub trait AudioNode: Send {
    /// REAL-TIME SAFETY: Called on the audio thread. Must not allocate, lock, block, or panic.
    fn render(
        &mut self,
        inputs: &[&AudioBuffer],
        output: &mut AudioBuffer,
        frame_range: Range<usize>,
        sample_rate: f64,
    );

    /// REAL-TIME SAFETY: Called on the audio thread. Must not allocate, lock, block, or panic.
    fn handle_event(&mut self, event: &Event);

    /// REAL-TIME SAFETY: Called on the audio thread. Must not allocate, lock, block, or panic.
    fn reset(&mut self);
}
