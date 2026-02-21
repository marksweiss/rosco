use std::hash::{Hash, Hasher};

use derive_builder::Builder;

use crate::audio_gen::oscillator::Waveform;
use crate::common::float_utils::float_eq;
use crate::note::constants::{DEFAULT_FREQUENCY, DEFAULT_VOLUME, INIT_START_TIME};
use crate::note::note_trait::BuilderWrapper;

#[derive(Builder, Clone, Debug)]
pub(crate) struct Note {
    #[builder(default = "DEFAULT_FREQUENCY")]
    pub(crate) frequency: f32,

    #[builder(default = "DEFAULT_VOLUME")]
    pub(crate) volume: f32,

    #[builder(default = "INIT_START_TIME")]
    pub(crate) start_time_ms: f32,

    #[builder(default = "INIT_START_TIME")]
    pub(crate) end_time_ms: f32,

    #[builder(default = "Vec::new()")]
    pub(crate) waveforms: Vec<Waveform>,
}

pub(crate) fn default_note() -> Note {
    NoteBuilder::default().build().unwrap()
}

pub(crate) fn rest_note(start_time_ms: f32, end_time_ms: f32) -> Note {
    NoteBuilder::default()
        .start_time_ms(start_time_ms)
        .end_time_ms(end_time_ms)
        .volume(0.0)
        .build().unwrap()
}

impl PartialEq for Note {
    fn eq(&self, other: &Self) -> bool {
        float_eq(self.frequency, other.frequency) &&
            float_eq(self.duration_ms(), other.duration_ms()) &&
            float_eq(self.volume, other.volume) &&
            float_eq(self.start_time_ms, other.start_time_ms)
    }
}
impl Eq for Note {}

impl Hash for Note {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.frequency.to_bits().hash(state);
        self.duration_ms().to_bits().hash(state);
        self.volume.to_bits().hash(state);
        self.start_time_ms.to_bits().hash(state);
    }
}

#[allow(dead_code)]
impl Note {
    pub(crate) fn duration_ms(&self) -> f32 {
        self.end_time_ms - self.start_time_ms
    }

    pub(crate) fn is_playing(&self, time_ms: f32) -> bool {
        time_ms >= self.start_time_ms && time_ms < self.end_time_ms
    }

    pub(crate) fn is_before_playing(&self, time_ms: f32) -> bool {
        time_ms < self.start_time_ms
    }

    pub(crate) fn is_after_playing(&self, time_ms: f32) -> bool {
        time_ms >= self.end_time_ms
    }

    pub(crate) fn duration_position(&self, cur_time_ms: f32) -> f32 {
        (cur_time_ms - self.start_time_ms) / self.duration_ms()
    }
}

impl BuilderWrapper<Note> for NoteBuilder {
    fn new() -> Note {
        NoteBuilder::default().build().unwrap()
    }
}

#[cfg(test)]
mod test_note {
    use crate::note::note::NoteBuilder;

    #[test]
    fn test_is_playing() {
        let note = setup_note()
            .start_time_ms(0.0)
            .build().unwrap();

        assert_eq!(note.is_playing(0.0), true);
        assert_eq!(note.is_playing(500.0), true);
        assert_eq!(note.is_playing(1000.0), false);
    }

    #[test]
    fn test_is_before_playing() {
        let note = setup_note()
            .start_time_ms(0.01)
            .build().unwrap();

        assert_eq!(note.is_before_playing(0.0), true);
        assert_eq!(note.is_before_playing(0.02), false);
    }

    #[test]
    fn test_is_after_playing() {
        let note = setup_note()
            .start_time_ms(0.0)
            .build().unwrap();

        assert_eq!(note.is_after_playing(0.0), false);
        assert_eq!(note.is_after_playing(500.0), false);
        assert_eq!(note.is_after_playing(1000.0), true);
    }

    #[test]
    fn test_duration_position() {
        let note = setup_note()
            .start_time_ms(0.0)
            .build().unwrap();

        assert_eq!(note.duration_position(0.0), 0.0);
        assert_eq!(note.duration_position(500.0), 0.5);
        assert_eq!(note.duration_position(1000.0), 1.0);
    }

    fn setup_note() -> NoteBuilder {
        NoteBuilder::default()
            .end_time_ms(1000.0)
            .volume(1.0)
            .clone()
    }
}
