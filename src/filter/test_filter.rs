#[cfg(test)]
mod filter_tests {
    use super::super::low_pass_filter::*;

    #[test]
    fn test_basic_filter_creation() {
        let filter = LowPassFilterBuilder::default()
            .cutoff_frequency(1000.0)
            .resonance(0.0)
            .mix(1.0)
            .build_with_coefficients();
        
        assert!(filter.is_ok());
    }

    #[test]
    fn test_filter_application() {
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