# Filter Module

This module implements audio filters for the Rosco audio system, following the specification in `tools/spec/filter.md`.

## Components

### LowPassFilter

A second-order IIR (Infinite Impulse Response) low-pass filter that attenuates frequencies above the cutoff frequency.

#### Features

- **Builder Pattern**: Uses `derive_builder` for easy construction
- **Real-time Processing**: Efficient IIR filter implementation
- **Parameter Validation**: Automatic clamping of frequencies to valid ranges
- **Mix Control**: Blend between filtered and dry signals
- **Resonance Control**: Adjustable Q factor for filter sharpness

#### Parameters

- `cutoff_frequency`: The frequency in Hz where filtering begins (20Hz to Nyquist)
- `resonance`: Q factor controlling filter sharpness (0.0 to 1.0)
- `mix`: Blend between original and filtered signal (0.0 = dry, 1.0 = fully filtered)

#### Usage

```rust
use crate::filter::low_pass_filter::*;

// Create a filter with default parameters
let mut filter = default_low_pass_filter();

// Or create with custom parameters
let mut filter = LowPassFilterBuilder::default()
    .cutoff_frequency(1000.0)
    .resonance(0.3)
    .mix(0.8)
    .build()
    .unwrap();

// Apply to audio samples
let filtered_sample = filter.apply_effect(input_sample, sample_clock);

// Reset filter state if needed
filter.reset();
```

#### Implementation Details

- Uses Direct Form II IIR filter structure
- Butterworth response with configurable Q
- Automatic coefficient calculation
- Thread-safe history management
- Efficient real-time processing

## Testing

Run the filter tests with:

```bash
cargo test filter::low_pass_filter::tests
```

## Examples

See `src/filter/example.rs` for usage examples and comparisons. 