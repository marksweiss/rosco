# Chorus Effect Specification

This document specifies the chorus effect functionality for the Rosco audio system, inspired by the implementation in [pafx/chorus.py](https://github.com/chenwj1989/pafx/blob/main/pafx/chorus.py).

## Overview

A chorus effect simulates the sound of multiple instruments or voices playing in unison by mixing the original (dry) signal with several delayed and modulated copies. The result is a richer, thicker sound with subtle pitch and timing variations, commonly used for vocals, guitars, and synthesizers.

The effect is achieved by passing the input signal through multiple delay lines, each modulated by a low-frequency oscillator (LFO) with different parameters. The modulated signals are mixed with the dry signal, each with its own gain.

## Chorus Struct

The `Chorus` struct represents a multi-voice chorus effect processor.

### Fields
- **sample_rate** (`f32`): The audio sample rate (Hz).
- **chorus_count** (`usize`): Number of chorus voices (delay+LFO paths).
- **chorus_gains** (`Vec<f32>`): Gain for each chorus voice.
- **dry_gain** (`f32`): Gain for the dry (unaffected) signal.
- **chorus_delays** (`Vec<usize>`): Base delay (in samples) for each chorus voice.
- **lfo_array** (`Vec<LFO>`): Array of LFOs, one per chorus voice, modulating the delay time.
- **delay_line** (`Delay`): A single delay line buffer used for all chorus taps.

### Parameters (for builder)
- `sample_rate: f32` — Audio sample rate in Hz.
- `delays: Vec<f32>` — Base delay times for each chorus voice (in seconds).
- `mod_freqs: Vec<f32>` — LFO frequency for each chorus voice (in Hz).
- `mod_width: Vec<f32>` — LFO modulation width for each chorus voice (in seconds).
- `chorus_gains: Vec<f32>` — Output gain for each chorus voice.
- `dry_gain: f32` — Output gain for the dry signal (default: 1.0).

## Builder Pattern

The `Chorus` struct is constructed using a builder, following the [`derive_builder`](https://docs.rs/derive_builder/) pattern as used in other effects in the system. This allows for ergonomic and flexible construction with sensible defaults and compile-time checks.

### `ChorusBuilder` Struct
- Each field in `Chorus` has a corresponding setter method in `ChorusBuilder`.
- Default values are provided for fields where appropriate (e.g., `dry_gain = 1.0`).
- The builder validates that all required fields are set and that vector lengths match where necessary.
- The builder computes any derived fields (e.g., converting delay times in seconds to samples, initializing LFOs and the delay line).

#### Example Usage
```rust
let chorus = ChorusBuilder::default()
    .sample_rate(44100.0)
    .delays(vec![0.015, 0.020, 0.030])
    .mod_freqs(vec![0.25, 0.33, 0.40])
    .mod_width(vec![0.003, 0.004, 0.005])
    .chorus_gains(vec![0.5, 0.5, 0.5])
    .dry_gain(1.0)
    .build()
    .unwrap();
```

### Builder Methods
- Each field has a corresponding setter (e.g., `.sample_rate(f32)`, `.delays(Vec<f32>)`, etc.).
- The `.build()` method validates parameters and constructs the `Chorus` struct.
- If validation fails (e.g., mismatched vector lengths), `.build()` returns an error.

## Methods

### `process(&mut self, x: f32) -> f32`
Processes a single input sample `x` and returns the output sample. The method:
- Mixes the dry signal (scaled by `dry_gain`).
- For each chorus voice:
    - Computes the modulated delay using the LFO.
    - Reads the delayed sample from the delay line (with linear interpolation).
    - Scales the delayed sample by the corresponding gain and adds it to the output.
- Pushes the input sample into the delay line buffer.

### `validate(...)`
Validates the input parameters for consistency (e.g., matching vector lengths, valid ranges).

## Implementation Notes

- The delay line should support random access and linear interpolation for fractional delay times.
- LFOs should be implemented as low-frequency sine oscillators.
- All processing should be real-time and efficient for audio streaming.
- The chorus effect can be extended to stereo by processing each channel independently or with cross-modulation. 