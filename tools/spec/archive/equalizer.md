# Equalizer Effect Specification

This document specifies the equalizer effect functionality for the Rosco audio system, inspired by the implementation in [pafx/equalizer.py](https://github.com/chenwj1989/pafx/blob/main/pafx/equalizer.py).

## Overview

An equalizer (EQ) is an audio effect that adjusts the balance of frequency components in an audio signal. It typically consists of multiple frequency bands, each with its own gain, allowing for precise control over the tonal balance of the sound. The most common implementation uses a series of filters (e.g., low shelf, peaking, high shelf) to shape the frequency response.

## Equalizer Struct

The `Equalizer` struct represents a multi-band equalizer effect processor.

### Fields
- **sample_rate** (`f32`): The audio sample rate (Hz).
- **num_bands** (`usize`): Number of EQ bands (typically 7-10, depending on sample rate).
- **gains** (`Vec<f32>`): Gain for each band (dB or linear).
- **filters** (`Vec<Biquad>`): Array of biquad filter objects, one per band.

### Parameters (for builder)
- `sample_rate: f32` — Audio sample rate in Hz.
- `gains: Vec<f32>` — Gain for each band.

## Builder Pattern

The `Equalizer` struct is constructed using a builder, following the [`derive_builder`](https://docs.rs/derive_builder/) pattern as used in other effects. This allows for ergonomic and flexible construction with sensible defaults and compile-time checks.

### `EqualizerBuilder` Struct
- Each field in `Equalizer` has a corresponding setter method in `EqualizerBuilder`.
- The builder validates that all required fields are set and that the number of gains matches the number of bands for the given sample rate.
- The builder initializes the appropriate biquad filters for each band (low shelf, peaking, high shelf).

#### Example Usage
```rust
let eq = EqualizerBuilder::default()
    .sample_rate(44100.0)
    .gains(vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
    .build()
    .unwrap();
```

### Builder Methods
- Each field has a corresponding setter (e.g., `.sample_rate(f32)`, `.gains(Vec<f32>)`).
- The `.build()` method validates parameters and constructs the `Equalizer` struct.
- If validation fails (e.g., mismatched vector lengths), `.build()` returns an error.

## Methods

### `process(&mut self, x: f32) -> f32`
Processes a single input sample `x` and returns the output sample. The method:
- Passes the input through each filter in sequence, applying the gain for each band.
- Sums or mixes the outputs as appropriate.

### `dump(&self)`
Prints or logs the state of each filter for debugging or analysis.

## Implementation Notes

- Filters should be implemented as biquad IIR filters (low shelf, peaking, high shelf).
- The number of bands and their center frequencies/bandwidths should be chosen based on the sample rate.
- All processing should be real-time and efficient for audio streaming. 