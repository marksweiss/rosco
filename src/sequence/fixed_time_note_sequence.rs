use derive_builder::Builder;
use crate::meter::durations::DurationType;
use crate::sequence::time_note_sequence::{TimeNoteSequence};
use crate::sequence::note_sequence_trait::{AppendNote, BuilderWrapper, NextNotes, SetCurPosition};

#[allow(dead_code)]
#[derive(Builder, Clone, Debug)]
pub(crate) struct FixedTimeNoteSequence {
    #[builder(default)]
    inner_sequence: TimeNoteSequence,
    
    #[builder(default = "DurationType::Quarter", setter(custom))]
    duration_type: DurationType,
    
    #[builder(default = "120", setter(custom))]
    pub(crate) tempo: u8,
    
    #[builder(default = "16")]
    num_steps: usize,
    
    #[builder(default = "0")]
    current_step: usize,
    
    // Computed field based on tempo and duration_type
    #[builder(default = "500.0")]
    step_duration_ms: f32,
}

impl FixedTimeNoteSequenceBuilder {
    pub(crate) fn duration_type(&mut self, duration_type: DurationType) -> &mut Self {
        self.duration_type = Some(duration_type);

        if let Some(tempo) = self.tempo {
            // Recalculate step_duration_ms
            let quarter_note_duration_ms = 60000.0 / tempo as f32;
            let step_duration_ms = quarter_note_duration_ms * (duration_type.to_factor() / 0.25);
            self.step_duration_ms = Some(step_duration_ms);
        }

        self
    }

    pub(crate) fn tempo(&mut self, tempo: u8) -> &mut Self {
        if tempo == 0 {
            panic!("Tempo must be greater than 0");
        }

        // Calculate step_duration_ms based on tempo
        let quarter_note_duration_ms = 60000.0 / tempo as f32;

        // Use the duration_type if set, otherwise it will use the default (Quarter)
        let duration_factor = if let Some(dt) = self.duration_type {
            dt.to_factor() / 0.25
        } else {
            1.0 // Quarter note is the default, so factor is 1.0
        };

        let step_duration_ms = quarter_note_duration_ms * duration_factor;

        self.tempo = Some(tempo);
        self.step_duration_ms = Some(step_duration_ms);
        self
    }
}

impl BuilderWrapper<FixedTimeNoteSequence> for FixedTimeNoteSequenceBuilder {
    fn new() -> FixedTimeNoteSequence {
        FixedTimeNoteSequenceBuilder::default().build().unwrap()
    }
}

impl AppendNote for FixedTimeNoteSequence {
    fn append_note(&mut self, note: crate::note::playback_note::PlaybackNote) {
        self.inner_sequence.append_note(note);
    }
}

impl NextNotes for FixedTimeNoteSequence {
    fn next_notes(&mut self) -> Vec<crate::note::playback_note::PlaybackNote> {
        self.inner_sequence.next_notes()
    }
}

impl SetCurPosition for FixedTimeNoteSequence {
    fn set_cur_position(&mut self, position: f32) {
        self.inner_sequence.set_cur_position(position);
    }
}

impl FixedTimeNoteSequence {
    /// Get all notes from the sequence (for testing purposes)
    #[allow(dead_code)]
    pub(crate) fn get_all_notes(&self) -> Vec<crate::note::playback_note::PlaybackNote> {
        let mut all_notes = Vec::new();
        for i in 0..self.inner_sequence.sequence_len() {
            all_notes.extend(self.inner_sequence.get_notes_at(i));
        }
        all_notes
    }
}

impl Iterator for FixedTimeNoteSequence {
    type Item = Vec<crate::note::playback_note::PlaybackNote>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner_sequence.next()
    }
}