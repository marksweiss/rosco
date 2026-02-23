use derive_builder::Builder;

use crate::note::playback_note::{PlaybackNote};
use crate::sequence::note_sequence_trait::{AppendNote, AppendNotes, BuilderWrapper, IterMutWrapper, NextNotes, SetCurPosition};

/**
Simple vec of vec sequence. Direct access to notes at an index, or just the first note at an index.
If an insert index is off the end the note will be appended. No opinion on any note properties.
*/
#[derive(Builder, Clone, Debug)]
pub(crate) struct GridNoteSequence {
    #[builder(default = "Vec::new()")]
    sequence: Vec<Vec<PlaybackNote>>,

    // initialize to 1, past dummy first note 
    #[builder(default = "0")]
    index: usize,
}

impl AppendNote for GridNoteSequence {
    fn append_note(&mut self, playback_note: PlaybackNote) {
        self.append_note(playback_note);
    }
}

impl AppendNotes for GridNoteSequence {
    fn append_notes(&mut self, notes: &[PlaybackNote]) {
        self.append_notes(notes);
    }
}

impl NextNotes for GridNoteSequence {
    fn next_notes(&mut self) -> Vec<PlaybackNote> {
        let notes = self.get_notes_at(self.index);
        self.increment();
        notes
    }
}

impl BuilderWrapper<GridNoteSequence> for GridNoteSequenceBuilder {
    fn new () -> GridNoteSequence {
        GridNoteSequenceBuilder::default().build().unwrap()
    }
}

impl SetCurPosition for GridNoteSequence {
    fn set_cur_position(&mut self, position: f32) {
        self.index = position as usize;
    }
}

impl IterMutWrapper for GridNoteSequence {
    fn iter_mut(&mut self) -> std::slice::IterMut<Vec<PlaybackNote>> {
        self.sequence.iter_mut()
    }
}

#[allow(dead_code)]
impl GridNoteSequence {

    pub(crate) fn append_notes(&mut self, playback_notes: &[PlaybackNote]) {
        if playback_notes.is_empty() {
            panic!("Notes to add must not be empty");
        }

        self.sequence.push(playback_notes.to_vec());
    }
    
    pub(crate) fn insert_notes(&mut self, playback_notes: Vec<PlaybackNote>) {
        if playback_notes.is_empty() {
            panic!("Notes to add must not be empty");
        }
        if self.index >= self.sequence.len() {
            self.append_notes(&playback_notes);
        }
        self.sequence.insert(self.index, playback_notes);
    }

    pub(crate) fn append_note(&mut self, playback_note: PlaybackNote) {
        self.sequence.push(vec![playback_note.clone()]);
    }
    
    pub(crate) fn insert_note(&mut self, playback_note: PlaybackNote) {
        if playback_note.note_start_time_ms() < 0.0 {
            panic!("Note start time must be >= 0.0");
        }
        self.sequence.insert(self.index, vec![playback_note]);
    }

    pub(crate) fn insert_notes_at(&mut self, playback_notes: &mut Vec<PlaybackNote>, index: usize) {
        if playback_notes.is_empty() {
            panic!("Notes to add must not be empty");
        }
        if index >= self.sequence.len() {
            self.append_notes(playback_notes);
            return;
        }
        self.sequence.insert(index, (*playback_notes).clone());
    }

    pub(crate) fn insert_note_at(&mut self, playback_note: PlaybackNote, index: usize) {
        if index >= self.sequence.len() {
            self.append_note(playback_note);
            return;
        }
        self.sequence.insert(index, vec![playback_note]);
    }

    pub(crate) fn get_note(&self) -> PlaybackNote {
        self.sequence[self.index][0].clone()
    }

    // Only makes sense with an index and as an internal method
    // Would be public in a grid- rather than time-based sequencer
    pub(crate) fn get_notes(&self) -> Vec<PlaybackNote> {
        if self.index >= self.sequence.len() {
            panic!("Index out of bounds");
        }
        self.sequence[self.index].clone()
    }

    // deprecated because makes no sense once we store a vector at each position
    pub(crate) fn get_note_at(&self, index: usize) -> PlaybackNote {
        if index >= self.sequence.len() {
            panic!("Index out of bounds");
        }
        self.sequence[index][0].clone()
    }

    // Only makes sense with an index and as an internal method
    // Would be public in a grid- rather than time-based sequencer
    // #VisibleForTesting
    pub(crate) fn get_notes_at(&self, index: usize) -> Vec<PlaybackNote> {
        if index > self.sequence.len() - 1 {
            return vec![];
        }
        
        self.sequence[index].clone()
    }
    
    // Only makes sense with an index and as an internal method
    // Would be public in a grid- rather than time-based sequencer
    pub(crate) fn notes_len(&self) -> usize {
        if self.index >= self.sequence.len() {
            panic!("Index out of bounds");
        }
        self.sequence[self.index].len()
    }

    // Only makes sense with an index and as an internal method
    // Would be public in a grid- rather than time-based sequencer
    pub(crate) fn notes_are_empty(&self) -> bool {
        if self.index >= self.sequence.len() {
            panic!("Index out of bounds");
        }
        self.sequence[self.index].is_empty()
    }

    // Only makes sense with an index and as an internal method
    // Would be public in a grid- rather than time-based sequencer
    pub(crate) fn get_index(&self) -> usize {
        self.index
    }

    // Only makes sense with an index and as an internal method
    // Would be public in a grid- rather than time-based sequencer
    pub(crate) fn increment(&mut self) {
        if self.index < self.sequence.len() {
            self.index += 1;
        }
    }

    // Only makes sense with an index and as an internal method
    // Would be public in a grid- rather than time-based sequencer
    pub(crate) fn decrement(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        }
    }

    // Only makes sense with an index and as an internal method
    // Would be public in a grid- rather than time-based sequencer
    pub(crate) fn reset_index(&mut self) {
        self.index = 0;
    }

    // Only makes sense with an index and as an internal method
    // Would be public in a grid- rather than time-based sequencer
    pub(crate) fn at_end(&self) -> bool {
        self.index >= self.sequence.len() - 1
    }

    // Only makes sense with an index and as an internal method
    // Would be public in a grid- rather than time-based sequencer
    pub(crate) fn sequence_is_empty(&self) -> bool {
        self.sequence.is_empty()
    }

    // Only makes sense with an index and as an internal method
    // Would be public in a grid- rather than time-based sequencer
    pub(crate) fn sequence_iter_mut(&mut self) -> std::slice::IterMut<Vec<PlaybackNote>> {
        self.sequence.iter_mut()
    }

    // Only makes sense with an index and as an internal method
    // Would be public in a grid- rather than time-based sequencer
    pub(crate) fn sequence_iter(&self) -> std::slice::Iter<Vec<PlaybackNote>> {
        self.sequence.iter()
    }

    // Only makes sense with an index and as an internal method
    // Would be public in a grid- rather than time-based sequencer
    pub(crate) fn sequence_len(&self) -> usize {
        self.sequence.len()
    }
}

impl<'a> Iterator for GridNoteSequence {
    type Item = Vec<PlaybackNote>;

    fn next(&mut self) -> Option<Self::Item> {
        let playback_notes = self.get_notes_at(self.index);
        self.increment();
        if playback_notes.is_empty() {
            return None;
        }

        Some(playback_notes)
    }
}

#[cfg(test)]
mod test_grid_note_sequence {
    use crate::common::float_utils::float_eq;
    use crate::note::note::NoteBuilder;
    use crate::note::playback_note;
    use crate::sequence::grid_note_sequence::GridNoteSequenceBuilder;

    #[test]
    fn test_append_note() {
        let note= playback_note::from_note(
            setup_note()
                .start_time_ms(0.0)
                .build().unwrap()
        );

        let sequence= GridNoteSequenceBuilder::default()
            .sequence(vec![vec![note.clone()]])
            .index(0)
            .build().unwrap();

        assert_eq!(sequence.get_notes()[0], note);
    }

    #[test]
    fn test_append_notes() {
        let note_1= playback_note::from_note(
            setup_note()
                .start_time_ms(0.0)
                .end_time_ms(1000.0)
                .build().unwrap()
        );
        let note_2= playback_note::from_note(
            setup_note()
                .start_time_ms(0.0)
                .end_time_ms(1000.0)
                .build().unwrap()
        );

        let sequence = GridNoteSequenceBuilder::default()
            .sequence(vec![vec![note_1.clone(), note_2.clone()]])
            .index(0)
            .build().unwrap();

        let actual = sequence.get_notes() ;
        assert_eq!(float_eq(actual[0].note_start_time_ms(), note_1.note_start_time_ms()), true);
        assert_eq!(actual[0], note_1);
        assert_eq!(actual[1], note_2);
    }

    #[test]
    fn test_insert_notes_get_notes_at() {
        let note_1= playback_note::from_note(
            setup_note()
                .start_time_ms(2.0)
                .build().unwrap()
        );
        let note_2= playback_note::from_note(
            setup_note()
                .start_time_ms(2.0)
                .build().unwrap()
        );
        let note_3= playback_note::from_note(
            setup_note()
                .start_time_ms(0.0)
                .build().unwrap()
        );
        let note_4= playback_note::from_note(
            setup_note()
                .start_time_ms(0.0)
                .build().unwrap()
        );
        let note_5= playback_note::from_note(
            setup_note()
                .start_time_ms(1.0)
                .build().unwrap()
        );

        let mut sequence= GridNoteSequenceBuilder::default()
            .index(0)
            .build().unwrap();
        sequence.insert_notes(vec![note_1.clone(), note_2.clone()]);
        sequence.increment();
        sequence.insert_notes(vec![note_3.clone(), note_4.clone()]);
        sequence.insert_notes(vec![note_5.clone()]);
        
        assert_eq!(sequence.get_notes_at(0), vec![note_1, note_2]);
        assert_eq!(sequence.get_notes_at(1), vec![note_5]);
        assert_eq!(sequence.get_notes_at(2), vec![note_3, note_4]);
    }

    #[test]
    fn test_insert_note_increment_decrement_get_note_at() {
        let note_1= playback_note::from_note(
            setup_note()
                .start_time_ms(1.0)
                .build().unwrap()
        );
        let note_2= playback_note::from_note(
            setup_note()
                .start_time_ms(2.0)
                .build().unwrap()
        );
        let note_3= playback_note::from_note(
            setup_note()
                .start_time_ms(0.0)
                .build().unwrap()
        );

        let mut sequence = GridNoteSequenceBuilder::default()
            .index(0)
            .build().unwrap();
        sequence.insert_note(note_1.clone());
        sequence.increment();
        sequence.insert_note_at(note_2.clone(), 1);
        sequence.decrement();
        sequence.insert_note(note_3.clone());

        // inserted at 0 and pushed to 1 by inserting note_3 at 0
        assert_eq!(sequence.get_note_at(0), note_3);
        // inserted at 1 and pushed to 2 by insert of note_2 at 1 and note_3 at 0
        assert_eq!(sequence.get_note_at(1), note_1);
        // inserted at 1 after note_2
        assert_eq!(sequence.get_note_at(2), note_2);
    }

    #[test]
    #[should_panic(expected = "Notes to add must not be empty")]
    fn test_append_empty_notes() {
        let mut sequence = GridNoteSequenceBuilder::default()
            .index(0)
            .build().unwrap();

        sequence.append_notes(&vec![]);
    }

    #[test]
    #[should_panic(expected = "Notes to add must not be empty")]
    fn test_insert_empty_notes() {
        let mut sequence = GridNoteSequenceBuilder::default()
            .index(0)
            .build().unwrap();

        sequence.insert_notes(vec![]);
    }

    #[test]
    #[should_panic(expected = "Note start time must be >= 0.0")]
    fn test_insert_invalid_note() {
        let note_1= playback_note::from_note(
            setup_note()
                .start_time_ms(-1.0)
                .build().unwrap()
        );

        let mut sequence = GridNoteSequenceBuilder::default()
            .index(0)
            .build().unwrap();

        sequence.insert_note(note_1);
    }

    fn setup_note() -> NoteBuilder {
        NoteBuilder::default()
            .clone()
    }
}
