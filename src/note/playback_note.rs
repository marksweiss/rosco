use derive_builder::Builder;
use crate::effect::delay::Delay;
use crate::envelope::envelope::Envelope;
use crate::effect::flanger::Flanger;
use crate::effect::lfo::LFO;
use crate::filter::low_pass_filter::LowPassFilter;
use crate::note::constants;
use crate::note::note;
use crate::note::note::Note;
use crate::note::note_trait::BuilderWrapper;
use crate::note::sampled_note::SampledNote;
use crate::track::track_effects::{no_op_effects, TrackEffects};

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub (crate) enum NoteType {
    Oscillator,
    Sample,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum NoteSource {
    Oscillator(Note),
    Sample(SampledNote),
}

#[derive(Builder, Clone, Debug, PartialEq)]
pub struct PlaybackNote {

    #[builder(default = "NoteSource::Oscillator(note::default_note())")]
    pub(crate) note_source: NoteSource,

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

    #[builder(default = "Vec::new()")]
    pub(crate) filters: Vec<LowPassFilter>,

    #[builder(default = "no_op_effects()")]
    pub(crate) track_effects: TrackEffects,

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

    pub(crate) fn note(&self) -> Option<&Note> {
        match &self.note_source {
            NoteSource::Oscillator(n) => Some(n),
            _ => None,
        }
    }

    pub(crate) fn note_mut(&mut self) -> Option<&mut Note> {
        match &mut self.note_source {
            NoteSource::Oscillator(n) => Some(n),
            _ => None,
        }
    }

    pub(crate) fn sampled_note(&self) -> Option<&SampledNote> {
        match &self.note_source {
            NoteSource::Sample(s) => Some(s),
            _ => None,
        }
    }

    pub(crate) fn sampled_note_mut(&mut self) -> Option<&mut SampledNote> {
        match &mut self.note_source {
            NoteSource::Sample(s) => Some(s),
            _ => None,
        }
    }

    pub(crate) fn is_oscillator(&self) -> bool {
        matches!(&self.note_source, NoteSource::Oscillator(_))
    }

    pub(crate) fn is_sample(&self) -> bool {
        matches!(&self.note_source, NoteSource::Sample(_))
    }

    pub(crate) fn note_start_time_ms(&self) -> f32 {
        match &self.note_source {
            NoteSource::Oscillator(n) => n.start_time_ms,
            NoteSource::Sample(s) => s.start_time_ms,
        }
    }

    pub(crate) fn set_note_start_time_ms(&mut self, start_time_ms: f32) {
        match &mut self.note_source {
            NoteSource::Oscillator(n) => n.start_time_ms = start_time_ms,
            NoteSource::Sample(s) => s.start_time_ms = start_time_ms,
        }
    }

    pub(crate) fn note_end_time_ms(&self) -> f32 {
        match &self.note_source {
            NoteSource::Oscillator(n) => n.end_time_ms,
            NoteSource::Sample(s) => s.end_time_ms,
        }
    }

    pub(crate) fn set_note_end_time_ms(&mut self, end_time_ms: f32) {
        match &mut self.note_source {
            NoteSource::Oscillator(n) => n.end_time_ms = end_time_ms,
            NoteSource::Sample(s) => s.end_time_ms = end_time_ms,
        }
    }

    pub(crate) fn note_duration_ms(&self) -> f32 {
        match &self.note_source {
            NoteSource::Oscillator(n) => n.duration_ms(),
            NoteSource::Sample(s) => s.duration_ms(),
        }
    }

    pub(crate) fn note_volume(&self) -> f32 {
        match &self.note_source {
            NoteSource::Oscillator(n) => n.volume,
            NoteSource::Sample(s) => s.volume,
        }
    }

    pub (crate) fn set_note_volume(&mut self, volume: f32) {
        match &mut self.note_source {
            NoteSource::Oscillator(n) => n.volume = volume,
            NoteSource::Sample(s) => s.volume = volume,
        }
    }

    pub(crate) fn apply_effects(&mut self, sample: f32, sample_position: f32,
                                sample_count: u64) -> f32 {
        let mut output_sample = sample;

        let envelope_position = sample_count as f32 /
            (self.playback_sample_end_time as f32 -
                self.playback_sample_start_time as f32);
        for envelope in self.envelopes.iter() {
            output_sample = envelope.apply_effect(output_sample, envelope_position);
        }
        for envelope in self.track_effects.envelopes.iter() {
            output_sample = envelope.apply_effect(output_sample, envelope_position);
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

        // Apply filters after envelopes, LFOs, flangers, and delays
        for filter in self.filters.iter_mut() {
            output_sample = filter.apply_effect(output_sample, sample_position);
        }

        output_sample
    }

    pub(crate) fn apply_effects_stereo(&mut self, sample: f32, sample_position: f32,
                                sample_count: u64) -> (f32, f32) {
        // Apply effects once to avoid corrupting stateful effects (filters, delays, flangers)
        let processed = self.apply_effects(sample, sample_position, sample_count);
        let mut left = processed;
        let mut right = processed;

        // Apply both per-note and track-level panning
        let factor = 1.0;
        if self.panning > 0.0 {
            left *= factor - (factor * self.panning.cos());
            right *= factor + (factor * self.panning.sin());
        } else if self.panning < 0.0 {
            left *= factor + (factor * self.panning.cos());
            right *= factor - (factor * self.panning.sin());
        }
        if self.track_effects.panning > 0.0 {
            left *= factor - (factor * self.track_effects.panning.cos());
            right *= factor + (factor * self.track_effects.panning.sin());
        } else if self.track_effects.panning < 0.0 {
            left *= factor + (factor * self.track_effects.panning.cos());
            right *= factor - (factor * self.track_effects.panning.sin());
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
        .note_source(NoteSource::Oscillator(note::rest_note(start_time_ms, end_time_ms)))
        .playback_start_time_ms(start_time_ms)
        .playback_end_time_ms(end_time_ms)
        .build().unwrap()
}

#[allow(dead_code)]
pub(crate) fn from_note(note: Note) -> PlaybackNote {
    PlaybackNoteBuilder::default()
        .note_source(NoteSource::Oscillator(note))
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
        assert_eq!(playback_note.note().unwrap(), &note::default_note());
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
