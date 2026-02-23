# Tremolo Effect Specification

This document specifies the tremolo effect functionality for the Rosco audio system, inspired by the implementation in [pafx/tremolo.py](https://github.com/chenwj1989/pafx/blob/main/pafx/tremolo.py).

## Overview

Tremolo is an audio effect that modulates the amplitude (volume) of a signal with a low-frequency oscillator (LFO), creating a periodic variation in loudness. It is commonly used for electric guitars, synthesizers, and vocals.

## Tremolo Struct

The `Tremolo` struct represents a tremolo effect processor.

### Fields
- **lfo** (`LFO`): Low-frequency oscillator used to modulate the amplitude.
- **mod_depth** (`f32`): Modulation depth (0.0 to 1.0).

### Parameters (for builder)
- `mod_freq: f32` — Modulation frequency (Hz).
- `mod_depth: f32` — Modulation depth (default: 0.5).
- `sample_rate: f32` — Audio sample rate (Hz, default: 44100.0).

## Builder Pattern

The `Tremolo` struct is constructed using a builder, following the [`derive_builder`](https://docs.rs/derive_builder/) pattern as used in other effects. This allows for ergonomic and flexible construction with sensible defaults and compile-time checks.

### `TremoloBuilder` Struct
- Each field in `Tremolo` has a corresponding setter method in `TremoloBuilder`.
- The builder validates that all required fields are set and that values are in valid ranges.
- The builder initializes the LFO and internal state.

#### Example Usage
```rust
let tremolo = TremoloBuilder::default()
    .mod_freq(5.0)
    .mod_depth(0.5)
    .sample_rate(44100.0)
    .build()
    .unwrap();
```

### Builder Methods
- Each field has a corresponding setter (e.g., `.mod_freq(f32)`, `.mod_depth(f32)`, etc.).
- The `.build()` method validates parameters and constructs the `Tremolo` struct.
- If validation fails, `.build()` returns an error.

## Methods

### `process(&mut self, x: f32) -> f32`
Processes a single input sample `x` and returns the output sample. The method:
- Computes the gain using the LFO and modulation depth.
- Multiplies the input by the computed gain.

## Implementation Notes

- The LFO should support at least sine waveform.
- All processing should be real-time and efficient for audio streaming. 