# AUDIO GEN Module
*Last auto-updated: 2025-07-28 00:17:35*
*This summary was automatically updated due to source file changes in this commit.*

*Last auto-updated: 2025-07-19 21:59:52*
*This summary was automatically updated due to source file changes in this commit.*

*Last auto-updated: 2025-07-19 21:57:08*
*This summary was automatically updated due to source file changes in this commit.*


## Purpose
Provides core audio synthesis and generation capabilities for the Rosco toolkit. This module handles oscillator-based sound generation and audio sample processing.

## Key Components
- **oscillator.rs**: Core oscillator implementations for different waveforms (sine, square, triangle, sawtooth, noise)
- **audio_gen.rs**: Main audio generation logic and coordination
- **get_sample.rs**: Sample retrieval and processing utilities

## Architecture
The audio generation module serves as the foundation for sound production in Rosco. It provides both oscillator-based synthesis for generated sounds and sample-based playback for recorded audio. The module is designed to work with the cpal audio library for real-time audio output.

## Dependencies
- Works closely with note module for playback instructions
- Integrates with track module for audio routing
- Uses common module utilities for shared functionality

## Usage Patterns
- Oscillators are configured with waveform type, frequency, volume, and timing
- Sample-based generation loads and processes WAV files via hound library
- Audio generation is coordinated through the main audio_gen module