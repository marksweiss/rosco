# NOTE Module
*Last auto-updated: 2025-08-20 01:28:37*
*This summary was automatically updated due to source file changes in this commit.*

*Last auto-updated: 2025-07-28 00:17:35*
*This summary was automatically updated due to source file changes in this commit.*


## Purpose
Provides core note representation, musical scales, and note management functionality. This module defines the fundamental musical building blocks used throughout the Rosco system.

## Key Components
- **note.rs**: Core note data structures and functionality
- **note_trait.rs**: Common interface for different note types
- **playback_note.rs**: Oscillator-based notes for synthesized sounds
- **sampled_note.rs**: Sample-based notes for recorded audio playback
- **note_pool.rs**: Note collection and management utilities
- **scales.rs**: Musical scale definitions and utilities
- **constants.rs**: Note-related constants and configuration

## Architecture
The note module supports two primary note types:
- **PlaybackNote**: Oscillator-based notes using synthesis (`osc:waveform:frequency:volume:step_index`)
- **SampledNote**: Audio sample-based notes using recorded audio (`samp:file_path:volume:step_index`)

Both note types implement the common note trait for consistent interface across the system.

## Dependencies
- Integrates with audio_gen for sound generation
- Works with scales for musical theory support
- Uses common module for shared utilities

## Usage Patterns
- Notes are created with specific timing, pitch, and volume parameters
- Note pools manage collections of notes for sequences
- Scales provide musical context and note relationships
- Note trait ensures consistent interface regardless of note type
- Step index provides timing information for sequencing