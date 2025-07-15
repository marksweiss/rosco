use std::collections::VecDeque;
use derive_builder::Builder;

use crate::common::float_utils::{float_eq, float_geq, float_leq};
use crate::note::playback_note;
use crate::note::playback_note::PlaybackNote;
use crate::sequence::note_sequence_trait::{AppendNote, AppendNotes, BuilderWrapper, IterMutWrapper, NextNotes, SetCurPosition};

#[allow(dead_code)]
static INIT_START_TIME: f32 = 0.0;

/**
A sequence of notes that are played back in time order. Notes are grouped by start time. A frontier
of indexes is maintained that contains all notes that have not completed playing.
*/
#[derive(Builder, Clone, Debug)]
#[derive(Default)]
pub(crate) struct TimeNoteSequence {
    #[builder(default = "Vec::new()")]
    sequence: Vec<Vec<PlaybackNote>>,

    #[builder(default = "0.0")]
    cur_position_ms: f32,

    // All positions in the grid before this have end times earlier than next_notes_time_ms
    // Allows O(1) access to scan for next notes window vs. always scanning from the beginning
    #[builder(default = "VecDeque::new()")]
    frontier_indexes: VecDeque<usize>,
}

impl AppendNote for TimeNoteSequence {
    fn append_note(&mut self, note: PlaybackNote) {
        self.append_notes(&vec![note]);
    }
}

impl AppendNotes for TimeNoteSequence {
    fn append_notes(&mut self, notes: &Vec<PlaybackNote>) {
        self.append_notes(notes);
    }
}

impl NextNotes for TimeNoteSequence {
    fn next_notes(&mut self) -> Vec<PlaybackNote> {
        self.get_next_notes_window()
    }
}

impl BuilderWrapper<TimeNoteSequence> for TimeNoteSequenceBuilder {
    fn new () -> TimeNoteSequence {
        TimeNoteSequenceBuilder::default().build().unwrap()
    }
}

impl SetCurPosition for TimeNoteSequence {
    fn set_cur_position(&mut self, position: f32) {
        self.cur_position_ms = position;
    }
}

impl IterMutWrapper for TimeNoteSequence {
    fn iter_mut(&mut self) -> std::slice::IterMut<Vec<PlaybackNote>> {
        self.sequence.iter_mut()
    }
}

#[allow(dead_code)]
impl TimeNoteSequence {

    // Manage PlaybackNotes
    pub(crate) fn append_notes(&mut self, playback_notes: &Vec<PlaybackNote>) {
        self.validate_notes_to_add(&playback_notes);

        if self.frontier_indexes.is_empty() {
            self.sequence.push(playback_notes.clone());
            // Went from no indexes with notes to the 0th index now has notes, start of frontier
            self.frontier_indexes.push_back(0);
            return;
        }

        // Maintain the invariant that all notes with the same start_time are grouped in one
        // note sequence at one index, so if these notes have the same start_time as current
        // notes in last position, add these to that position
        // For the max index in the frontier, check for same start time -- this is the supported
        // semantics for append, which means "add to the end"
        let max_frontier_index = self.frontier_indexes[self.frontier_indexes.len() - 1];
        let min_frontier_start_time_ms = self.get_frontier_min_start_time();
        if float_eq(min_frontier_start_time_ms, playback_notes[0].note_start_time_ms()) {
            self.sequence[max_frontier_index].append(&mut playback_notes.clone());
        } else {
            if min_frontier_start_time_ms > playback_notes[0].note_start_time_ms() {
                panic!("PlaybackNotes must be appended sorted by start time");
            }
            self.sequence.push(playback_notes.clone());
            self.frontier_indexes.push_back(max_frontier_index + 1);
        }
    }

    pub(crate) fn append_note(&mut self, playback_note: PlaybackNote) {
        self.append_notes(&vec![playback_note]);
    }

    pub(crate) fn insert_notes(&mut self, playback_notes: Vec<PlaybackNote>) {
        self.validate_notes_to_add(&playback_notes);

        // Find insert position where existing notes at position have same start time as notes
        // to insert, or notes at existing position have greater start time as notes to insert
        let mut insert_position: usize = 0;
        let notes_start_time_ms = playback_notes[0].note_start_time_ms();
        let mut inserted = false;
        while insert_position < self.sequence.len() {
            let min_start_time_ms = self.get_min_start_time(insert_position);

            if float_eq(min_start_time_ms, notes_start_time_ms) {
                self.sequence[insert_position].append(&mut playback_notes.clone());
                inserted = true;
                break;
            }
            if min_start_time_ms > notes_start_time_ms {
                // Move all notes from this position until self.index one position forward
                self.sequence.insert(insert_position, playback_notes.clone());
                inserted = true;
                break;
            }

            insert_position += 1;
        }
        if !inserted {
            self.sequence.push(playback_notes.clone());
            self.frontier_indexes.push_back(insert_position);
        }
    }

    pub(crate) fn insert_note(&mut self, playback_note: PlaybackNote) {
        self.insert_notes(vec![playback_note]);
    }

    pub(crate) fn insert_notes_multi_position(&mut self, playback_notes: Vec<PlaybackNote>) {
        playback_notes.iter().for_each(|playback_note| {
            self.insert_note((*playback_note).clone());
        })
    }

    pub(crate) fn get_next_notes_window(&mut self) -> Vec<PlaybackNote> {
        fn note_ref_into_note(playback_note: &PlaybackNote, cur_notes_time_ms: f32,
                              window_end_time_ms: f32) -> PlaybackNote {
            let mut new_playback_note: PlaybackNote = playback_note.clone();
            new_playback_note.playback_start_time_ms = cur_notes_time_ms;
            new_playback_note.playback_end_time_ms = window_end_time_ms;
            new_playback_note
        }
        
        let mut window_playback_notes = Vec::new();
        self.remove_completed_frontier_indexes();
        if self.frontier_indexes.is_empty() {
            return window_playback_notes;
        }

        let window_start_time_ms = self.get_frontier_min_start_time();
        let window_end_time_ms = self.get_frontier_min_end_time(self.cur_position_ms);
        // If the current note time is earlier than that, emit a rest note and increment
        // the current notes time to the frontier min start time + epsilon
        if self.cur_position_ms < window_start_time_ms {
            window_playback_notes.push(
                playback_note::playback_rest_note(self.cur_position_ms,
                    window_start_time_ms)
            );
            self.cur_position_ms = window_start_time_ms;

            return window_playback_notes;
        }

        // If the current note time is the same as the frontier min start time, emit all notes
        // in the frontier with the same start time and increment the current notes time to the
        // earliest end time in the frontier. This is the next window emit, note to end time.
        if float_eq(self.cur_position_ms, window_start_time_ms) {
            let playback_notes: Vec<PlaybackNote> = self.get_frontier_notes()
                .iter()
                .flatten()
                .filter(|playback_note|
                    float_eq(playback_note.note_start_time_ms(), self.cur_position_ms)
                )
                .map(|playback_note|
                    note_ref_into_note(playback_note,
                                       self.cur_position_ms, window_end_time_ms)
                )
                .collect();

            window_playback_notes.extend_from_slice(&playback_notes);

            // if notes_time_ms is greater than the frontier min start time, get all notes in the
            // frontier that are playing at the current notes time and emit them up to end time
            // as the next window and increment the current notes time to the end time
        } else if self.cur_position_ms > window_start_time_ms {
            let playback_notes: Vec<PlaybackNote> = self.get_frontier_notes()
                .iter()
                .flatten()
                .filter(|playback_note|
                    float_leq(playback_note.note_start_time_ms(), self.cur_position_ms) &&
                    float_geq(playback_note.note_end_time_ms(), self.cur_position_ms)
                )
                .filter(|playback_note| playback_note.note_duration_ms() > 0.0)
                .map(|playback_note|
                    note_ref_into_note(playback_note, self.cur_position_ms,
                                       window_end_time_ms)
                )
                .collect();

            window_playback_notes.extend_from_slice(&playback_notes);
        }

        self.cur_position_ms = window_end_time_ms;

        window_playback_notes
    }
    
    pub(crate) fn notes_iter_mut(&mut self) -> std::slice::IterMut<Vec<PlaybackNote>> {
        self.sequence.iter_mut()
    }

    fn get_frontier_notes(&self) ->  &[Vec<PlaybackNote>] {
        let min_frontier_index = self.frontier_indexes[0];
        let max_frontier_index = self.frontier_indexes[self.frontier_indexes.len() - 1];

        &self.sequence[min_frontier_index..(max_frontier_index + 1)]
    }

    fn remove_completed_frontier_indexes(&mut self) {
        let mut frontier_indexes_to_pop: usize = 0;
        // Loop over the notes in the position, if any have an end time later than current
        // note time, then the note hasn't been completed yet so the index is still active.
        // OTOH if all notes at an index have end times <= note_time_ms, that index is done
        for i in 0..self.frontier_indexes.len() {
            if self.sequence[self.frontier_indexes[i]].iter().all(
                    |playback_note|
                    float_leq(playback_note.note_end_time_ms(), self.cur_position_ms)) {
                frontier_indexes_to_pop += 1;
            }
        }

        for _ in 0..frontier_indexes_to_pop {
            self.frontier_indexes.pop_front();
        }
    }

    fn get_frontier_min_start_time(&self) -> f32 {
        // Get the earliest start time of all notes in the frontier
        let mut start_time_ms = f32::MAX;
        for playback_note in self.get_frontier_notes().iter().flatten() {
            if playback_note.note_start_time_ms() < start_time_ms {
                start_time_ms = playback_note.note_start_time_ms();
            }
        }
        start_time_ms
    }

    fn get_min_start_time(&self, index: usize) -> f32 {
        // Get the earliest start time of all notes in the frontier
        let mut min_start_time_ms = f32::MAX;
        for playback_note in self.sequence[index].iter() {
            if playback_note.note_start_time_ms() < min_start_time_ms {
                min_start_time_ms = playback_note.note_start_time_ms();
            }
        }
        min_start_time_ms
    }

    fn get_frontier_min_end_time(&self, note_time_ms: f32) -> f32 {
        let mut end_time_ms = f32::MAX;
        
        // First pass, is what is the earliest end time in the future, after note_time_ms
        // for a note that starts on or before note_time_ms and ends after it
        for playback_note in self.get_frontier_notes().iter().flatten() {
            if float_leq(playback_note.note_start_time_ms(), note_time_ms) &&
                    playback_note.note_end_time_ms() > note_time_ms &&
                    playback_note.note_end_time_ms() < end_time_ms {
                end_time_ms = playback_note.note_end_time_ms();
            }
        }

        // Second pass, is there a note that starts after note_time_ms earlier than the
        // earliest end time. Because if there is then that is the end time of this window
        for playback_note in self.get_frontier_notes().iter().flatten() {
            if playback_note.note_start_time_ms() > note_time_ms &&
                    playback_note.note_start_time_ms() < end_time_ms {
                end_time_ms = playback_note.note_start_time_ms();
            }
        }

        end_time_ms
    }

    fn validate_notes_to_add(&self, playback_notes: &Vec<PlaybackNote>) {
        for playback_note in playback_notes {
            if playback_note.note_start_time_ms() < 0.0 {
                panic!("PlaybackNote start time must be >= 0.0");
            }
        }
    }

    // #VisibleForTesting
    pub(crate) fn get_notes_at(&self, index: usize) -> Vec<PlaybackNote> {
        self.sequence[index].clone()
    }

    // #VisibleForTesting
    pub(crate) fn sequence_len(&self) -> usize {
        self.sequence.len()
    }
}

// Custom iterator for TrackGrid over the note_windows in the grid
impl<'a> Iterator for TimeNoteSequence {
    type Item = Vec<PlaybackNote>;

    fn next(&mut self) -> Option<Self::Item> {
        let notes_window = self.get_next_notes_window();
        if notes_window.is_empty() {
            return None;
        }

        Some(notes_window)
    }
}


#[cfg(test)]
mod test_time_note_sequence {
    use crate::common::float_utils::assert_float_eq;
    use crate::note::note::NoteBuilder;
    use crate::note::playback_note;
    use crate::note::playback_note::NoteType;
    use crate::sequence::time_note_sequence::TimeNoteSequenceBuilder;

    #[test]
    fn test_get_next_notes_window() {
        let mut pb_note_1 = playback_note::from_note(
            NoteType::Oscillator,
            setup_note()
                .start_time_ms(0.0)
                .end_time_ms(1000.0)
                .build().unwrap()
        );
        pb_note_1.playback_start_time_ms = 0.0;
        pb_note_1.playback_end_time_ms = 1000.0;
        let mut pb_note_2 = playback_note::from_note(
            NoteType::Oscillator,
            setup_note()
                .start_time_ms(500.0)
                .end_time_ms(1500.0)
                .build().unwrap()
        );
        pb_note_2.playback_start_time_ms = 500.0;
        pb_note_2.playback_end_time_ms = 1500.0;
        let mut pb_note_3 = playback_note::from_note(
            NoteType::Oscillator,
            setup_note()
                .start_time_ms(1000.0)
                .end_time_ms(2000.0)
                .build().unwrap()
        );
        pb_note_3.playback_start_time_ms = 1000.0;
        pb_note_3.playback_end_time_ms = 2000.0;
        let mut pb_note_4 = playback_note::from_note(
            NoteType::Oscillator,
            setup_note()
                .start_time_ms(1000.0)
                .end_time_ms(2000.0)
                .build().unwrap()
        );
        pb_note_4.playback_start_time_ms = 1000.0;
        pb_note_4.playback_end_time_ms = 2000.0;
        let mut pb_note_5 = playback_note::from_note(
            NoteType::Oscillator,
            setup_note()
                .start_time_ms(2500.0)
                .end_time_ms(3500.0)
                .build().unwrap()
        );
        pb_note_5.playback_start_time_ms = 2500.0;
        pb_note_5.playback_end_time_ms = 3500.0;
        
        let mut sequence = TimeNoteSequenceBuilder::default().build().unwrap();

        sequence.append_note(pb_note_1.clone());
        sequence.append_note(pb_note_2.clone());
        sequence.append_notes(&vec![pb_note_3.clone(), pb_note_4.clone()]);
        sequence.append_note(pb_note_5.clone());

        assert_eq!(sequence.frontier_indexes.len(), 4);
        assert_eq!(sequence.sequence.len(), 4);
        assert_eq!(sequence.sequence[0].len(), 1);
        assert_eq!(sequence.sequence[0][0], pb_note_1);
        assert_eq!(sequence.sequence[1].len(), 1);
        assert_eq!(sequence.sequence[1][0], pb_note_2);
        assert_eq!(sequence.sequence[2].len(), 2);
        assert_eq!(sequence.sequence[2][0], pb_note_3);
        assert_eq!(sequence.sequence[2][1], pb_note_4);
        assert_eq!(sequence.sequence[2].len(), 2);
        assert_eq!(sequence.sequence[3].len(), 1);
        assert_eq!(sequence.sequence[3][0], pb_note_5);

        // 1 start 0 - 500
        let mut pb_notes_window = sequence.get_next_notes_window();
        assert_eq!(pb_notes_window.len(), 1);
        assert_float_eq(pb_notes_window[0].playback_start_time_ms, 0.0);
        assert_float_eq(pb_notes_window[0].playback_end_time_ms, 500.0);
        assert_float_eq(pb_notes_window[0].playback_duration_ms(), 500.0);

        // 1 500 - 1000
        // 2 start 500 - 1000
        pb_notes_window = sequence.get_next_notes_window();
        assert_eq!(pb_notes_window.len(), 2);
        assert_float_eq(pb_notes_window[0].playback_start_time_ms, 500.0);
        assert_float_eq(pb_notes_window[0].playback_end_time_ms, 1000.0);
        assert_float_eq(pb_notes_window[0].playback_duration_ms(), 500.0);
        assert_float_eq(pb_notes_window[1].playback_start_time_ms, 500.0);
        assert_float_eq(pb_notes_window[1].playback_end_time_ms, 1000.0);
        assert_float_eq(pb_notes_window[1].playback_duration_ms(), 500.0);

        // 2 1000 - 1500
        // 3 start 1000 - 1500
        // 4 start 1000 - 1500
        pb_notes_window = sequence.get_next_notes_window();
        assert_eq!(pb_notes_window.len(), 3);
        assert_float_eq(pb_notes_window[0].playback_start_time_ms, 1000.0);
        assert_float_eq(pb_notes_window[0].playback_end_time_ms, 1500.0);
        assert_float_eq(pb_notes_window[0].playback_duration_ms(), 500.0);
        assert_float_eq(pb_notes_window[1].playback_start_time_ms, 1000.0);
        assert_float_eq(pb_notes_window[1].playback_end_time_ms, 1500.0);
        assert_float_eq(pb_notes_window[1].playback_duration_ms(), 500.0);
        assert_float_eq(pb_notes_window[2].playback_start_time_ms, 1000.0);
        assert_float_eq(pb_notes_window[2].playback_end_time_ms, 1500.0);
        assert_float_eq(pb_notes_window[2].playback_duration_ms(), 500.0);

        // 3 1500 - 2000
        // 4 1500 - 2000
        pb_notes_window = sequence.get_next_notes_window();
        assert_eq!(pb_notes_window.len(), 2);
        assert_float_eq(pb_notes_window[0].playback_start_time_ms, 1500.0);
        assert_float_eq(pb_notes_window[0].playback_end_time_ms, 2000.0);
        assert_float_eq(pb_notes_window[0].playback_duration_ms(), 500.0);
        assert_float_eq(pb_notes_window[1].playback_start_time_ms, 1500.0);
        assert_float_eq(pb_notes_window[1].playback_end_time_ms, 2000.0);
        assert_float_eq(pb_notes_window[1].playback_duration_ms(), 500.0);
        
        // Rest 2000 - 2500
        pb_notes_window = sequence.get_next_notes_window();
        assert_eq!(pb_notes_window.len(), 1);
        assert_float_eq(pb_notes_window[0].playback_start_time_ms, 2000.0);
        assert_float_eq(pb_notes_window[0].playback_end_time_ms, 2500.0);
        assert_float_eq(pb_notes_window[0].playback_duration_ms(), 500.0);
        // 0 volume because it is a rest note
        assert_float_eq(pb_notes_window[0].note.volume, 0.0);
        
        // 5 start 2500 - 3500
        pb_notes_window = sequence.get_next_notes_window();
        assert_eq!(pb_notes_window.len(), 1);
        assert_float_eq(pb_notes_window[0].playback_start_time_ms, 2500.0);
        assert_float_eq(pb_notes_window[0].playback_end_time_ms, 3500.0);
        assert_float_eq(pb_notes_window[0].playback_duration_ms(), 1000.0);
    }

    #[test]
    fn test_insert() {
        let note_1= playback_note::from_note(
            NoteType::Oscillator, 
            setup_note()
                .start_time_ms(0.0)
                .build().unwrap()
        );
        let note_2= playback_note::from_note(
            NoteType::Oscillator,
            setup_note()
                .start_time_ms(500.0)
                .build().unwrap()
        );
        let note_3= playback_note::from_note(
            NoteType::Oscillator,
            setup_note()
                .start_time_ms(1000.0)
                .build().unwrap()
        );
        let note_4= playback_note::from_note(
            NoteType::Oscillator,
            setup_note()
                .start_time_ms(1000.0)
                .build().unwrap()
        );
        let note_5= playback_note::from_note(
            NoteType::Oscillator,
            setup_note()
                .start_time_ms(2500.0)
                .build().unwrap()
        );
        let mut sequence = TimeNoteSequenceBuilder::default().build().unwrap();
        
        sequence.insert_note(note_5.clone());
        let mut notes = sequence.get_notes_at(0);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0], note_5);
    
        sequence.insert_note(note_2.clone());
        notes = sequence.get_notes_at(0);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0], note_2);
        notes = sequence.get_notes_at(1);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0], note_5);
    
        sequence.insert_note(note_3.clone());
        sequence.insert_note(note_4.clone());
        notes = sequence.get_notes_at(0);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0], note_2);
        notes = sequence.get_notes_at(1);
        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0], note_3);
        assert_eq!(notes[0], note_4);
        
        sequence.insert_note(note_1.clone());
        notes = sequence.get_notes_at(0);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0], note_1);
    }
    
    #[test]
    fn test_insert_multi_position() {
        let note_1= playback_note::from_note(
            NoteType::Oscillator,
            setup_note()
                .start_time_ms(0.0)
                .build().unwrap()
        );
        let note_2= playback_note::from_note(
            NoteType::Oscillator,
            setup_note()
                .start_time_ms(500.0)
                .build().unwrap()
        );
        let note_3= playback_note::from_note(
            NoteType::Oscillator,
            setup_note()
                .start_time_ms(1000.0)
                .build().unwrap()
        );
        let note_4= playback_note::from_note(
            NoteType::Oscillator,
            setup_note()
                .start_time_ms(1000.0)
                .build().unwrap()
        );
        let note_5= playback_note::from_note(
            NoteType::Oscillator,
            setup_note()
                .start_time_ms(2500.0)
                .build().unwrap()
        );
        let mut sequence = TimeNoteSequenceBuilder::default().build().unwrap();
    
        sequence.insert_notes_multi_position(vec![note_5.clone(), note_2.clone(),
                                                  note_3.clone(), note_4.clone(), note_1.clone()]);
        assert_eq!(sequence.sequence.len(), 4);
        assert_eq!(sequence.sequence[0].len(), 1);
        assert_eq!(sequence.sequence[1].len(), 1);
        assert_eq!(sequence.sequence[2].len(), 2);
        assert_eq!(sequence.sequence[3].len(), 1);
    
        let mut notes = sequence.get_notes_at(0);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0], note_1);
    
        notes = sequence.get_notes_at(1);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0], note_2);
    
        notes = sequence.get_notes_at(2);
        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0], note_3);
        assert_eq!(notes[1], note_4);
    
        notes = sequence.get_notes_at(3);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0], note_5);
    }
    
    fn setup_note() -> NoteBuilder {
        NoteBuilder::default()
            .volume(1.0)
            .clone()
    }
}
