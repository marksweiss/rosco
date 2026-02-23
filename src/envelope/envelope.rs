use std::hash::{Hash, Hasher};

use derive_builder::Builder;

use crate::common::float_utils::float_eq;
use crate::envelope::envelope_pair::EnvelopePair;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum EnvelopeCurve {
    Linear,
    Exponential,
}

const DEFAULT_EXP_STEEPNESS: f32 = 5.0;

// State for an ADSR envelope. User sets the position from the start where attack, decay, sustain
// and release end, and the volume level at each of these positions. The envelope defaults to
// starting from (0, 0) and connecting from their to start, and connecting from the position
// of the end of sustain to the end of the note, which is the release.
#[derive(Builder, Clone, Copy, Debug)]
#[builder(build_fn(validate = "Self::validate"))]
pub(crate) struct Envelope {
    #[builder(default = "EnvelopePair(0.0, 0.0)")]
    pub(crate) start: EnvelopePair,

    // These three attributes control the shape of the envelope
    //          attack
    //       -          -   decay  --  sustain  -
    // start                                       release
    pub(crate) attack: EnvelopePair,
    pub(crate) decay: EnvelopePair,
    pub(crate) sustain: EnvelopePair,

    #[builder(default = "EnvelopePair(1.0, 0.0)")]
    pub(crate) release: EnvelopePair,

    #[builder(default = "EnvelopeCurve::Linear")]
    pub(crate) attack_curve: EnvelopeCurve,
    #[builder(default = "EnvelopeCurve::Linear")]
    pub(crate) decay_curve: EnvelopeCurve,
    #[builder(default = "EnvelopeCurve::Linear")]
    pub(crate) sustain_curve: EnvelopeCurve,
    #[builder(default = "EnvelopeCurve::Linear")]
    pub(crate) release_curve: EnvelopeCurve,

    #[builder(default = "DEFAULT_EXP_STEEPNESS")]
    pub(crate) steepness: f32,
}

impl EnvelopeBuilder {
    pub(crate) fn validate(&self) -> Result<Envelope, String> {
        let attack = self.attack.unwrap();
        let decay = self.decay.unwrap();
        let sustain = self.sustain.unwrap();

        if attack.0 > decay.0 || decay.0 > sustain.0  {
            return Err(
                String::from("Envelope: attack, decay, sustain, release must be in order"));
        }
        if attack.0 < 0.0 || attack.0 > 1.0 || attack.1 < 0.0 || attack.1 > 1.0 {
            return Err(
                String::from("Envelope: attack position and volume must be between 0.0 and 1.0"));
        }
        if decay.0 < 0.0 || decay.0 > 1.0 || decay.1 < 0.0 || decay.1 > 1.0 {
            return Err(
                String::from("Envelope: decay position and volume must be between 0.0 and 1.0"));
        }
        if sustain.0 < 0.0 || sustain.0 > 1.0 || sustain.1 < 0.0 || sustain.1 > 1.0 {
            return Err(
                String::from("Envelope: sustain position and volume must be between 0.0 and 1.0"));
        }

        Ok(Envelope {
            start: EnvelopePair(0.0, 0.0),
            attack,
            decay,
            sustain,
            release: EnvelopePair(1.0, 0.0),
            attack_curve: self.attack_curve.unwrap_or(EnvelopeCurve::Linear),
            decay_curve: self.decay_curve.unwrap_or(EnvelopeCurve::Linear),
            sustain_curve: self.sustain_curve.unwrap_or(EnvelopeCurve::Linear),
            release_curve: self.release_curve.unwrap_or(EnvelopeCurve::Linear),
            steepness: self.steepness.unwrap_or(DEFAULT_EXP_STEEPNESS),
        })
    }
}

#[allow(dead_code)]
pub (crate) fn default_envelope() -> Envelope {
    Envelope {
        start: EnvelopePair(0.0, 0.0),
        attack: EnvelopePair(0.02, 1.0),
        decay: EnvelopePair(0.51, 1.0),
        sustain: EnvelopePair(0.98, 1.0),
        release: EnvelopePair(1.0, 0.0),
        attack_curve: EnvelopeCurve::Linear,
        decay_curve: EnvelopeCurve::Linear,
        sustain_curve: EnvelopeCurve::Linear,
        release_curve: EnvelopeCurve::Linear,
        steepness: DEFAULT_EXP_STEEPNESS,
    }
}

impl Envelope {

    // TODO MOVE BOTH TO FREE FUNCTIONS AND JUST TAKE THE ADSR VALUES AS ARGS SO CAN BE
    //  A CLOSURE IN THE gen_notes CALLBACK in audio_gen
    pub(crate) fn volume_factor(&self, position: f32) -> f32 {
        if position < self.attack.0 {
            self.interpolate_segment(self.start, self.attack, position, self.attack_curve)
        } else if position < self.decay.0 {
            self.interpolate_segment(self.attack, self.decay, position, self.decay_curve)
        } else if position < self.sustain.0 {
            self.interpolate_segment(self.decay, self.sustain, position, self.sustain_curve)
        } else {
            self.interpolate_segment(self.sustain, self.release, position, self.release_curve)
        }
    }

    pub(crate) fn apply_effect(&self, sample: f32, position: f32) -> f32 {
       sample * self.volume_factor(position)
    }

    fn interpolate_segment(&self, start: EnvelopePair, end: EnvelopePair,
                           position: f32, curve: EnvelopeCurve) -> f32 {
        match curve {
            EnvelopeCurve::Linear => self.volume_for_segment_position_linear(start, end, position),
            EnvelopeCurve::Exponential => self.volume_for_segment_position_exponential(start, end, position),
        }
    }

    fn volume_for_segment_position_linear(&self, start: EnvelopePair, end: EnvelopePair,
                                          position: f32) -> f32 {
        let start_position = start.0;
        let start_volume = start.1;
        let end_position = end.0;
        let end_volume = end.1;

        let slope = (end_volume - start_volume) / (end_position - start_position);
        let intercept = start_volume - (slope * start_position);
        // y = mx + b, where slope = m and intercept = b, so b = y - mx
        // so the value along the line for any position
        slope * position + intercept
    }

    fn volume_for_segment_position_exponential(&self, start: EnvelopePair, end: EnvelopePair,
                                               position: f32) -> f32 {
        let t = (position - start.0) / (end.0 - start.0); // normalize 0.0-1.0 within segment
        let exp_t = 1.0 - (-self.steepness * t).exp();
        let exp_t_normalized = exp_t / (1.0 - (-self.steepness).exp()); // normalize so t=1.0 maps to 1.0
        start.1 + (end.1 - start.1) * exp_t_normalized
    }
}

impl PartialEq for Envelope {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start &&
            self.attack == other.attack &&
            self.decay == other.decay &&
            self.sustain == other.sustain &&
            self.release == other.release &&
            self.attack_curve == other.attack_curve &&
            self.decay_curve == other.decay_curve &&
            self.sustain_curve == other.sustain_curve &&
            self.release_curve == other.release_curve &&
            float_eq(self.steepness, other.steepness)
    }
}
impl Eq for Envelope {}

impl Hash for Envelope {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.start.hash(state);
        self.attack.hash(state);
        self.decay.hash(state);
        self.sustain.hash(state);
        self.release.hash(state);
        self.attack_curve.hash(state);
        self.decay_curve.hash(state);
        self.sustain_curve.hash(state);
        self.release_curve.hash(state);
        self.steepness.to_bits().hash(state);
    }
}

#[cfg(test)]
mod test_envelope {
    use crate::envelope::envelope::{EnvelopeBuilder, EnvelopeCurve};
    use crate::envelope::envelope_pair::EnvelopePair;
    use crate::common::float_utils::assert_float_eq;

    #[test]
    fn test_volume_factor() {
        let envelope = EnvelopeBuilder::default()
            .attack(EnvelopePair(0.3, 0.9))
            .decay(EnvelopePair(0.35, 0.7))
            .sustain(EnvelopePair(0.6, 0.65))
            .build().unwrap();

        assert_float_eq(envelope.volume_factor(0.0), 0.0);
        assert_float_eq(envelope.volume_factor(0.15), 0.45);
        assert_float_eq(envelope.volume_factor(0.3), 0.9);
        assert_float_eq(envelope.volume_factor(0.35), 0.7);
        assert_float_eq(envelope.volume_factor(0.6), 0.65);
        assert_float_eq(envelope.volume_factor(0.8), 0.325);
        assert_float_eq(envelope.volume_factor(1.0), 0.0);
    }

    #[test]
    fn test_exponential_boundary_values() {
        // Exponential curves must match linear at segment start and end points
        let envelope = EnvelopeBuilder::default()
            .attack(EnvelopePair(0.3, 0.9))
            .decay(EnvelopePair(0.35, 0.7))
            .sustain(EnvelopePair(0.6, 0.65))
            .attack_curve(EnvelopeCurve::Exponential)
            .decay_curve(EnvelopeCurve::Exponential)
            .sustain_curve(EnvelopeCurve::Exponential)
            .release_curve(EnvelopeCurve::Exponential)
            .build().unwrap();

        // Start of envelope
        assert_float_eq(envelope.volume_factor(0.0), 0.0);
        // End of attack / start of decay
        assert_float_eq(envelope.volume_factor(0.3), 0.9);
        // End of decay / start of sustain
        assert_float_eq(envelope.volume_factor(0.35), 0.7);
        // End of sustain / start of release
        assert_float_eq(envelope.volume_factor(0.6), 0.65);
        // End of release
        assert_float_eq(envelope.volume_factor(1.0), 0.0);
    }

    #[test]
    fn test_exponential_mid_segment_faster_rise() {
        // For a rising segment (attack: 0.0 -> 0.9), exponential should produce
        // a higher value at mid-segment than linear (faster initial rise)
        let linear_env = EnvelopeBuilder::default()
            .attack(EnvelopePair(0.3, 0.9))
            .decay(EnvelopePair(0.35, 0.7))
            .sustain(EnvelopePair(0.6, 0.65))
            .build().unwrap();

        let exp_env = EnvelopeBuilder::default()
            .attack(EnvelopePair(0.3, 0.9))
            .decay(EnvelopePair(0.35, 0.7))
            .sustain(EnvelopePair(0.6, 0.65))
            .attack_curve(EnvelopeCurve::Exponential)
            .build().unwrap();

        let mid_position = 0.15; // midpoint of attack segment (0.0 to 0.3)
        let linear_val = linear_env.volume_factor(mid_position);
        let exp_val = exp_env.volume_factor(mid_position);

        // Exponential curve rises faster initially, so mid-segment value should be higher
        assert!(exp_val > linear_val,
            "Exponential mid-segment value ({}) should be > linear ({})", exp_val, linear_val);
    }

    #[test]
    fn test_mixed_curve_types() {
        // Test envelope with different curve types per segment
        let envelope = EnvelopeBuilder::default()
            .attack(EnvelopePair(0.1, 0.8))
            .decay(EnvelopePair(0.3, 0.6))
            .sustain(EnvelopePair(0.8, 0.4))
            .attack_curve(EnvelopeCurve::Exponential)
            .decay_curve(EnvelopeCurve::Linear)
            .sustain_curve(EnvelopeCurve::Linear)
            .release_curve(EnvelopeCurve::Exponential)
            .build().unwrap();

        // Boundary values still hold
        assert_float_eq(envelope.volume_factor(0.0), 0.0);
        assert_float_eq(envelope.volume_factor(0.1), 0.8);
        assert_float_eq(envelope.volume_factor(0.3), 0.6);
        assert_float_eq(envelope.volume_factor(0.8), 0.4);
        assert_float_eq(envelope.volume_factor(1.0), 0.0);

        // Linear decay segment should match linear interpolation
        let linear_env = EnvelopeBuilder::default()
            .attack(EnvelopePair(0.1, 0.8))
            .decay(EnvelopePair(0.3, 0.6))
            .sustain(EnvelopePair(0.8, 0.4))
            .build().unwrap();
        assert_float_eq(envelope.volume_factor(0.2), linear_env.volume_factor(0.2));
    }

    #[test]
    fn test_custom_steepness() {
        let default_env = EnvelopeBuilder::default()
            .attack(EnvelopePair(0.3, 0.9))
            .decay(EnvelopePair(0.35, 0.7))
            .sustain(EnvelopePair(0.6, 0.65))
            .attack_curve(EnvelopeCurve::Exponential)
            .build().unwrap();

        let steep_env = EnvelopeBuilder::default()
            .attack(EnvelopePair(0.3, 0.9))
            .decay(EnvelopePair(0.35, 0.7))
            .sustain(EnvelopePair(0.6, 0.65))
            .attack_curve(EnvelopeCurve::Exponential)
            .steepness(10.0)
            .build().unwrap();

        let mid = 0.15;
        let default_val = default_env.volume_factor(mid);
        let steep_val = steep_env.volume_factor(mid);

        // Higher steepness means even faster initial rise, so mid-segment value differs
        assert!((default_val - steep_val).abs() > 0.01,
            "Different steepness should produce different values: default={}, steep={}", default_val, steep_val);
    }

    #[test]
    fn test_default_curves_are_linear() {
        // Envelope built without specifying curves should behave identically to linear
        let default_env = EnvelopeBuilder::default()
            .attack(EnvelopePair(0.3, 0.9))
            .decay(EnvelopePair(0.35, 0.7))
            .sustain(EnvelopePair(0.6, 0.65))
            .build().unwrap();

        assert_eq!(default_env.attack_curve, EnvelopeCurve::Linear);
        assert_eq!(default_env.decay_curve, EnvelopeCurve::Linear);
        assert_eq!(default_env.sustain_curve, EnvelopeCurve::Linear);
        assert_eq!(default_env.release_curve, EnvelopeCurve::Linear);
    }
}
