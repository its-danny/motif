/// Global, monotonic ID allocation. IDs are never reused within a project.
#[derive(Debug, Default)]
pub struct IdAllocator {
    next_track: u64,
    next_note: u64,
}

impl IdAllocator {
    pub fn next_track_id(&mut self) -> TrackId {
        let track = TrackId(self.next_track);

        self.next_track += 1;

        track
    }

    pub fn next_note_id(&mut self) -> NoteId {
        let note = NoteId(self.next_note);

        self.next_note += 1;

        note
    }
}

/// Stable note identifier. Never reused, so undo references remain valid across edits.
#[derive(Debug, PartialEq)]
pub struct NoteId(pub u64);

/// Stable track identifier. Never reused, so undo references remain valid across edits.
#[derive(Debug, PartialEq)]
pub struct TrackId(pub u64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_note_id() {
        let mut allocator = IdAllocator::default();

        assert_eq!(allocator.next_note_id(), NoteId(0));
        assert_eq!(allocator.next_note_id(), NoteId(1));
        assert_eq!(allocator.next_note_id(), NoteId(2));
    }
}
