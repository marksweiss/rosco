use derive_builder::Builder;
use std::sync::{Arc, Mutex};
use crate::effect::delay::Delay;
use crate::envelope::Envelope;
use crate::effect::flanger::Flanger;
use crate::effect::lfo::Lfo;
use crate::filter::low_pass_filter::LowPassFilter;
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

#[derive(Builder, Clone, Debug)]
pub struct PlaybackNote {

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
    pub(crate) lfos: Vec<Lfo>,

    #[builder(default = "Vec::new()")]
    pub(crate) flangers: Vec<Flanger>,

    #[builder(default = "Vec::new()")]
    pub(crate) delays: Vec<Delay>,

    #[builder(default = "Vec::new()")]
    pub(crate) filters: Vec<LowPassFilter>,

    #[builder(default = "Arc::new(Mutex::new(no_op_effects()))")]
    pub(crate) track_effects: Arc<Mutex<TrackEffects>>,

    // TODO enforce -1.0..1.0 with builder validator or custom builder
    #[builder(default = "0.0")]
    pub(crate) panning: f32,

    // TODO enforce 0 or 1 with builder validator or custom builder
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
                {
                    let track_effects = self.track_effects.lock().unwrap();
                    for envelope in track_effects.envelopes.iter() {
                        output_sample = envelope.apply_effect(
                            output_sample, // sample_position);
                            sample_count as f32 /
                                (self.playback_sample_end_time as f32 -
                                    self.playback_sample_start_time as f32));
                    }
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
                {
                    let track_effects = self.track_effects.lock().unwrap();
                    for envelope in track_effects.envelopes.iter() {
                        output_sample = envelope.apply_effect(
                            output_sample,
                            sample_count as f32 /
                                (self.playback_sample_end_time as f32 -
                                    self.playback_sample_start_time as f32));
                    }
                }
            }
        }
        
        for lfo in self.lfos.iter() {
            output_sample = lfo.apply_effect(output_sample, sample_count);
        }

        // Apply track-level effects (requires mutex lock due to shared state)
        {
            let mut track_effects = self.track_effects.lock().unwrap();
            
            // Apply LFOs
            for lfo in track_effects.lfos.iter() {
                output_sample = lfo.apply_effect(output_sample, sample_count);
            }

            // Apply flangers
            for flanger in track_effects.flangers.iter_mut() {
                output_sample = flanger.apply_effect(output_sample, sample_position);
            }
            
            // Apply delays
            for delay in track_effects.delays.iter_mut() {
                output_sample = delay.apply_effect(output_sample, sample_position);
            }

            // Apply filters
            for filter in track_effects.low_pass_filters.iter_mut() {
                output_sample = filter.apply_effect(output_sample, sample_position);
            }
            
            for filter in track_effects.high_pass_filters.iter_mut() {
                output_sample = filter.apply_effect(output_sample, sample_position);
            }
            
            for filter in track_effects.band_pass_filters.iter_mut() {
                output_sample = filter.apply_effect(output_sample, sample_position);
            }
            
            for filter in track_effects.notch_filters.iter_mut() {
                output_sample = filter.apply_effect(output_sample, sample_position);
            }
        }

        // Apply individual note effects  
        for flanger in self.flangers.iter_mut() {
            output_sample = flanger.apply_effect(output_sample, sample_position);
        }
        
        for delay in self.delays.iter_mut() {
            output_sample = delay.apply_effect(output_sample, sample_position);
        }

        // Apply filters before LFOs
        for filter in self.filters.iter_mut() {
            output_sample = filter.apply_effect(output_sample, sample_position);
        }

        output_sample
    }

    pub(crate) fn apply_effects_stereo(&mut self, sample: f32, sample_position: f32,
                                sample_count: u64) -> (f32, f32) {
        let mut left = self.apply_effects(sample, sample_position, sample_count);
        let mut right = self.apply_effects(sample, sample_position, sample_count);

        // Apply both per-note and track-level panning
        let factor = 1.0;
        if self.panning > 0.0 {
            left *= factor - (factor * self.panning.cos());
            right *= factor + (factor * self.panning.sin());
        } else if self.panning < 0.0 {
            left *= factor + (factor *self.panning.cos());
            right *= factor - (factor *self.panning.sin());
        }
        {
            let track_effects = self.track_effects.lock().unwrap();
            if track_effects.panning > 0.0 {
                left *= factor - (factor * track_effects.panning.cos());
                right *= factor + (factor * track_effects.panning.sin());
            } else if track_effects.panning < 0.0 {
                left *= factor + (factor * track_effects.panning.cos());
                right *= factor - (factor * track_effects.panning.sin());
            }
        }
        
        (left, right)
    }
}

// Custom PartialEq implementation to handle Arc<Mutex<TrackEffects>>
impl PartialEq for PlaybackNote {
    fn eq(&self, other: &Self) -> bool {
        // Compare all fields except track_effects
        if self.note_type != other.note_type ||
           self.note != other.note ||
           self.sampled_note != other.sampled_note ||
           self.playback_start_time_ms != other.playback_start_time_ms ||
           self.playback_end_time_ms != other.playback_end_time_ms ||
           self.playback_sample_start_time != other.playback_sample_start_time ||
           self.playback_sample_end_time != other.playback_sample_end_time ||
           self.envelopes != other.envelopes ||
           self.lfos != other.lfos ||
           self.flangers != other.flangers ||
           self.delays != other.delays ||
           self.filters != other.filters ||
           self.panning != other.panning ||
           self.num_channels != other.num_channels {
            return false;
        }
        
        // Compare track_effects by locking both mutexes
        // Note: This could potentially deadlock in theory, but in practice
        // it should be safe for testing purposes
        let self_effects = self.track_effects.lock().unwrap();
        let other_effects = other.track_effects.lock().unwrap();
        *self_effects == *other_effects
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

    #[test]
    fn test_playback_note_with_filters() {
        let playback_note = PlaybackNoteBuilder::default()
            .filters(vec![crate::filter::low_pass_filter::default_low_pass_filter()])
            .build().unwrap();
        assert_eq!(playback_note.filters.len(), 1);
    }
}
