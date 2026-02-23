use std::collections::VecDeque;
use derive_builder::Builder;

use crate::common::constants::SAMPLE_RATE;

static DEFAULT_AVG_DELAY_S: f32 = 0.007;
static DEFAULT_MOD_WIDTH_S: f32 = 0.003;
static DEFAULT_MOD_FREQ: f32 = 5.0;

#[derive(Builder, Debug)]
pub(crate) struct Vibrato {
    #[builder(default = "SAMPLE_RATE")]
    pub(crate) sample_rate: f32,

    // Average delay in seconds
    #[builder(default = "DEFAULT_AVG_DELAY_S")]
    pub(crate) avg_delay: f32,

    // Modulation width in seconds
    #[builder(default = "DEFAULT_MOD_WIDTH_S")]
    pub(crate) mod_width: f32,

    // LFO frequency in Hz
    #[builder(default = "DEFAULT_MOD_FREQ")]
    pub(crate) mod_freq: f32,

    // Circular delay buffer — sized to hold avg_delay + mod_width samples + margin
    #[builder(field(private),
              default = "VecDeque::from(vec![0.0; ((DEFAULT_AVG_DELAY_S + DEFAULT_MOD_WIDTH_S) * SAMPLE_RATE) as usize + 4])")]
    buffer: VecDeque<f32>,

    #[builder(default = "0", setter(skip))]
    write_pos: usize,

    #[builder(default = "0.0", setter(skip))]
    lfo_phase: f32,
}

impl Clone for Vibrato {
    fn clone(&self) -> Self {
        let buf_size = ((self.avg_delay + self.mod_width) * self.sample_rate) as usize + 4;
        Vibrato {
            sample_rate: self.sample_rate,
            avg_delay: self.avg_delay,
            mod_width: self.mod_width,
            mod_freq: self.mod_freq,
            buffer: VecDeque::from(vec![0.0; buf_size]),
            write_pos: 0,
            lfo_phase: 0.0,
        }
    }
}

impl PartialEq for Vibrato {
    fn eq(&self, other: &Self) -> bool {
        self.sample_rate == other.sample_rate &&
        self.avg_delay == other.avg_delay &&
        self.mod_width == other.mod_width &&
        self.mod_freq == other.mod_freq
    }
}

impl Vibrato {
    pub(crate) fn apply_effect(&mut self, sample: f32, _sample_clock: f32) -> f32 {
        let buf_len = self.buffer.len();

        // Compute modulated delay in samples
        let lfo_value = (2.0 * std::f32::consts::PI * self.lfo_phase).sin();
        let delay_samples = (self.avg_delay + self.mod_width * lfo_value) * self.sample_rate;
        let delay_samples = delay_samples.max(0.5).min((buf_len - 1) as f32);

        // Fractional delay with linear interpolation
        let delay_int = delay_samples as usize;
        let frac = delay_samples - delay_int as f32;

        let read_pos_0 = (self.write_pos + buf_len - delay_int) % buf_len;
        let read_pos_1 = (self.write_pos + buf_len - delay_int - 1) % buf_len;

        let delayed_0 = self.buffer[read_pos_0];
        let delayed_1 = self.buffer[read_pos_1];
        let output = delayed_0 + frac * (delayed_1 - delayed_0);

        // Write input into buffer
        self.buffer[self.write_pos] = sample;
        self.write_pos = (self.write_pos + 1) % buf_len;

        // Advance LFO phase
        self.lfo_phase += self.mod_freq / self.sample_rate;
        if self.lfo_phase >= 1.0 {
            self.lfo_phase -= 1.0;
        }

        output
    }
}

#[allow(dead_code)]
pub(crate) fn default_vibrato() -> Vibrato {
    VibratoBuilder::default()
        .build().unwrap()
}

#[allow(dead_code)]
pub(crate) fn no_op_vibrato() -> Vibrato {
    VibratoBuilder::default()
        .mod_width(0.0)
        .build().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_construction() {
        let vibrato = default_vibrato();
        assert_eq!(vibrato.avg_delay, DEFAULT_AVG_DELAY_S);
        assert_eq!(vibrato.mod_width, DEFAULT_MOD_WIDTH_S);
        assert_eq!(vibrato.mod_freq, DEFAULT_MOD_FREQ);
        assert_eq!(vibrato.sample_rate, SAMPLE_RATE);
    }

    #[test]
    fn test_custom_construction() {
        let vibrato = VibratoBuilder::default()
            .avg_delay(0.01)
            .mod_width(0.005)
            .mod_freq(8.0)
            .build().unwrap();
        assert_eq!(vibrato.avg_delay, 0.01);
        assert_eq!(vibrato.mod_width, 0.005);
        assert_eq!(vibrato.mod_freq, 8.0);
    }

    #[test]
    fn test_effect_modifies_signal() {
        let mut vibrato = default_vibrato();
        // Feed a steady signal and collect outputs
        let input = 0.8;
        let mut any_nonzero = false;
        for i in 0..2000 {
            let output = vibrato.apply_effect(input, i as f32);
            // After the buffer fills, outputs should appear
            if output.abs() > 1e-6 {
                any_nonzero = true;
            }
        }
        assert!(any_nonzero, "vibrato should produce output after buffer fills");
    }

    #[test]
    fn test_clone_resets_state() {
        let mut vibrato = default_vibrato();
        for i in 0..1000 {
            vibrato.apply_effect(0.5, i as f32);
        }
        let cloned = vibrato.clone();
        assert_eq!(cloned.lfo_phase, 0.0);
        assert_eq!(cloned.write_pos, 0);
    }

    #[test]
    fn test_partial_eq() {
        let a = default_vibrato();
        let b = default_vibrato();
        assert_eq!(a, b);

        let c = VibratoBuilder::default()
            .mod_freq(10.0)
            .build().unwrap();
        assert_ne!(a, c);
    }
}
