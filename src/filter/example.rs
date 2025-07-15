// Example usage of the low-pass filter
// This file demonstrates how to create and use the LowPassFilter

use crate::filter::low_pass_filter::*;

pub fn example_filter_usage() {
    // Create a low-pass filter with custom parameters
    let mut filter = LowPassFilterBuilder::default()
        .cutoff_frequency(1000.0)  // 1kHz cutoff
        .resonance(0.3)            // Moderate resonance
        .mix(0.8)                  // 80% filtered, 20% dry
        .build()
        .unwrap();
    
    // Process some audio samples
    let samples = vec![1.0, 0.5, -0.3, 0.8, -0.1];
    let mut filtered_samples = Vec::new();
    
    for sample in &samples {
        let filtered = filter.apply_effect(*sample, 0.0);
        filtered_samples.push(filtered);
    }
    
    println!("Original samples: {:?}", samples);
    println!("Filtered samples: {:?}", filtered_samples);
}

pub fn example_filter_comparison() {
    // Compare different cutoff frequencies
    let frequencies = vec![500.0, 1000.0, 2000.0];
    let test_sample = 1.0;
    
    for freq in frequencies {
        let mut filter = LowPassFilterBuilder::default()
            .cutoff_frequency(freq)
            .resonance(0.0)
            .mix(1.0)
            .build()
            .unwrap();
        
        let output = filter.apply_effect(test_sample, 0.0);
        println!("Cutoff {}Hz: input={}, output={}", freq, test_sample, output);
    }
} 