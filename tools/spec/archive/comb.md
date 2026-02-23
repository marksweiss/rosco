# Comb Filter Specification

This document specifies the comb filter effect functionality for the Rosco audio system, inspired by the implementation in [pafx/comb.py](https://github.com/chenwj1989/pafx/blob/main/pafx/comb.py).

## Overview

A comb filter is an audio effect that creates a series of regularly spaced notches or peaks in the frequency response by mixing a signal with a delayed version of itself. It is commonly used for flanging, chorus, and reverb effects, and can be implemented with feedback and damping controls.

## Comb Struct

The `Comb` struct represents a feedback comb filter effect processor.

### Fields
- **delay_length** (`usize`): The length of the delay line (in samples).
- **feedback** (`f32`): Feedback coefficient (0.0 to <1.0).
- **damp** (`f32`): Damping factor for the feedback path (0.0 to 1.0).
- **delay** (`Delay`): Delay line buffer.
- **store** (`f32`): Internal state for the damped feedback signal.

### Parameters (for builder)
- `delay_length: usize` — Length of the delay line in samples.
- `feedback: f32` — Feedback coefficient.
- `damp: f32` — Damping factor.

## Builder Pattern

The `Comb` struct is constructed using a builder, following the [`derive_builder`](https://docs.rs/derive_builder/) pattern as used in other effects. This allows for ergonomic and flexible construction with sensible defaults and compile-time checks.

### `CombBuilder` Struct
- Each field in `Comb` has a corresponding setter method in `CombBuilder`.
- The builder validates that all required fields are set and that values are in valid ranges.
- The builder initializes the delay line and internal state.

#### Example Usage
```rust
let comb = CombBuilder::default()
    .delay_length(441)
    .feedback(0.7)
    .damp(0.2)
    .build()
    .unwrap();
```

### Builder Methods
- Each field has a corresponding setter (e.g., `.delay_length(usize)`, `.feedback(f32)`, etc.).
- The `.build()` method validates parameters and constructs the `Comb` struct.
- If validation fails, `.build()` returns an error.

## Methods

### `process(&mut self, x: f32) -> f32`
Processes a single input sample `x` and returns the output sample. The method:
- Reads the delayed sample from the delay line.
- Applies damping to the feedback path.
- Mixes the input with the feedback and writes to the delay line.

### `set_feedback(&mut self, feedback: f32)`
Sets the feedback coefficient.

### `set_damp(&mut self, damp: f32)`
Sets the damping factor.

## Implementation Notes

- The delay line should support efficient random access.
- All processing should be real-time and efficient for audio streaming. 