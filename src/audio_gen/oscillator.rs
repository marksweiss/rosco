use rand::thread_rng;
use rand_distr::{Distribution, Normal};
use std::sync::Arc;

use crate::common::constants::SAMPLE_RATE;

static TWO_PI: f32 = 2.0 * std::f32::consts::PI;
static NUM_TABLE_SAMPLES: usize = 1024;
static SAMPLE_COUNT_FACTOR: f32 = SAMPLE_RATE / NUM_TABLE_SAMPLES as f32;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Hash, PartialEq)]
pub(crate) enum Waveform {
    GaussianNoise,
    Saw,
    Sine,
    Square,
    Triangle,
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

pub(crate) fn generate_saw_table() -> Vec<f32> {
    let mut table = Vec::with_capacity(NUM_TABLE_SAMPLES);
    for i in 0..NUM_TABLE_SAMPLES {
        let sample = 2.0 * (i as f32 / NUM_TABLE_SAMPLES as f32) - 1.0;
        table.push(sample);
    }
    table
}

pub(crate) fn generate_square_table() -> Vec<f32> {
    let mut table = Vec::with_capacity(NUM_TABLE_SAMPLES);
    for i in 0..NUM_TABLE_SAMPLES {
        let sample = if i < NUM_TABLE_SAMPLES / 2 {
            1.0
        } else {
            -1.0
        };
        table.push(sample);
    }
    table
}

pub(crate) fn generate_triangle_table() -> Vec<f32> {
    let mut table = Vec::with_capacity(NUM_TABLE_SAMPLES);
    for i in 0..NUM_TABLE_SAMPLES {
        let sample = 4.0 * ((i as f32 / NUM_TABLE_SAMPLES as f32)
            - ((i as f32 / NUM_TABLE_SAMPLES as f32) + 0.5)
            .floor()).abs()
            - 1.0;
        table.push(sample);
    }
    table
}

pub(crate) fn get_sample(table: &Vec<f32>, frequency: f32, sample_count: u64) -> f32 {
    table[((frequency * sample_count as f32) / SAMPLE_COUNT_FACTOR) as usize % NUM_TABLE_SAMPLES]
}

pub(crate) fn get_gaussian_noise_sample() -> f32 {
    let normal = Normal::new(0.0, 1.0).unwrap();
    let mut rng = thread_rng();
    normal.sample(&mut rng)
}

// TODO DEPRECATE THESE?
#[allow(dead_code)]
pub(crate) fn get_triangle_sample(frequency: f32, sample_position: f32) -> f32 {
    4.0 * ((frequency * sample_position)
        - ((frequency * sample_position) + 0.5)
        .floor()).abs()
        - 1.0
}

#[allow(dead_code)]
pub(crate) fn get_square_sample(frequency: f32, sample_position: f32) -> f32 {
    if (sample_position * frequency) % 1.0 < 0.5 {
        1.0
    } else {
        -1.0
    }
}

#[allow(dead_code)]
pub(crate) fn get_saw_sample(frequency: f32, sample_position: f32) -> f32 {
    2.0 * ((frequency * sample_position)
        - ((frequency * sample_position) + 0.5)
        .floor()).abs()
        - 1.0
}
// Test comment
