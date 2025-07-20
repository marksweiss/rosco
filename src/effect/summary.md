# Effects Module

## Purpose
Implements audio effects processing capabilities for the Rosco toolkit. This module provides various time-based and modulation effects that can be applied to tracks.

## Key Components
- **delay.rs**: Digital delay effect implementation
- **flanger.rs**: Flanger effect with modulation capabilities
- **lfo.rs**: Low-frequency oscillator for modulation effects

## Architecture
Effects are designed to process audio in real-time and can be applied to tracks through the track effects system. Each effect typically provides parameters for controlling intensity, timing, and modulation characteristics.

## Dependencies
- Uses common module for shared utilities
- Integrates with track module through TrackEffects
- May use rand for random modulation in certain effects

## Usage Patterns
- Effects are applied to tracks via the TrackEffects system
- LFO can be used to modulate other effects or audio parameters
- Delay provides echo and reverb-like effects
- Flanger creates sweeping, modulated delay effects
- Effects can be chained and combined on individual tracks