use derive_builder::Builder;
use crate::common::constants::{SAMPLE_RATE, NYQUIST_FREQUENCY};

static DEFAULT_CUTOFF_FREQUENCY: f32 = 1000.0;
static DEFAULT_RESONANCE: f32 = 0.0;
static DEFAULT_MIX: f32 = 1.0;

/// Low-pass filter that attenuates frequencies above the cutoff frequency
/// 
/// This filter uses a second-order IIR (Infinite Impulse Response) filter
/// with a Butterworth response. The filter allows frequencies below the
/// cutoff frequency to pass through while attenuating frequencies above it.
#[derive(Builder, Debug)]
pub(crate) struct LowPassFilter {
    /// The cutoff frequency in Hz where the filter begins to affect the signal
    #[builder(default = "DEFAULT_CUTOFF_FREQUENCY")]
    pub(crate) cutoff_frequency: f32,

    /// Resonance/Q factor that controls the sharpness of the filter response
    /// Higher values create a more pronounced peak near the cutoff frequency
    #[builder(default = "DEFAULT_RESONANCE")]
    pub(crate) resonance: f32,

    /// Mix level of the filtered signal (0.0 = dry, 1.0 = fully filtered)
    #[builder(default = "DEFAULT_MIX")]
    pub(crate) mix: f32,

    /// Complement of mix, computed at build time
    #[builder(field(private), default = "1.0 - self.mix.unwrap_or(DEFAULT_MIX)")]
    mix_complement: f32,

    /// Filter coefficients for the IIR filter
    #[builder(field(private), default = "FilterCoefficients { b0: 1.0, b1: 0.0, b2: 0.0, a1: 0.0, a2: 0.0 }")]
    coefficients: FilterCoefficients,

    /// Previous input samples for the filter
    #[builder(field(private), default = "[0.0; 2]")]
    x_history: [f32; 2],

    /// Previous output samples for the filter
    #[builder(field(private), default = "[0.0; 2]")]
    y_history: [f32; 2],
}

/// Filter coefficients for the second-order IIR filter
#[derive(Debug, Clone)]
struct FilterCoefficients {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
}

impl Clone for LowPassFilter {
    fn clone(&self) -> Self {
        LowPassFilter {
            cutoff_frequency: self.cutoff_frequency,
            resonance: self.resonance,
            mix: self.mix,
            mix_complement: self.mix_complement,
            coefficients: self.coefficients.clone(),
            x_history: self.x_history,
            y_history: self.y_history,
        }
    }
}

impl PartialEq for LowPassFilter {
    fn eq(&self, other: &Self) -> bool {
        self.cutoff_frequency == other.cutoff_frequency &&
        self.resonance == other.resonance &&
        self.mix == other.mix &&
        self.mix_complement == other.mix_complement &&
        self.x_history == other.x_history &&
        self.y_history == other.y_history
    }
}

impl LowPassFilter {
    /// Apply the low-pass filter to a single sample
    /// 
    /// # Arguments
    /// * `sample` - The input sample to filter
    /// * `_sample_clock` - The current sample clock (unused but kept for consistency with other effects)
    /// 
    /// # Returns
    /// The filtered sample
    pub(crate) fn apply_effect(&mut self, sample: f32, _sample_clock: f32) -> f32 {
        // Apply the IIR filter
        let filtered_sample = self.apply_iir_filter(sample);
        
        // Mix the original and filtered signals
        sample * self.mix_complement + filtered_sample * self.mix
    }

    /// Apply the IIR filter using the current coefficients
    fn apply_iir_filter(&mut self, sample: f32) -> f32 {
        // Direct Form II implementation
        let w = sample - self.coefficients.a1 * self.x_history[0] - self.coefficients.a2 * self.x_history[1];
        let output = self.coefficients.b0 * w + self.coefficients.b1 * self.x_history[0] + self.coefficients.b2 * self.x_history[1];
        
        // Update history
        self.x_history[1] = self.x_history[0];
        self.x_history[0] = w;
        self.y_history[1] = self.y_history[0];
        self.y_history[0] = output;
        
        output
    }

    /// Update the filter coefficients based on current cutoff frequency and resonance
    pub(crate) fn update_coefficients(&mut self) {
        self.coefficients = self.calculate_coefficients();
    }

    /// Calculate the filter coefficients for the current parameters
    fn calculate_coefficients(&self) -> FilterCoefficients {
        // Clamp cutoff frequency to valid range
        let cutoff = self.cutoff_frequency.max(20.0).min(NYQUIST_FREQUENCY * 0.99);
        
        // Convert frequency to normalized frequency (0 to 1)
        let omega = 2.0 * std::f32::consts::PI * cutoff / SAMPLE_RATE;
        
        // Calculate Q factor from resonance (resonance is typically 0.0 to 1.0)
        let q = if self.resonance > 0.0 {
            1.0 / (2.0 * self.resonance)
        } else {
            0.707 // Default Q for Butterworth response
        };
        
        // Calculate filter coefficients for a second-order low-pass filter
        let alpha = omega.sin() / (2.0 * q);
        let cos_w = omega.cos();
        
        let b0 = (1.0 - cos_w) / 2.0;
        let b1 = 1.0 - cos_w;
        let b2 = (1.0 - cos_w) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_w;
        let a2 = 1.0 - alpha;
        
        // Normalize coefficients by a0
        FilterCoefficients {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }

    /// Reset the filter state (clear history)
    pub(crate) fn reset(&mut self) {
        self.x_history = [0.0; 2];
        self.y_history = [0.0; 2];
    }
}

impl LowPassFilterBuilder {
    pub fn build_with_coefficients(&mut self) -> Result<LowPassFilter, String> {
        // Clamp cutoff_frequency if set
        if let Some(cutoff) = self.cutoff_frequency {
            let clamped = cutoff.max(20.0).min(NYQUIST_FREQUENCY * 0.99);
            self.cutoff_frequency = Some(clamped);
        }
        let mut filter = self.build().map_err(|e| e.to_string())?;
        filter.update_coefficients();
        Ok(filter)
    }
}

/// Create a default low-pass filter
#[allow(dead_code)]
pub(crate) fn default_low_pass_filter() -> LowPassFilter {
    LowPassFilterBuilder::default()
        .cutoff_frequency(DEFAULT_CUTOFF_FREQUENCY)
        .resonance(DEFAULT_RESONANCE)
        .mix(DEFAULT_MIX)
        .build_with_coefficients().unwrap()
}

/// Create a low-pass filter that passes through the signal unchanged
#[allow(dead_code)]
pub(crate) fn no_op_low_pass_filter() -> LowPassFilter {
    LowPassFilterBuilder::default()
        .cutoff_frequency(NYQUIST_FREQUENCY)
        .resonance(0.0)
        .mix(0.0)
        .build_with_coefficients().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_filter_creation() {
        let filter = default_low_pass_filter();
        assert_eq!(filter.cutoff_frequency, DEFAULT_CUTOFF_FREQUENCY);
        assert_eq!(filter.resonance, DEFAULT_RESONANCE);
        assert_eq!(filter.mix, DEFAULT_MIX);
    }

    #[test]
    fn test_no_op_filter() {
        let mut filter = no_op_low_pass_filter();
        let input_sample = 0.5;
        let output = filter.apply_effect(input_sample, 0.0);
        // Should pass through unchanged since mix is 0.0
        assert!((output - input_sample).abs() < 1e-6);
    }

    #[test]
    fn test_filter_coefficients_calculation() {
        let mut filter = LowPassFilterBuilder::default()
            .cutoff_frequency(1000.0)
            .resonance(0.0)
            .build_with_coefficients().unwrap();
        
        // Coefficients should be calculated
        assert_ne!(filter.coefficients.b0, 0.0);
        assert_ne!(filter.coefficients.b1, 0.0);
        assert_ne!(filter.coefficients.b2, 0.0);
    }

    #[test]
    fn test_filter_frequency_clamping() {
        let mut filter = LowPassFilterBuilder::default()
            .cutoff_frequency(-100.0) // Invalid negative frequency
            .build_with_coefficients().unwrap();
        
        // Should be clamped to minimum frequency
        assert_eq!(filter.cutoff_frequency, 20.0);
        
        let mut filter = LowPassFilterBuilder::default()
            .cutoff_frequency(NYQUIST_FREQUENCY + 1000.0) // Invalid high frequency
            .build_with_coefficients().unwrap();
        
        // Should be clamped to just below Nyquist
        assert_eq!(filter.cutoff_frequency, NYQUIST_FREQUENCY * 0.99);
    }

    #[test]
    fn test_filter_reset() {
        let mut filter = default_low_pass_filter();
        
        // Process some samples to populate history
        filter.apply_effect(1.0, 0.0);
        filter.apply_effect(0.5, 0.0);
        
        // Reset should clear history
        filter.reset();
        assert_eq!(filter.x_history, [0.0; 2]);
        assert_eq!(filter.y_history, [0.0; 2]);
    }

    #[test]
    fn test_filter_mix_behavior() {
        let mut filter = LowPassFilterBuilder::default()
            .cutoff_frequency(100.0) // Low cutoff for noticeable effect
            .mix(0.5)
            .build_with_coefficients().unwrap();
        
        let input_sample = 1.0;
        let output = filter.apply_effect(input_sample, 0.0);
        
        // Output should be between input and fully filtered
        assert!(output < input_sample);
        assert!(output > 0.0);
    }

    #[test]
    fn test_filter_clone() {
        let original = default_low_pass_filter();
        let cloned = original.clone();
        
        assert_eq!(original.cutoff_frequency, cloned.cutoff_frequency);
        assert_eq!(original.resonance, cloned.resonance);
        assert_eq!(original.mix, cloned.mix);
    }
} 