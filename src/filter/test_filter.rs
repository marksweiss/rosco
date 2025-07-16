#[cfg(test)]
mod filter_tests {
    use super::super::low_pass_filter::*;
    use super::super::high_pass_filter::*;
    use super::super::band_pass_filter::*;
    use super::super::notch_filter::*;

    #[test]
    fn test_basic_low_pass_filter_creation() {
        let filter = LowPassFilterBuilder::default()
            .cutoff_frequency(1000.0)
            .resonance(0.0)
            .mix(1.0)
            .build_with_coefficients();

        assert!(filter.is_ok());
    }

    #[test]
    fn test_low_pass_filter_application() {
        let mut filter = LowPassFilterBuilder::default()
            .cutoff_frequency(500.0)
            .resonance(0.0)
            .mix(1.0)
            .build_with_coefficients()
            .unwrap();

        let input = 1.0;
        let output = filter.apply_effect(input, 0.0);

        // Output should be different from input due to filtering
        assert_ne!(input, output);
    }

    #[test]
    fn test_basic_high_pass_filter_creation() {
        let filter = HighPassFilterBuilder::default()
            .cutoff_frequency(1000.0)
            .resonance(0.0)
            .mix(1.0)
            .build_with_coefficients();

        assert!(filter.is_ok());
    }

    #[test]
    fn test_high_pass_filter_application() {
        let mut filter = HighPassFilterBuilder::default()
            .cutoff_frequency(2000.0)
            .resonance(0.0)
            .mix(1.0)
            .build_with_coefficients()
            .unwrap();

        let input = 1.0;
        let output = filter.apply_effect(input, 0.0);

        // Output should be different from input due to filtering
        assert_ne!(input, output);
    }

    #[test]
    fn test_basic_band_pass_filter_creation() {
        let filter = BandPassFilterBuilder::default()
            .center_frequency(1000.0)
            .bandwidth(200.0)
            .resonance(0.0)
            .mix(1.0)
            .build_with_coefficients();

        assert!(filter.is_ok());
    }

    #[test]
    fn test_band_pass_filter_application() {
        let mut filter = BandPassFilterBuilder::default()
            .center_frequency(1000.0)
            .bandwidth(200.0)
            .resonance(0.0)
            .mix(1.0)
            .build_with_coefficients()
            .unwrap();

        let input = 1.0;
        let output = filter.apply_effect(input, 0.0);

        // Output should be different from input due to filtering
        assert_ne!(input, output);
    }

    #[test]
    fn test_basic_notch_filter_creation() {
        let filter = NotchFilterBuilder::default()
            .center_frequency(1000.0)
            .bandwidth(100.0)
            .resonance(0.0)
            .mix(1.0)
            .build_with_coefficients();

        assert!(filter.is_ok());
    }

    #[test]
    fn test_notch_filter_application() {
        let mut filter = NotchFilterBuilder::default()
            .center_frequency(1000.0)
            .bandwidth(100.0)
            .resonance(0.0)
            .mix(1.0)
            .build_with_coefficients()
            .unwrap();

        let input = 1.0;
        let output = filter.apply_effect(input, 0.0);

        // Output should be different from input due to filtering
        assert_ne!(input, output);
    }

    #[test]
    fn test_filter_mix_behavior() {
        // Test that mix parameter works correctly for all filter types
        let input_sample = 1.0;

        // Low-pass filter with 50% mix
        let mut lp_filter = LowPassFilterBuilder::default()
            .cutoff_frequency(100.0)
            .mix(0.5)
            .build_with_coefficients()
            .unwrap();
        let lp_output = lp_filter.apply_effect(input_sample, 0.0);
        assert!(lp_output > 0.0 && lp_output < input_sample);

        // High-pass filter with 50% mix
        let mut hp_filter = HighPassFilterBuilder::default()
            .cutoff_frequency(5000.0)
            .mix(0.5)
            .build_with_coefficients()
            .unwrap();
        let hp_output = hp_filter.apply_effect(input_sample, 0.0);
        assert!(hp_output > 0.0 && hp_output < input_sample);

        // Band-pass filter with 50% mix
        let mut bp_filter = BandPassFilterBuilder::default()
            .center_frequency(1000.0)
            .bandwidth(100.0)
            .mix(0.5)
            .build_with_coefficients()
            .unwrap();
        let bp_output = bp_filter.apply_effect(input_sample, 0.0);
        assert!(bp_output > 0.0 && bp_output < input_sample);

        // Notch filter with 50% mix
        let mut notch_filter = NotchFilterBuilder::default()
            .center_frequency(1000.0)
            .bandwidth(100.0)
            .mix(0.5)
            .build_with_coefficients()
            .unwrap();
        let notch_output = notch_filter.apply_effect(input_sample, 0.0);
        assert!(notch_output > 0.0 && notch_output < input_sample);
    }

    #[test]
    fn test_filter_reset_behavior() {
        // Test that reset works correctly for all filter types
        // We can't directly access private history fields, but we can test
        // that reset doesn't panic and that the filter continues to work
        let input_sample = 1.0;

        // Low-pass filter
        let mut lp_filter = default_low_pass_filter();
        let output1 = lp_filter.apply_effect(input_sample, 0.0);
        lp_filter.reset();
        let output2 = lp_filter.apply_effect(input_sample, 0.0);
        // After reset, the first output should be the same as a fresh filter
        assert!((output1 - output2).abs() < 1e-6);

        // High-pass filter
        let mut hp_filter = default_high_pass_filter();
        let output1 = hp_filter.apply_effect(input_sample, 0.0);
        hp_filter.reset();
        let output2 = hp_filter.apply_effect(input_sample, 0.0);
        assert!((output1 - output2).abs() < 1e-6);

        // Band-pass filter
        let mut bp_filter = default_band_pass_filter();
        let output1 = bp_filter.apply_effect(input_sample, 0.0);
        bp_filter.reset();
        let output2 = bp_filter.apply_effect(input_sample, 0.0);
        assert!((output1 - output2).abs() < 1e-6);

        // Notch filter
        let mut notch_filter = default_notch_filter();
        let output1 = notch_filter.apply_effect(input_sample, 0.0);
        notch_filter.reset();
        let output2 = notch_filter.apply_effect(input_sample, 0.0);
        assert!((output1 - output2).abs() < 1e-6);
    }

    #[test]
    fn test_all_filter_types_integration() {
        // Integration test to verify all filter types work together
        let input_sample = 1.0;

        // Create all filter types with different parameters
        let mut lp_filter = LowPassFilterBuilder::default()
            .cutoff_frequency(800.0)
            .resonance(0.2)
            .mix(0.7)
            .build_with_coefficients()
            .unwrap();

        let mut hp_filter = HighPassFilterBuilder::default()
            .cutoff_frequency(200.0)
            .resonance(0.3)
            .mix(0.6)
            .build_with_coefficients()
            .unwrap();

        let mut bp_filter = BandPassFilterBuilder::default()
            .center_frequency(1000.0)
            .bandwidth(300.0)
            .resonance(0.4)
            .mix(0.8)
            .build_with_coefficients()
            .unwrap();

        let mut notch_filter = NotchFilterBuilder::default()
            .center_frequency(1500.0)
            .bandwidth(100.0)
            .resonance(0.5)
            .mix(0.9)
            .build_with_coefficients()
            .unwrap();

        // Apply filters in sequence (like a filter chain)
        let mut sample = input_sample;
        sample = lp_filter.apply_effect(sample, 0.0);
        sample = hp_filter.apply_effect(sample, 0.0);
        sample = bp_filter.apply_effect(sample, 0.0);
        sample = notch_filter.apply_effect(sample, 0.0);

        // The final output should be different from the input
        assert_ne!(sample, input_sample);

        // And should be a valid number
        assert!(sample.is_finite());
        assert!(!sample.is_nan());
    }

    #[test]
    fn test_filter_parameters() {
        let filter = LowPassFilterBuilder::default()
            .cutoff_frequency(2000.0)
            .resonance(0.5)
            .mix(0.8)
            .build_with_coefficients()
            .unwrap();
        
        assert_eq!(filter.cutoff_frequency, 2000.0);
        assert_eq!(filter.resonance, 0.5);
        assert_eq!(filter.mix, 0.8);
    }
} 