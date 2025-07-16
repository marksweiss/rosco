# LFO (Low Frequency Oscillator) Specification

This document specifies the LFO functionality for the Rosco audio system, inspired by the implementation in [pafx/lfo.py](https://github.com/chenwj1989/pafx/blob/main/pafx/lfo.py).

## Overview

A Low Frequency Oscillator (LFO) is a signal generator that produces periodic waveforms at frequencies below the audible range (typically <20 Hz). LFOs are used to modulate parameters of other effects, such as pitch, amplitude, or filter cutoff, to create vibrato, tremolo, and other time-varying effects.

## LFO Struct

The `LFO` struct represents a low-frequency oscillator for modulation purposes.

### Fields
- **sample_rate** (`f32`): The audio sample rate (Hz).
- **frequency** (`f32`): Frequency of the LFO (Hz).
- **width** (`f32`): Amplitude or modulation depth of the LFO.
- **waveform** (`Waveform`): Type of waveform (e.g., sine, triangle, etc.).
- **phase** (`f32`): Current phase of the oscillator (0.0 to 1.0).
- **bias** (`f32`): DC offset applied to the output.

### Parameters (for builder)
- `sample_rate: f32` — Audio sample rate in Hz.
- `frequency: f32` — LFO frequency in Hz.
- `width: f32` — Modulation depth.
- `waveform: Waveform` — Type of waveform (default: sine).
- `offset: f32` — Initial phase offset (default: 0.0).
- `bias: f32` — DC bias (default: 0.0).

## Builder Pattern

The `LFO` struct is constructed using a builder, following the [`derive_builder`](https://docs.rs/derive_builder/) pattern as used in other effects. This allows for ergonomic and flexible construction with sensible defaults and compile-time checks.

### `LFOBuilder` Struct
- Each field in `LFO` has a corresponding setter method in `LFOBuilder`.
- The builder validates that all required fields are set and that values are in valid ranges.
- The builder initializes the phase and computes the phase increment per sample.

#### Example Usage
```rust
let lfo = LFOBuilder::default()
    .sample_rate(44100.0)
    .frequency(0.5)
    .width(1.0)
    .waveform(Waveform::Sine)
    .offset(0.0)
    .bias(0.0)
    .build()
    .unwrap();
```

### Builder Methods
- Each field has a corresponding setter (e.g., `.frequency(f32)`, `.width(f32)`, etc.).
- The `.build()` method validates parameters and constructs the `LFO` struct.
- If validation fails, `.build()` returns an error.

## Methods

### `process(&self, n: u64) -> f32`
Returns the LFO value at sample index `n`.

### `tick(&mut self, i: u32) -> f32`
Advances the phase by `i` steps and returns the current LFO value.

## Implementation Notes

- The LFO should support at least sine waveform, with extensibility for others.
- All processing should be real-time and efficient for audio streaming. 