use std::collections::VecDeque;
use derive_builder::Builder;

use crate::common::constants::SAMPLE_RATE;

// ~22ms max delay at 44.1kHz — typical flanging range
static MAX_BUFFER_SIZE: usize = 1024;
static DEFAULT_DELAY_MS: f32 = 5.0;
static DEFAULT_DEPTH_MS: f32 = 4.0;
static DEFAULT_RATE_HZ: f32 = 0.25;
static DEFAULT_MIX: f32 = 0.5;
static DEFAULT_FEEDBACK: f32 = 0.3;

#[derive(Builder, Debug)]
pub(crate) struct Flanger {
    // Center delay time in ms (typically 1-10ms)
    #[builder(default = "DEFAULT_DELAY_MS")]
    pub(crate) delay_ms: f32,

    // Depth of LFO modulation in ms (how far the delay sweeps around center)
    #[builder(default = "DEFAULT_DEPTH_MS")]
    pub(crate) depth_ms: f32,

    // LFO rate in Hz (how fast the sweep oscillates)
    #[builder(default = "DEFAULT_RATE_HZ")]
    pub(crate) rate_hz: f32,

    // The mix level of the effect (0.0 = dry, 1.0 = fully wet)
    #[builder(default = "DEFAULT_MIX")]
    pub(crate) mix: f32,

    // Feedback amount (0.0 to <1.0, higher = more resonant)
    #[builder(default = "DEFAULT_FEEDBACK")]
    pub(crate) feedback: f32,

    // Complement of mix, computed at build time
    #[builder(field(private), default = "1.0 - self.mix.unwrap_or(DEFAULT_MIX)")]
    mix_complement: f32,

    // Circular delay buffer
    #[builder(field(private), default = "VecDeque::from(vec![0.0; MAX_BUFFER_SIZE])")]
    buffer: VecDeque<f32>,

    // Write position in the circular buffer
    #[builder(default = "0", setter(skip))]
    write_pos: usize,

    // LFO phase (0.0 to 1.0)
    #[builder(default = "0.0", setter(skip))]
    lfo_phase: f32,
}

impl Clone for Flanger {
    fn clone(&self) -> Self {
        Flanger {
            delay_ms: self.delay_ms,
            depth_ms: self.depth_ms,
            rate_hz: self.rate_hz,
            mix: self.mix,
            feedback: self.feedback,
            mix_complement: self.mix_complement,
            buffer: VecDeque::from(vec![0.0; MAX_BUFFER_SIZE]),
            write_pos: 0,
            lfo_phase: 0.0,
        }
    }
}

impl PartialEq for Flanger {
    fn eq(&self, other: &Self) -> bool {
        self.delay_ms == other.delay_ms &&
        self.depth_ms == other.depth_ms &&
        self.rate_hz == other.rate_hz &&
        self.mix == other.mix &&
        self.feedback == other.feedback
    }
}

impl Flanger {
    pub(crate) fn apply_effect(&mut self, sample: f32, _sample_clock: f32) -> f32 {
        // Compute current delay time from LFO
        let lfo_value = (2.0 * std::f32::consts::PI * self.lfo_phase).sin();
        let delay_samples = (self.delay_ms + self.depth_ms * lfo_value) * SAMPLE_RATE / 1000.0;
        let delay_samples = delay_samples.max(0.5).min((MAX_BUFFER_SIZE - 1) as f32);

        // Fractional delay with linear interpolation
        let delay_int = delay_samples as usize;
        let frac = delay_samples - delay_int as f32;

        let buf_len = self.buffer.len();
        let read_pos_0 = (self.write_pos + buf_len - delay_int) % buf_len;
        let read_pos_1 = (self.write_pos + buf_len - delay_int - 1) % buf_len;

        let delayed_0 = self.buffer[read_pos_0];
        let delayed_1 = self.buffer[read_pos_1];
        let delayed = delayed_0 + frac * (delayed_1 - delayed_0);

        // Write input + feedback into buffer
        let write_sample = sample + delayed * self.feedback;
        self.buffer[self.write_pos] = write_sample;

        // Advance write position
        self.write_pos = (self.write_pos + 1) % buf_len;

        // Advance LFO phase
        self.lfo_phase += self.rate_hz / SAMPLE_RATE;
        if self.lfo_phase >= 1.0 {
            self.lfo_phase -= 1.0;
        }

        // Mix original and flanged signals
        sample * self.mix_complement + delayed * self.mix
    }
}

#[allow(dead_code)]
pub(crate) fn default_flanger() -> Flanger {
    FlangerBuilder::default()
        .build().unwrap()
}

#[allow(dead_code)]
pub(crate) fn no_op_flanger() -> Flanger {
    FlangerBuilder::default()
        .mix(0.0)
        .build().unwrap()
}
