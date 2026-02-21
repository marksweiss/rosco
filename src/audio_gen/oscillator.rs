use rand::thread_rng;
use rand_distr::{Distribution, Normal};
use std::sync::Arc;

use crate::common::constants::SAMPLE_RATE;

static TWO_PI: f32 = 2.0 * std::f32::consts::PI;
static NUM_TABLE_SAMPLES: usize = 1024;
static SAMPLE_COUNT_FACTOR: f32 = SAMPLE_RATE / NUM_TABLE_SAMPLES as f32;

#[derive(Clone, Copy, Debug, Hash, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Waveform {
    GaussianNoise,
    Saw,
    Sine,
    Square,
    Triangle,
    Noise, // Add alias for consistency with TUI
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct OscillatorTables {
    pub(crate) sine_table: Arc<Vec<f32>>,
    pub(crate) saw_table: Arc<Vec<f32>>,
    pub(crate) square_table: Arc<Vec<f32>>,
    pub(crate) triangle_table: Arc<Vec<f32>>,
}

impl OscillatorTables {
    pub(crate) fn new() -> OscillatorTables {
        OscillatorTables {
            sine_table: Arc::new(generate_sine_table()),
            saw_table: Arc::new(generate_saw_table()),
            square_table: Arc::new(generate_square_table()),
            triangle_table: Arc::new(generate_triangle_table()),
        }
    }
}

pub(crate) fn generate_sine_table() -> Vec<f32> {
    let mut table = Vec::with_capacity(NUM_TABLE_SAMPLES);
    for i in 0..NUM_TABLE_SAMPLES {
        let sample = (TWO_PI * i as f32 / NUM_TABLE_SAMPLES as f32).sin();
        table.push(sample);
    }
    table
}

// Band-limited sawtooth: sum of sin(k*x)/k for k=1..max_harmonic
// This prevents aliasing by only including harmonics below Nyquist
pub(crate) fn generate_saw_table() -> Vec<f32> {
    let max_harmonic = (SAMPLE_RATE / 2.0 / 20.0) as usize; // harmonics up to Nyquist for ~20Hz fundamental
    let mut table = vec![0.0f32; NUM_TABLE_SAMPLES];
    for k in 1..=max_harmonic {
        let sign = if k % 2 == 0 { 1.0 } else { -1.0 };
        for i in 0..NUM_TABLE_SAMPLES {
            let phase = TWO_PI * i as f32 / NUM_TABLE_SAMPLES as f32;
            table[i] += sign * (k as f32 * phase).sin() / k as f32;
        }
    }
    // Normalize to [-1, 1]
    let max_val = table.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    if max_val > 0.0 {
        for sample in table.iter_mut() {
            *sample /= max_val;
        }
    }
    table
}

// Band-limited square: sum of sin((2k-1)*x)/(2k-1) for odd harmonics
pub(crate) fn generate_square_table() -> Vec<f32> {
    let max_harmonic = (SAMPLE_RATE / 2.0 / 20.0) as usize;
    let mut table = vec![0.0f32; NUM_TABLE_SAMPLES];
    let mut k = 1;
    while k <= max_harmonic {
        for i in 0..NUM_TABLE_SAMPLES {
            let phase = TWO_PI * i as f32 / NUM_TABLE_SAMPLES as f32;
            table[i] += (k as f32 * phase).sin() / k as f32;
        }
        k += 2; // odd harmonics only
    }
    // Normalize to [-1, 1]
    let max_val = table.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    if max_val > 0.0 {
        for sample in table.iter_mut() {
            *sample /= max_val;
        }
    }
    table
}

// Band-limited triangle: sum of (-1)^k * sin((2k-1)*x)/(2k-1)^2 for odd harmonics
pub(crate) fn generate_triangle_table() -> Vec<f32> {
    let max_harmonic = (SAMPLE_RATE / 2.0 / 20.0) as usize;
    let mut table = vec![0.0f32; NUM_TABLE_SAMPLES];
    let mut k = 1;
    let mut sign = 1.0f32;
    while k <= max_harmonic {
        let k_f = k as f32;
        for i in 0..NUM_TABLE_SAMPLES {
            let phase = TWO_PI * i as f32 / NUM_TABLE_SAMPLES as f32;
            table[i] += sign * (k_f * phase).sin() / (k_f * k_f);
        }
        sign = -sign;
        k += 2; // odd harmonics only
    }
    // Normalize to [-1, 1]
    let max_val = table.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    if max_val > 0.0 {
        for sample in table.iter_mut() {
            *sample /= max_val;
        }
    }
    table
}

pub(crate) fn get_sample(table: &[f32], frequency: f32, sample_count: u64) -> f32 {
    let phase = (frequency * sample_count as f32) / SAMPLE_COUNT_FACTOR;
    let index = phase as usize % NUM_TABLE_SAMPLES;
    let frac = phase.fract();
    let next_index = (index + 1) % NUM_TABLE_SAMPLES;
    // Linear interpolation between adjacent table entries
    table[index] + frac * (table[next_index] - table[index])
}

pub(crate) fn get_gaussian_noise_sample() -> f32 {
    let normal = Normal::new(0.0, 1.0).unwrap();
    let mut rng = thread_rng();
    normal.sample(&mut rng)
}

