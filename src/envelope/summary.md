# Envelope Module

## Purpose
Implements ADSR (Attack, Decay, Sustain, Release) envelope generation for controlling audio parameters over time. Envelopes are essential for shaping the amplitude and other characteristics of sounds.

## Key Components
- **envelope.rs**: Core ADSR envelope implementation
- **envelope_pair.rs**: Stereo envelope processing for left/right channels

## Architecture
The envelope module provides time-based control over audio parameters using the classic ADSR model:
- **Attack**: Time to reach peak level
- **Decay**: Time to fall from peak to sustain level
- **Sustain**: Level maintained during note hold
- **Release**: Time to fade to silence after note release

## Dependencies
- Uses common module utilities
- Integrates with audio_gen for amplitude control
- Works with track system for per-track envelope processing

## Usage Patterns
- Envelopes are applied to notes and tracks to control amplitude over time
- Envelope pairs handle stereo processing for spatial audio effects
- ADSR parameters can be configured per track or per note
- Envelopes provide natural-sounding amplitude curves for musical notes