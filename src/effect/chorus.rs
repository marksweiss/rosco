use std::collections::VecDeque;
use derive_builder::Builder;

use crate::common::constants::SAMPLE_RATE;

static DEFAULT_CHORUS_COUNT: usize = 3;
static DEFAULT_DRY_GAIN: f32 = 0.7;

// Default per-voice parameters
static DEFAULT_CHORUS_GAINS: [f32; 3] = [0.4, 0.4, 0.4];
static DEFAULT_CHORUS_DELAYS_S: [f32; 3] = [0.015, 0.020, 0.030];
static DEFAULT_MOD_FREQS: [f32; 3] = [0.25, 0.33, 0.40];
static DEFAULT_MOD_WIDTHS_S: [f32; 3] = [0.003, 0.004, 0.005];

#[derive(Builder, Debug)]
#[builder(build_fn(skip))]
pub(crate) struct Chorus {
    #[builder(default = "SAMPLE_RATE")]
    pub(crate) sample_rate: f32,

    #[builder(default = "DEFAULT_CHORUS_COUNT")]
    pub(crate) chorus_count: usize,

    #[builder(default = "DEFAULT_CHORUS_GAINS.to_vec()")]
    pub(crate) chorus_gains: Vec<f32>,

    #[builder(default = "DEFAULT_DRY_GAIN")]
    pub(crate) dry_gain: f32,

    // Base delay times per voice in seconds
    #[builder(default = "DEFAULT_CHORUS_DELAYS_S.to_vec()")]
    pub(crate) chorus_delays: Vec<f32>,

    // LFO frequencies per voice in Hz
    #[builder(default = "DEFAULT_MOD_FREQS.to_vec()")]
    pub(crate) mod_freqs: Vec<f32>,

    // LFO modulation widths per voice in seconds
    #[builder(default = "DEFAULT_MOD_WIDTHS_S.to_vec()")]
    pub(crate) mod_widths: Vec<f32>,

    // Per-voice LFO phases (private, runtime state)
    #[builder(field(private))]
    lfo_phases: Vec<f32>,

    // Shared circular delay buffer
    #[builder(field(private))]
    buffer: VecDeque<f32>,

    // Write position in the circular buffer
    #[builder(field(private))]
    write_pos: usize,
}

impl Clone for Chorus {
    fn clone(&self) -> Self {
        let max_delay_samples = self.chorus_delays.iter()
            .zip(self.mod_widths.iter())
            .map(|(&d, &w)| ((d + w) * self.sample_rate) as usize)
            .max()
            .unwrap_or(0) + 4;

        Chorus {
            sample_rate: self.sample_rate,
            chorus_count: self.chorus_count,
            chorus_gains: self.chorus_gains.clone(),
            dry_gain: self.dry_gain,
            chorus_delays: self.chorus_delays.clone(),
            mod_freqs: self.mod_freqs.clone(),
            mod_widths: self.mod_widths.clone(),
            lfo_phases: vec![0.0; self.chorus_count],
            buffer: VecDeque::from(vec![0.0; max_delay_samples]),
            write_pos: 0,
        }
    }
}

impl PartialEq for Chorus {
    fn eq(&self, other: &Self) -> bool {
        self.sample_rate == other.sample_rate &&
        self.chorus_count == other.chorus_count &&
        self.chorus_gains == other.chorus_gains &&
        self.dry_gain == other.dry_gain &&
        self.chorus_delays == other.chorus_delays &&
        self.mod_freqs == other.mod_freqs &&
        self.mod_widths == other.mod_widths
    }
}

#[allow(dead_code)]
impl ChorusBuilder {
    pub(crate) fn build(&mut self) -> Result<Chorus, String> {
        let sample_rate = self.sample_rate.unwrap_or(SAMPLE_RATE);
        let chorus_count = self.chorus_count.unwrap_or(DEFAULT_CHORUS_COUNT);
        let chorus_gains = self.chorus_gains.clone()
            .unwrap_or_else(|| DEFAULT_CHORUS_GAINS.to_vec());
        let dry_gain = self.dry_gain.unwrap_or(DEFAULT_DRY_GAIN);
        let chorus_delays = self.chorus_delays.clone()
            .unwrap_or_else(|| DEFAULT_CHORUS_DELAYS_S.to_vec());
        let mod_freqs = self.mod_freqs.clone()
            .unwrap_or_else(|| DEFAULT_MOD_FREQS.to_vec());
        let mod_widths = self.mod_widths.clone()
            .unwrap_or_else(|| DEFAULT_MOD_WIDTHS_S.to_vec());

        // Validate vector lengths match chorus_count
        if chorus_gains.len() != chorus_count {
            return Err(format!("chorus_gains length {} does not match chorus_count {}",
                chorus_gains.len(), chorus_count));
        }
        if chorus_delays.len() != chorus_count {
            return Err(format!("chorus_delays length {} does not match chorus_count {}",
                chorus_delays.len(), chorus_count));
        }
        if mod_freqs.len() != chorus_count {
            return Err(format!("mod_freqs length {} does not match chorus_count {}",
                mod_freqs.len(), chorus_count));
        }
        if mod_widths.len() != chorus_count {
            return Err(format!("mod_widths length {} does not match chorus_count {}",
                mod_widths.len(), chorus_count));
        }

        // Compute buffer size: max possible delay across all voices + margin
        let max_delay_samples = chorus_delays.iter()
            .zip(mod_widths.iter())
            .map(|(&d, &w)| ((d + w) * sample_rate) as usize)
            .max()
            .unwrap_or(0) + 4;

        Ok(Chorus {
            sample_rate,
            chorus_count,
            chorus_gains,
            dry_gain,
            chorus_delays,
            mod_freqs,
            mod_widths,
            lfo_phases: vec![0.0; chorus_count],
            buffer: VecDeque::from(vec![0.0; max_delay_samples]),
            write_pos: 0,
        })
    }
}

impl Chorus {
    pub(crate) fn apply_effect(&mut self, sample: f32, _sample_clock: f32) -> f32 {
        let buf_len = self.buffer.len();
        let mut output = self.dry_gain * sample;

        for voice in 0..self.chorus_count {
            // Compute modulated delay for this voice
            let lfo_value = (2.0 * std::f32::consts::PI * self.lfo_phases[voice]).sin();
            let delay_samples = (self.chorus_delays[voice] +
                self.mod_widths[voice] * lfo_value) * self.sample_rate;
            let delay_samples = delay_samples.max(0.5).min((buf_len - 1) as f32);

            // Linear interpolation
            let delay_int = delay_samples as usize;
            let frac = delay_samples - delay_int as f32;

            let read_pos_0 = (self.write_pos + buf_len - delay_int) % buf_len;
            let read_pos_1 = (self.write_pos + buf_len - delay_int - 1) % buf_len;

            let delayed_0 = self.buffer[read_pos_0];
            let delayed_1 = self.buffer[read_pos_1];
            let delayed = delayed_0 + frac * (delayed_1 - delayed_0);

            output += self.chorus_gains[voice] * delayed;

            // Advance this voice's LFO phase
            self.lfo_phases[voice] += self.mod_freqs[voice] / self.sample_rate;
            if self.lfo_phases[voice] >= 1.0 {
                self.lfo_phases[voice] -= 1.0;
            }
        }

        // Write input into buffer
        self.buffer[self.write_pos] = sample;
        self.write_pos = (self.write_pos + 1) % buf_len;

        output
    }
}

#[allow(dead_code)]
pub(crate) fn default_chorus() -> Chorus {
    ChorusBuilder::default()
        .build().unwrap()
}

#[allow(dead_code)]
pub(crate) fn no_op_chorus() -> Chorus {
    ChorusBuilder::default()
        .chorus_gains(vec![0.0, 0.0, 0.0])
        .dry_gain(1.0)
        .build().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_construction() {
        let chorus = default_chorus();
        assert_eq!(chorus.chorus_count, DEFAULT_CHORUS_COUNT);
        assert_eq!(chorus.dry_gain, DEFAULT_DRY_GAIN);
        assert_eq!(chorus.chorus_gains.len(), DEFAULT_CHORUS_COUNT);
        assert_eq!(chorus.chorus_delays.len(), DEFAULT_CHORUS_COUNT);
        assert_eq!(chorus.mod_freqs.len(), DEFAULT_CHORUS_COUNT);
        assert_eq!(chorus.mod_widths.len(), DEFAULT_CHORUS_COUNT);
    }

    #[test]
    fn test_custom_construction() {
        let chorus = ChorusBuilder::default()
            .chorus_count(2)
            .chorus_gains(vec![0.5, 0.5])
            .chorus_delays(vec![0.01, 0.02])
            .mod_freqs(vec![0.5, 0.6])
            .mod_widths(vec![0.002, 0.003])
            .dry_gain(0.8)
            .build().unwrap();
        assert_eq!(chorus.chorus_count, 2);
        assert_eq!(chorus.dry_gain, 0.8);
    }

    #[test]
    fn test_mismatched_vector_lengths() {
        let result = ChorusBuilder::default()
            .chorus_count(2)
            .chorus_gains(vec![0.5]) // wrong length
            .chorus_delays(vec![0.01, 0.02])
            .mod_freqs(vec![0.5, 0.6])
            .mod_widths(vec![0.002, 0.003])
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_no_op_passes_signal() {
        let mut chorus = no_op_chorus();
        let input = 0.5;
        // Feed enough samples to fill the buffer, then check output
        for _ in 0..2000 {
            chorus.apply_effect(input, 0.0);
        }
        let output = chorus.apply_effect(input, 0.0);
        // With all chorus_gains at 0.0 and dry_gain at 1.0, output should equal input
        assert!((output - input).abs() < 1e-4,
            "no-op chorus should pass signal through, got {}", output);
    }

    #[test]
    fn test_effect_modifies_signal() {
        let mut chorus = default_chorus();
        let input = 0.5;
        // Fill the buffer first
        for i in 0..2000 {
            chorus.apply_effect(input, i as f32);
        }
        let output = chorus.apply_effect(input, 2000.0);
        // dry_gain is 0.7, so even with no chorus, output != input
        // With chorus voices active, output should differ from input
        assert!((output - input).abs() > 1e-6,
            "chorus should modify the signal");
    }

    #[test]
    fn test_clone_resets_state() {
        let mut chorus = default_chorus();
        for i in 0..1000 {
            chorus.apply_effect(0.5, i as f32);
        }
        let cloned = chorus.clone();
        assert_eq!(cloned.lfo_phases, vec![0.0; DEFAULT_CHORUS_COUNT]);
        assert_eq!(cloned.write_pos, 0);
    }

    #[test]
    fn test_partial_eq() {
        let a = default_chorus();
        let b = default_chorus();
        assert_eq!(a, b);

        let c = ChorusBuilder::default()
            .dry_gain(0.9)
            .build().unwrap();
        assert_ne!(a, c);
    }
}
