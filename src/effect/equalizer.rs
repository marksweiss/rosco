use derive_builder::Builder;

use crate::common::constants::SAMPLE_RATE;

static DEFAULT_NUM_BANDS: usize = 8;

// Standard octave center frequencies (Hz)
static DEFAULT_CENTER_FREQUENCIES: [f32; 8] = [
    63.0, 125.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0
];

// Default Q factor for peaking filters
static DEFAULT_Q: f32 = 1.414;

#[derive(Clone, Debug)]
struct BiquadState {
    // Coefficients
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    // Direct Form II state
    z1: f32,
    z2: f32,
}

impl BiquadState {
    fn process(&mut self, input: f32) -> f32 {
        // Direct Form II Transposed
        let output = self.b0 * input + self.z1;
        self.z1 = self.b1 * input - self.a1 * output + self.z2;
        self.z2 = self.b2 * input - self.a2 * output;
        output
    }

    fn reset(&mut self) {
        self.z1 = 0.0;
        self.z2 = 0.0;
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum FilterType {
    LowShelf,
    Peaking,
    HighShelf,
}

fn compute_biquad(filter_type: FilterType, sample_rate: f32, freq: f32,
                  gain_db: f32, q: f32) -> BiquadState {
    let a = 10.0_f32.powf(gain_db / 40.0); // sqrt of linear gain
    let w0 = 2.0 * std::f32::consts::PI * freq / sample_rate;
    let cos_w0 = w0.cos();
    let sin_w0 = w0.sin();
    let alpha = sin_w0 / (2.0 * q);

    let (b0, b1, b2, a0, a1, a2) = match filter_type {
        FilterType::LowShelf => {
            let two_sqrt_a_alpha = 2.0 * a.sqrt() * alpha;
            (
                a * ((a + 1.0) - (a - 1.0) * cos_w0 + two_sqrt_a_alpha),
                2.0 * a * ((a - 1.0) - (a + 1.0) * cos_w0),
                a * ((a + 1.0) - (a - 1.0) * cos_w0 - two_sqrt_a_alpha),
                (a + 1.0) + (a - 1.0) * cos_w0 + two_sqrt_a_alpha,
                -2.0 * ((a - 1.0) + (a + 1.0) * cos_w0),
                (a + 1.0) + (a - 1.0) * cos_w0 - two_sqrt_a_alpha,
            )
        }
        FilterType::Peaking => {
            (
                1.0 + alpha * a,
                -2.0 * cos_w0,
                1.0 - alpha * a,
                1.0 + alpha / a,
                -2.0 * cos_w0,
                1.0 - alpha / a,
            )
        }
        FilterType::HighShelf => {
            let two_sqrt_a_alpha = 2.0 * a.sqrt() * alpha;
            (
                a * ((a + 1.0) + (a - 1.0) * cos_w0 + two_sqrt_a_alpha),
                -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_w0),
                a * ((a + 1.0) + (a - 1.0) * cos_w0 - two_sqrt_a_alpha),
                (a + 1.0) - (a - 1.0) * cos_w0 + two_sqrt_a_alpha,
                2.0 * ((a - 1.0) - (a + 1.0) * cos_w0),
                (a + 1.0) - (a - 1.0) * cos_w0 - two_sqrt_a_alpha,
            )
        }
    };

    // Normalize by a0
    BiquadState {
        b0: b0 / a0,
        b1: b1 / a0,
        b2: b2 / a0,
        a1: a1 / a0,
        a2: a2 / a0,
        z1: 0.0,
        z2: 0.0,
    }
}

#[derive(Builder, Debug)]
#[builder(build_fn(skip))]
pub(crate) struct Equalizer {
    #[builder(default = "SAMPLE_RATE")]
    pub(crate) sample_rate: f32,

    #[builder(default = "DEFAULT_NUM_BANDS")]
    pub(crate) num_bands: usize,

    // Gains in dB per band
    #[builder(default = "vec![0.0; DEFAULT_NUM_BANDS]")]
    pub(crate) gains: Vec<f32>,

    // Center frequencies per band
    #[builder(default = "DEFAULT_CENTER_FREQUENCIES.to_vec()")]
    pub(crate) center_frequencies: Vec<f32>,

    // Biquad filter states (private, computed at build time)
    #[builder(field(private))]
    filters: Vec<BiquadState>,
}

impl Clone for Equalizer {
    fn clone(&self) -> Self {
        let filters = self.filters.iter().map(|f| {
            let mut cloned = f.clone();
            cloned.reset();
            cloned
        }).collect();

        Equalizer {
            sample_rate: self.sample_rate,
            num_bands: self.num_bands,
            gains: self.gains.clone(),
            center_frequencies: self.center_frequencies.clone(),
            filters,
        }
    }
}

impl PartialEq for Equalizer {
    fn eq(&self, other: &Self) -> bool {
        self.sample_rate == other.sample_rate &&
        self.num_bands == other.num_bands &&
        self.gains == other.gains &&
        self.center_frequencies == other.center_frequencies
    }
}

#[allow(dead_code)]
impl EqualizerBuilder {
    pub(crate) fn build(&mut self) -> Result<Equalizer, String> {
        let sample_rate = self.sample_rate.unwrap_or(SAMPLE_RATE);
        let num_bands = self.num_bands.unwrap_or(DEFAULT_NUM_BANDS);
        let gains = self.gains.clone()
            .unwrap_or_else(|| vec![0.0; num_bands]);
        let center_frequencies = self.center_frequencies.clone()
            .unwrap_or_else(|| DEFAULT_CENTER_FREQUENCIES.to_vec());

        if gains.len() != num_bands {
            return Err(format!("gains length {} does not match num_bands {}",
                gains.len(), num_bands));
        }
        if center_frequencies.len() != num_bands {
            return Err(format!("center_frequencies length {} does not match num_bands {}",
                center_frequencies.len(), num_bands));
        }

        // Compute biquad filters for each band
        let filters: Vec<BiquadState> = (0..num_bands).map(|i| {
            let filter_type = if i == 0 {
                FilterType::LowShelf
            } else if i == num_bands - 1 {
                FilterType::HighShelf
            } else {
                FilterType::Peaking
            };
            compute_biquad(filter_type, sample_rate, center_frequencies[i],
                           gains[i], DEFAULT_Q)
        }).collect();

        Ok(Equalizer {
            sample_rate,
            num_bands,
            gains,
            center_frequencies,
            filters,
        })
    }
}

impl Equalizer {
    pub(crate) fn apply_effect(&mut self, sample: f32, _sample_clock: f32) -> f32 {
        let mut output = sample;
        for filter in self.filters.iter_mut() {
            output = filter.process(output);
        }
        output
    }
}

#[allow(dead_code)]
pub(crate) fn default_equalizer() -> Equalizer {
    EqualizerBuilder::default()
        .build().unwrap()
}

#[allow(dead_code)]
pub(crate) fn no_op_equalizer() -> Equalizer {
    // All gains at 0 dB = unity pass-through
    EqualizerBuilder::default()
        .build().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_construction() {
        let eq = default_equalizer();
        assert_eq!(eq.num_bands, DEFAULT_NUM_BANDS);
        assert_eq!(eq.gains.len(), DEFAULT_NUM_BANDS);
        assert_eq!(eq.center_frequencies.len(), DEFAULT_NUM_BANDS);
        assert_eq!(eq.filters.len(), DEFAULT_NUM_BANDS);
    }

    #[test]
    fn test_custom_construction() {
        let eq = EqualizerBuilder::default()
            .num_bands(4)
            .gains(vec![3.0, -2.0, 0.0, 6.0])
            .center_frequencies(vec![100.0, 500.0, 2000.0, 8000.0])
            .build().unwrap();
        assert_eq!(eq.num_bands, 4);
        assert_eq!(eq.gains, vec![3.0, -2.0, 0.0, 6.0]);
    }

    #[test]
    fn test_mismatched_gains_length() {
        let result = EqualizerBuilder::default()
            .num_bands(4)
            .gains(vec![0.0, 0.0]) // wrong length
            .center_frequencies(vec![100.0, 500.0, 2000.0, 8000.0])
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_no_op_identity() {
        let mut eq = no_op_equalizer();
        let input = 0.5;
        // Run a steady-state signal through to let filter transients settle
        for _ in 0..1000 {
            eq.apply_effect(input, 0.0);
        }
        let output = eq.apply_effect(input, 0.0);
        // With all gains at 0 dB, output should be very close to input
        assert!((output - input).abs() < 0.01,
            "no-op equalizer should approximately pass signal through, got {}", output);
    }

    #[test]
    fn test_effect_with_boost() {
        let mut eq = EqualizerBuilder::default()
            .gains(vec![12.0, 12.0, 12.0, 12.0, 12.0, 12.0, 12.0, 12.0])
            .build().unwrap();
        let input = 0.3;
        // Feed signal to settle transients
        for _ in 0..1000 {
            eq.apply_effect(input, 0.0);
        }
        let output = eq.apply_effect(input, 0.0);
        // Boosting all bands should increase signal level
        assert!(output.abs() > input.abs(),
            "boosted EQ should increase signal level, got {}", output);
    }

    #[test]
    fn test_clone_resets_state() {
        let mut eq = default_equalizer();
        for _ in 0..1000 {
            eq.apply_effect(0.5, 0.0);
        }
        let cloned = eq.clone();
        // Filter state should be reset
        for filter in &cloned.filters {
            assert_eq!(filter.z1, 0.0);
            assert_eq!(filter.z2, 0.0);
        }
    }

    #[test]
    fn test_partial_eq() {
        let a = default_equalizer();
        let b = default_equalizer();
        assert_eq!(a, b);

        let c = EqualizerBuilder::default()
            .gains(vec![6.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
            .build().unwrap();
        assert_ne!(a, c);
    }
}
