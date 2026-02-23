# Tapped Delay Line Specification

This document specifies the tapped delay line functionality for the Rosco audio system, inspired by the implementation in [pafx/tapped_delay_line.py](https://github.com/chenwj1989/pafx/blob/main/pafx/tapped_delay_line.py).

## Overview

A tapped delay line is an audio processing structure that provides multiple delayed versions (taps) of an input signal, each with its own gain. It is a fundamental building block for effects such as reverb, echo, and multi-tap delay.

## TappedDelayLine Struct

The `TappedDelayLine` struct represents a tapped delay line effect processor.

### Fields
- **delay_length** (`usize`): The length of the delay buffer (in samples).
- **buffer** (`Vec<f32>`): Circular buffer for storing past samples.
- **tap_delays** (`Vec<usize>`): Delay (in samples) for each tap.
- **tap_gains** (`Vec<f32>`): Gain for each tap.
- **pos** (`usize`): Current write position in the buffer.

### Parameters (for builder)
- `tap_delays: Vec<usize>` — Delay (in samples) for each tap.
- `tap_gains: Vec<f32>` — Gain for each tap.

## Builder Pattern

The `TappedDelayLine` struct is constructed using a builder, following the [`derive_builder`](https://docs.rs/derive_builder/) pattern as used in other effects. This allows for ergonomic and flexible construction with sensible defaults and compile-time checks.

### `TappedDelayLineBuilder` Struct
- Each field in `TappedDelayLine` has a corresponding setter method in `TappedDelayLineBuilder`.
- The builder validates that all required fields are set and that vector lengths match.
- The builder computes the required buffer length and initializes the buffer.

#### Example Usage
```rust
let tapped_delay = TappedDelayLineBuilder::default()
    .tap_delays(vec![100, 200, 400])
    .tap_gains(vec![0.5, 0.3, 0.2])
    .build()
    .unwrap();
```

### Builder Methods
- Each field has a corresponding setter (e.g., `.tap_delays(Vec<usize>)`, `.tap_gains(Vec<f32>)`, etc.).
- The `.build()` method validates parameters and constructs the `TappedDelayLine` struct.
- If validation fails, `.build()` returns an error.

## Methods

### `process(&mut self, x: f32) -> f32`
Processes a single input sample `x` and returns the output sample. The method:
- Writes the input sample to the buffer at the current position.
- For each tap, reads the delayed sample from the buffer, multiplies by the tap gain, and sums the results.
- Advances the buffer position.
- Returns the summed output.

## Implementation Notes

- The buffer should be implemented as a circular buffer for efficiency.
- All processing should be real-time and efficient for audio streaming. 