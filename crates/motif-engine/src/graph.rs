use crate::{buffer::AudioBuffer, events::ScheduledEvent, node::AudioNode};

/// The sample-accurate event slicing loop. Walks events in order,
/// rendering sub-ranges between them so state changes (e.g. NoteOn)
/// take effect at the exact sample. Events must be pre-sorted by offset.
pub fn evaluate_node(
    node: &mut dyn AudioNode,
    inputs: &[&AudioBuffer],
    output: &mut AudioBuffer,
    events: &[ScheduledEvent],
    sample_rate: f64,
) {
    let total_frames = output.frames();
    let mut cursor: usize = 0;

    for event in events {
        let offset = event.sample_offset as usize;

        // Render frames up to this event.
        if offset > cursor {
            node.render(inputs, output, cursor..offset, sample_rate);
        }

        // Apply the event at the exact sample.
        node.handle_event(&event.event);

        cursor = offset;
    }

    // Render remaining frames after the last event.
    if cursor < total_frames {
        node.render(inputs, output, cursor..total_frames, sample_rate);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{Event, MidiEvent};
    use std::ops::Range;

    #[derive(Debug, PartialEq)]
    enum Action {
        Render(Range<usize>),
        Event,
    }

    struct SpyNode {
        log: Vec<Action>,
    }

    impl SpyNode {
        fn new() -> Self {
            Self { log: Vec::new() }
        }
    }

    impl AudioNode for SpyNode {
        fn render(
            &mut self,
            _inputs: &[&AudioBuffer],
            _output: &mut AudioBuffer,
            frame_range: Range<usize>,
            _sample_rate: f64,
        ) {
            self.log.push(Action::Render(frame_range));
        }

        fn handle_event(&mut self, _event: &Event) {
            self.log.push(Action::Event);
        }

        fn reset(&mut self) {}
    }

    fn make_event(offset: u32) -> ScheduledEvent {
        ScheduledEvent {
            sample_offset: offset,
            event: Event::Midi(MidiEvent::NoteOn {
                note: wmidi::Note::C4,
                velocity: wmidi::Velocity::MAX,
            }),
        }
    }

    #[test]
    fn no_events_renders_full_buffer() {
        let mut node = SpyNode::new();
        let mut output = AudioBuffer::new(2, 8);
        output.prepare(8);

        evaluate_node(&mut node, &[], &mut output, &[], 44100.0);

        assert_eq!(node.log, vec![Action::Render(0..8)]);
    }

    #[test]
    fn single_event_mid_buffer() {
        let mut node = SpyNode::new();
        let mut output = AudioBuffer::new(2, 8);
        output.prepare(8);
        let events = [make_event(4)];

        evaluate_node(&mut node, &[], &mut output, &events, 44100.0);

        assert_eq!(
            node.log,
            vec![Action::Render(0..4), Action::Event, Action::Render(4..8)]
        );
    }

    #[test]
    fn event_at_offset_zero_no_empty_render() {
        let mut node = SpyNode::new();
        let mut output = AudioBuffer::new(2, 8);
        output.prepare(8);
        let events = [make_event(0)];

        evaluate_node(&mut node, &[], &mut output, &events, 44100.0);

        assert_eq!(node.log, vec![Action::Event, Action::Render(0..8)]);
    }

    #[test]
    fn event_at_last_sample() {
        let mut node = SpyNode::new();
        let mut output = AudioBuffer::new(2, 8);
        output.prepare(8);
        let events = [make_event(7)];

        evaluate_node(&mut node, &[], &mut output, &events, 44100.0);

        assert_eq!(
            node.log,
            vec![Action::Render(0..7), Action::Event, Action::Render(7..8)]
        );
    }

    #[test]
    fn event_at_total_frames_no_trailing_render() {
        let mut node = SpyNode::new();
        let mut output = AudioBuffer::new(2, 8);
        output.prepare(8);
        let events = [make_event(8)];

        evaluate_node(&mut node, &[], &mut output, &events, 44100.0);

        assert_eq!(node.log, vec![Action::Render(0..8), Action::Event]);
    }

    #[test]
    fn two_events_same_offset_no_empty_render_between() {
        let mut node = SpyNode::new();
        let mut output = AudioBuffer::new(2, 8);
        output.prepare(8);
        let events = [make_event(4), make_event(4)];

        evaluate_node(&mut node, &[], &mut output, &events, 44100.0);

        assert_eq!(
            node.log,
            vec![
                Action::Render(0..4),
                Action::Event,
                Action::Event,
                Action::Render(4..8),
            ]
        );
    }

    #[test]
    fn multiple_events_spread_across_buffer() {
        let mut node = SpyNode::new();
        let mut output = AudioBuffer::new(2, 8);
        output.prepare(8);
        let events = [make_event(2), make_event(5), make_event(7)];

        evaluate_node(&mut node, &[], &mut output, &events, 44100.0);

        assert_eq!(
            node.log,
            vec![
                Action::Render(0..2),
                Action::Event,
                Action::Render(2..5),
                Action::Event,
                Action::Render(5..7),
                Action::Event,
                Action::Render(7..8),
            ]
        );
    }

    #[test]
    fn zero_frame_buffer_no_events_no_render() {
        let mut node = SpyNode::new();
        let mut output = AudioBuffer::new(2, 8);
        output.prepare(0);

        evaluate_node(&mut node, &[], &mut output, &[], 44100.0);

        assert_eq!(node.log, vec![]);
    }
}
