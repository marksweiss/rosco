use derive_builder::Builder;

use crate::common::constants::SAMPLE_RATE;

static DEFAULT_MOD_FREQ: f32 = 5.0;
static DEFAULT_MOD_DEPTH: f32 = 0.5;

#[derive(Builder, Debug)]
pub(crate) struct Tremolo {
    #[builder(default = "DEFAULT_MOD_FREQ")]
    pub(crate) mod_freq: f32,

    #[builder(default = "DEFAULT_MOD_DEPTH")]
    pub(crate) mod_depth: f32,

    #[builder(default = "SAMPLE_RATE")]
    pub(crate) sample_rate: f32,

    #[builder(default = "0.0", setter(skip))]
    lfo_phase: f32,
}

impl Clone for Tremolo {
    fn clone(&self) -> Self {
        Tremolo {
            mod_freq: self.mod_freq,
            mod_depth: self.mod_depth,
            sample_rate: self.sample_rate,
            lfo_phase: 0.0,
        }
    }
}

impl PartialEq for Tremolo {
    fn eq(&self, other: &Self) -> bool {
        self.mod_freq == other.mod_freq &&
        self.mod_depth == other.mod_depth &&
        self.sample_rate == other.sample_rate
    }
}

impl Tremolo {
    pub(crate) fn apply_effect(&mut self, sample: f32, _sample_clock: f32) -> f32 {
        let lfo_value = (2.0 * std::f32::consts::PI * self.lfo_phase).sin();
        // gain ranges from (1.0 - mod_depth) to 1.0
        let gain = 1.0 - self.mod_depth * (1.0 - lfo_value) / 2.0;

        // Advance LFO phase
        self.lfo_phase += self.mod_freq / self.sample_rate;
        if self.lfo_phase >= 1.0 {
            self.lfo_phase -= 1.0;
        }

        sample * gain
    }
}

#[allow(dead_code)]
pub(crate) fn default_tremolo() -> Tremolo {
    TremoloBuilder::default()
        .build().unwrap()
}

#[allow(dead_code)]
pub(crate) fn no_op_tremolo() -> Tremolo {
    TremoloBuilder::default()
        .mod_depth(0.0)
        .build().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_construction() {
        let tremolo = default_tremolo();
        assert_eq!(tremolo.mod_freq, DEFAULT_MOD_FREQ);
        assert_eq!(tremolo.mod_depth, DEFAULT_MOD_DEPTH);
        assert_eq!(tremolo.sample_rate, SAMPLE_RATE);
    }

    #[test]
    fn test_custom_construction() {
        let tremolo = TremoloBuilder::default()
            .mod_freq(8.0)
            .mod_depth(0.8)
            .build().unwrap();
        assert_eq!(tremolo.mod_freq, 8.0);
        assert_eq!(tremolo.mod_depth, 0.8);
    }

    #[test]
    fn test_no_op_identity() {
        let mut tremolo = no_op_tremolo();
        let input = 0.5;
        for _ in 0..100 {
            let output = tremolo.apply_effect(input, 0.0);
            assert!((output - input).abs() < 1e-6,
                "no-op tremolo should pass signal through unchanged");
        }
    }

    #[test]
    fn test_effect_modifies_signal() {
        let mut tremolo = default_tremolo();
        let input = 0.5;
        let mut any_different = false;
        for i in 0..1000 {
            let output = tremolo.apply_effect(input, i as f32);
            if (output - input).abs() > 1e-6 {
                any_different = true;
                break;
            }
        }
        assert!(any_different, "tremolo should modify the signal");
    }

    #[test]
    fn test_clone_resets_state() {
        let mut tremolo = default_tremolo();
        // Advance LFO phase
        for i in 0..1000 {
            tremolo.apply_effect(0.5, i as f32);
        }
        let cloned = tremolo.clone();
        assert_eq!(cloned.lfo_phase, 0.0);
    }

    #[test]
    fn test_partial_eq() {
        let a = default_tremolo();
        let b = default_tremolo();
        assert_eq!(a, b);

        let c = TremoloBuilder::default()
            .mod_freq(10.0)
            .build().unwrap();
        assert_ne!(a, c);
    }
}
