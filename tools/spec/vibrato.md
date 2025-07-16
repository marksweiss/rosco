# Vibrato Effect Specification

This document specifies the vibrato effect functionality for the Rosco audio system, inspired by the implementation in [pafx/vibrato.py](https://github.com/chenwj1989/pafx/blob/main/pafx/vibrato.py).

## Overview

Vibrato is an audio effect that modulates the pitch of a signal with a low-frequency oscillator (LFO), creating a periodic variation in pitch. It is commonly used for vocals, guitars, and synthesizers.

## Vibrato Struct

The `Vibrato` struct represents a vibrato effect processor.

### Fields
- **sample_rate** (`f32`): The audio sample rate (Hz).
- **avg_delay** (`usize`): Average delay in samples.
- **mod_width** (`usize`): Modulation width in samples.
- **delay_line** (`Delay`): Delay line buffer for pitch modulation.
- **lfo** (`LFO`): Low-frequency oscillator for modulating the delay.

### Parameters (for builder)
- `sample_rate: f32` — Audio sample rate in Hz.
- `delay: f32` — Base delay time (in seconds).
- `mod_width: f32` — Modulation width (in seconds).
- `mod_freq: f32` — LFO frequency (in Hz).

## Builder Pattern

The `Vibrato` struct is constructed using a builder, following the [`derive_builder`](https://docs.rs/derive_builder/) pattern as used in other effects. This allows for ergonomic and flexible construction with sensible defaults and compile-time checks.

### `VibratoBuilder` Struct
- Each field in `Vibrato` has a corresponding setter method in `VibratoBuilder`.
- The builder validates that all required fields are set and that values are in valid ranges.
- The builder computes the required delay line length and initializes the LFO.

#### Example Usage
```rust
let vibrato = VibratoBuilder::default()
    .sample_rate(44100.0)
    .delay(0.005)
    .mod_width(0.002)
    .mod_freq(5.0)
    .build()
    .unwrap();
```

### Builder Methods
- Each field has a corresponding setter (e.g., `.delay(f32)`, `.mod_width(f32)`, etc.).
- The `.build()` method validates parameters and constructs the `Vibrato` struct.
- If validation fails, `.build()` returns an error.

## Methods

### `process(&mut self, x: f32) -> f32`
Processes a single input sample `x` and returns the output sample. The method:
- Computes the modulated delay using the LFO.
- Reads the delayed sample from the delay line (with linear interpolation).
- Pushes the input sample into the delay line buffer.
- Returns the interpolated output sample.

## Implementation Notes

- The delay line should support random access and linear interpolation for fractional delay times.
- The LFO should support at least sine waveform.
- All processing should be real-time and efficient for audio streaming. 