use derive_builder::Builder;
use crate::effect::delay::Delay;
use crate::envelope::envelope::Envelope;
use crate::effect::flanger::Flanger;
use crate::effect::lfo::LFO;

#[derive(Builder, Clone, Debug, PartialEq)]
pub(crate) struct TrackEffects {
    #[allow(dead_code)]
    #[builder(default = "Vec::new()")]
    pub(crate) envelopes: Vec<Envelope>,

    #[allow(dead_code)]
    #[builder(default = "Vec::new()")]
    pub(crate) lfos: Vec<LFO>,

    #[allow(dead_code)]
    #[builder(default = "Vec::new()")]
    pub(crate) flangers: Vec<Flanger>,

    #[allow(dead_code)]
    #[builder(default = "Vec::new()")]
    pub(crate) delays: Vec<Delay>,

    // TODO enforce -1.0..1.0 with builder validator or custom builder
    #[builder(default = "0.0")]
    pub(crate) panning: f32,

    // TODO enforce 0 or 1 with builder validator or custom builder
    #[builder(default = "1")]
    pub(crate) num_channels: i8,
}

pub(crate) fn no_op_effects() -> TrackEffects {
    TrackEffectsBuilder::default().build().unwrap()
}

impl TrackEffects {

    #[allow(dead_code)]
    pub(crate) fn has_envelopes(&self) -> bool {
        !self.envelopes.is_empty()
    }

    #[allow(dead_code)]
    pub(crate) fn has_lfos(&self) -> bool {
        !self.lfos.is_empty()
    }

    #[allow(dead_code)]
    pub(crate) fn has_flangers(&self) -> bool {
        !self.flangers.is_empty()
    }

    #[allow(dead_code)]
    pub(crate) fn has_delays(&self) -> bool {
        !self.delays.is_empty()
    }
    
    #[allow(dead_code)]
    pub(crate) fn has_effects(&self) -> bool {
        self.has_envelopes() || self.has_lfos() || self.has_flangers() || self.has_delays()
    }
}