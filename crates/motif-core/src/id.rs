#[derive(Debug, Default)]
pub struct IdAllocator {
    next_note: u64,
}

impl IdAllocator {
    pub fn next_note_id(&mut self) -> NoteId {
        let note = NoteId(self.next_note);

        self.next_note += 1;

        note
    }
}

#[derive(Debug, PartialEq)]
pub struct NoteId(pub u64);

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
