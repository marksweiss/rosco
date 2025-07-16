// Example usage of the low-pass filter
// This file demonstrates how to create and use the LowPassFilter

use crate::filter::low_pass_filter::*;
use crate::filter::high_pass_filter::*;
use crate::filter::band_pass_filter::*;
use crate::filter::notch_filter::*;

pub fn example_filter_usage() {
    // Create a low-pass filter with custom parameters
    let mut filter = LowPassFilterBuilder::default()
        .cutoff_frequency(1000.0)  // 1kHz cutoff
        .resonance(0.3)            // Moderate resonance
        .mix(0.8)                  // 80% filtered, 20% dry
        .build_with_coefficients()
        .unwrap();

    // Process some audio samples
    let samples = vec![1.0, 0.5, -0.3, 0.8, -0.1];
    let mut filtered_samples = Vec::new();

    for sample in &samples {
        let filtered = filter.apply_effect(*sample, 0.0);
        filtered_samples.push(filtered);
    }

    println!("Low-pass filter:");
    println!("Original samples: {:?}", samples);
    println!("Filtered samples: {:?}", filtered_samples);
}

pub fn example_high_pass_filter_usage() {
    // Create a high-pass filter with custom parameters
    let mut filter = HighPassFilterBuilder::default()
        .cutoff_frequency(500.0)   // 500Hz cutoff
        .resonance(0.2)            // Light resonance
        .mix(0.9)                  // 90% filtered, 10% dry
        .build_with_coefficients()
        .unwrap();

    // Process some audio samples
    let samples = vec![1.0, 0.5, -0.3, 0.8, -0.1];
    let mut filtered_samples = Vec::new();

    for sample in &samples {
        let filtered = filter.apply_effect(*sample, 0.0);
        filtered_samples.push(filtered);
    }

    println!("High-pass filter:");
    println!("Original samples: {:?}", samples);
    println!("Filtered samples: {:?}", filtered_samples);
}

pub fn example_band_pass_filter_usage() {
    // Create a band-pass filter with custom parameters
    let mut filter = BandPassFilterBuilder::default()
        .center_frequency(1000.0)  // 1kHz center
        .bandwidth(200.0)          // 200Hz bandwidth
        .resonance(0.4)            // Moderate resonance
        .mix(0.8)                  // 80% filtered, 20% dry
        .build_with_coefficients()
        .unwrap();

    // Process some audio samples
    let samples = vec![1.0, 0.5, -0.3, 0.8, -0.1];
    let mut filtered_samples = Vec::new();

    for sample in &samples {
        let filtered = filter.apply_effect(*sample, 0.0);
        filtered_samples.push(filtered);
    }

    println!("Band-pass filter:");
    println!("Original samples: {:?}", samples);
    println!("Filtered samples: {:?}", filtered_samples);
}

pub fn example_notch_filter_usage() {
    // Create a notch filter with custom parameters
    let mut filter = NotchFilterBuilder::default()
        .center_frequency(1000.0)  // 1kHz center
        .bandwidth(100.0)          // 100Hz notch width
        .resonance(0.6)            // High resonance for sharp notch
        .mix(0.8)                  // 80% filtered, 20% dry
        .build_with_coefficients()
        .unwrap();

    // Process some audio samples
    let samples = vec![1.0, 0.5, -0.3, 0.8, -0.1];
    let mut filtered_samples = Vec::new();

    for sample in &samples {
        let filtered = filter.apply_effect(*sample, 0.0);
        filtered_samples.push(filtered);
    }

    println!("Notch filter:");
    println!("Original samples: {:?}", samples);
    println!("Filtered samples: {:?}", filtered_samples);
}

pub fn example_filter_comparison() {
    // Compare different filter types with the same frequency settings
    let test_sample = 1.0;
    let frequency = 1000.0;

    println!("Filter comparison at {}Hz:", frequency);

    // Low-pass filter
    let mut lp_filter = LowPassFilterBuilder::default()
        .cutoff_frequency(frequency)
        .resonance(0.0)
        .mix(1.0)
        .build_with_coefficients()
        .unwrap();
    let lp_output = lp_filter.apply_effect(test_sample, 0.0);
    println!("Low-pass: input={}, output={}", test_sample, lp_output);

    // High-pass filter
    let mut hp_filter = HighPassFilterBuilder::default()
        .cutoff_frequency(frequency)
        .resonance(0.0)
        .mix(1.0)
        .build_with_coefficients()
        .unwrap();
    let hp_output = hp_filter.apply_effect(test_sample, 0.0);
    println!("High-pass: input={}, output={}", test_sample, hp_output);

    // Band-pass filter
    let mut bp_filter = BandPassFilterBuilder::default()
        .center_frequency(frequency)
        .bandwidth(200.0)
        .resonance(0.0)
        .mix(1.0)
        .build_with_coefficients()
        .unwrap();
    let bp_output = bp_filter.apply_effect(test_sample, 0.0);
    println!("Band-pass: input={}, output={}", test_sample, bp_output);

    // Notch filter
    let mut notch_filter = NotchFilterBuilder::default()
        .center_frequency(frequency)
        .bandwidth(100.0)
        .resonance(0.0)
        .mix(1.0)
        .build_with_coefficients()
        .unwrap();
    let notch_output = notch_filter.apply_effect(test_sample, 0.0);
    println!("Notch: input={}, output={}", test_sample, notch_output);
}

pub fn example_all_filters() {
    println!("=== Filter Examples ===");
    example_filter_usage();
    println!();
    example_high_pass_filter_usage();
    println!();
    example_band_pass_filter_usage();
    println!();
    example_notch_filter_usage();
    println!();
    example_filter_comparison();
}