use derive_builder::Builder;
use crate::common::float_utils::float_geq;
use crate::effect::delay::Delay;
use crate::envelope::envelope::Envelope;
use crate::effect::flanger::Flanger;
use crate::effect::lfo::LFO;
use crate::note::constants;
use crate::note::note;
use crate::note::note::Note;
use crate::note::note_trait::BuilderWrapper;
use crate::note::sampled_note;
use crate::note::sampled_note::SampledNote;
use crate::track::track_effects::{no_op_effects, TrackEffects};

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub (crate) enum NoteType {
    Oscillator,
    Sample,
}

#[derive(Builder, Clone, Debug, PartialEq)]
pub(crate) struct PlaybackNote {

    #[builder(default = "NoteType::Oscillator")]
    pub(crate) note_type: NoteType,
    
    #[builder(default = "note::default_note()")]
    pub(crate) note: Note,

    #[builder(default = "sampled_note::default_sample_note()")]
    pub(crate) sampled_note: SampledNote,

    #[builder(default = "constants::INIT_START_TIME")]
    pub(crate) playback_start_time_ms: f32,

    #[builder(default = "constants::INIT_END_TIME")]
    pub(crate) playback_end_time_ms: f32,

    #[builder(default = "0")]
    pub(crate) playback_sample_start_time: u64,
    #[builder(default = "0")]
    pub(crate) playback_sample_end_time: u64,

    #[builder(default = "Vec::new()")]
    pub(crate) envelopes: Vec<Envelope>,

    #[builder(default = "Vec::new()")]
    pub(crate) lfos: Vec<LFO>,

    #[builder(default = "Vec::new()")]
    pub(crate) flangers: Vec<Flanger>,

    #[builder(default = "Vec::new()")]
    pub(crate) delays: Vec<Delay>,

    #[builder(default = "no_op_effects()")]
    pub(crate) track_effects: TrackEffects,
    
    #[builder(default = "0.0")]
    pub(crate) panning: f32,

    #[builder(default = "1")]
    pub(crate) num_channels: i8,
}

#[allow(dead_code)]
impl PlaybackNote {
    pub(crate) fn playback_duration_ms(&self) -> f32 {
        self.playback_end_time_ms - self.playback_start_time_ms
    }

    pub(crate) fn note_start_time_ms(&self) -> f32 {
        match self.note_type {
            NoteType::Oscillator => self.note.start_time_ms,
            NoteType::Sample => self.sampled_note.start_time_ms,
        }
    }

    pub(crate) fn set_note_start_time_ms(&mut self, start_time_ms: f32) {
        match self.note_type {
            NoteType::Oscillator => self.note.start_time_ms = start_time_ms,
            NoteType::Sample => self.sampled_note.start_time_ms = start_time_ms,
        }
    }

    pub(crate) fn note_end_time_ms(&self) -> f32 {
        match self.note_type {
            NoteType::Oscillator => self.note.end_time_ms,
            NoteType::Sample => self.sampled_note.end_time_ms,
        }
    }

    pub(crate) fn set_note_end_time_ms(&mut self, end_time_ms: f32) {
        match self.note_type {
            NoteType::Oscillator => self.note.end_time_ms = end_time_ms,
            NoteType::Sample => self.sampled_note.end_time_ms = end_time_ms,
        }
    }

    pub(crate) fn note_duration_ms(&self) -> f32 {
        match self.note_type {
            NoteType::Oscillator => self.note.duration_ms(),
            NoteType::Sample => self.sampled_note.duration_ms(),
        }
    }

    pub(crate) fn note_volume(&self) -> f32 {
        match self.note_type {
            NoteType::Oscillator => self.note.volume,
            NoteType::Sample => self.sampled_note.volume,
        }
    }

    pub (crate) fn set_note_volume(&mut self, volume: f32) {
        match self.note_type {
            NoteType::Oscillator => self.note.volume = volume,
            NoteType::Sample => self.sampled_note.volume = volume,
        }
    }

    pub(crate) fn apply_effects(&mut self, sample: f32, sample_position: f32,
                                sample_count: u64) -> f32 {
        let mut output_sample = sample;

        match self.note_type {
            
            NoteType::Oscillator => {
                for envelope in self.envelopes.iter() {
                    output_sample = envelope.apply_effect(
                        output_sample, // sample_position);
                        sample_count as f32 /
                            (self.playback_sample_end_time as f32 -
                                self.playback_sample_start_time as f32));
                }
                for envelope in self.track_effects.envelopes.iter() {
                    output_sample = envelope.apply_effect(
                        output_sample, // sample_position);
                        sample_count as f32 /
                            (self.playback_sample_end_time as f32 -
                                self.playback_sample_start_time as f32));
                }
            }
            
            NoteType::Sample => { 
                for envelope in self.envelopes.iter() {
                    output_sample = envelope.apply_effect(
                        output_sample,
                        sample_count as f32 /
                            (self.playback_sample_end_time as f32 -
                                self.playback_sample_start_time as f32));
                }
                for envelope in self.track_effects.envelopes.iter() {
                    output_sample = envelope.apply_effect(
                        output_sample,
                        sample_count as f32 /
                            (self.playback_sample_end_time as f32 -
                                self.playback_sample_start_time as f32));
                }
            }
        }
        
        for lfo in self.lfos.iter() {
            output_sample = lfo.apply_effect(output_sample, sample_count);
        }

        for lfo in self.track_effects.lfos.iter() {
            output_sample = lfo.apply_effect(output_sample, sample_count);
        }

        for flanger in self.flangers.iter_mut() {
            output_sample = flanger.apply_effect(output_sample, sample_position);
        }
        
        for flanger in self.track_effects.flangers.iter_mut() {
            output_sample = flanger.apply_effect(output_sample, sample_position);
        }
        
        for delay in self.delays.iter_mut() {
            output_sample = delay.apply_effect(output_sample, sample_position);
        }

        for delay in self.track_effects.delays.iter_mut() {
            output_sample = delay.apply_effect(output_sample, sample_position);
        }

        output_sample
    }

    pub(crate) fn apply_effects_stereo(&mut self, sample: f32, sample_position: f32,
                                sample_count: u64) -> (f32, f32) {
        let mut left = self.apply_effects(sample, sample_position, sample_count);
        let mut right = self.apply_effects(sample, sample_position, sample_count);
        if float_geq(self.panning, 0.0) {
            left *= 1.0 - self.panning / 2.0;
            right *= 1.0 + self.panning / 2.0;
        } else if self.panning < 0.0 {
            left *= 1.0 + self.panning / 2.0;
            right *= 1.0 - self.panning / 2.0;
        }
        (left, right) 
    }
}

#[allow(dead_code)]
pub(crate) fn default_playback_note() -> PlaybackNote {
    PlaybackNoteBuilder::default().build().unwrap()
}

#[allow(dead_code)]
pub(crate) fn playback_rest_note(start_time_ms: f32, end_time_ms: f32) -> PlaybackNote {
    PlaybackNoteBuilder::default()
        .note_type(NoteType::Oscillator)
        .note(note::rest_note(start_time_ms, end_time_ms))
        .playback_start_time_ms(start_time_ms)
        .playback_end_time_ms(end_time_ms)
        .build().unwrap()
}

#[allow(dead_code)]
pub(crate) fn from_note(note_type: NoteType, note: Note) -> PlaybackNote {
    PlaybackNoteBuilder::default()
        .note_type(note_type)
        .note(note)
        .build().unwrap()
}

impl BuilderWrapper<PlaybackNote> for PlaybackNoteBuilder {
    fn new() -> PlaybackNote {
        PlaybackNoteBuilder::default().build().unwrap()
    }
}

#[cfg(test)]
mod test_playback_note {
    use crate::envelope::envelope;
    use crate::effect::{delay, flanger};
    use crate::effect::lfo;
    use crate::note::constants;
    use crate::note::note;
    use crate::note::playback_note::PlaybackNoteBuilder;

    #[test]
    fn test_default_playback_note() {
        let playback_note = PlaybackNoteBuilder::default().build().unwrap();
        assert_eq!(playback_note.note, note::default_note());
        assert_eq!(playback_note.playback_start_time_ms, constants::INIT_START_TIME);
        assert_eq!(playback_note.playback_end_time_ms, constants::INIT_END_TIME);
        assert_eq!(playback_note.playback_duration_ms(), constants::DEFAULT_DURATION);
        assert_eq!(playback_note.envelopes.is_empty(), true);
        assert_eq!(playback_note.lfos.is_empty(), true);
        assert_eq!(playback_note.flangers.is_empty(), true);
        assert_eq!(playback_note.delays.is_empty(), true);
    }

    #[test]
    fn test_playback_note_with_envelope() {
        let playback_note = PlaybackNoteBuilder::default()
            .envelopes(vec![envelope::default_envelope()])
            .build().unwrap();
        assert_eq!(playback_note.envelopes, vec![envelope::default_envelope()]);
    }
    
    #[test]
    fn test_playback_note_with_lfos() {
        let playback_note = PlaybackNoteBuilder::default()
            .lfos(vec![lfo::default_lfo()])
            .build().unwrap();
        assert_eq!(playback_note.lfos, vec![lfo::default_lfo()]);
    }

    #[test]
    fn test_playback_note_with_flangers() {
        let playback_note = PlaybackNoteBuilder::default()
            .flangers(vec![flanger::default_flanger()])
            .build().unwrap();
        assert_eq!(playback_note.flangers, vec![flanger::default_flanger()]);
    }

    #[test]
    fn test_playback_note_with_delays() {
        let playback_note = PlaybackNoteBuilder::default()
            .delays(vec![delay::default_delay()])
            .build().unwrap();
        assert_eq!(playback_note.delays, vec![delay::default_delay()]);
    }
}
