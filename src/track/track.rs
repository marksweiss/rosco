use derive_builder::Builder;

use crate::common::constants::NO_TRACK;
use crate::track::track_effects;
use crate::track::track_effects::TrackEffects;

static DEFAULT_TRACK_VOLUME: f32 = 1.0;

#[allow(dead_code)]
#[derive(Builder, Clone, Debug)]
pub struct Track<SequenceType> {
    #[builder(default = "NO_TRACK")]
    pub(crate) num: i16,

    #[builder(default = "DEFAULT_TRACK_VOLUME")]
    pub(crate) volume: f32,

    pub(crate) sequence: SequenceType,

    #[builder(default = "track_effects::no_op_effects()")]
    pub(crate) effects: TrackEffects,
}

impl<SequenceType> Track<SequenceType> {}
