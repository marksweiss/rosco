# Sequence Module

## Purpose
Implements note sequencing functionality for organizing musical events in time. This module provides different sequencing strategies for various musical composition needs.

## Key Components
- **note_sequence_trait.rs**: Common interface for all sequence types
- **fixed_time_note_sequence.rs**: Sequences with fixed timing intervals
- **grid_note_sequence.rs**: Grid-based sequencing for rhythmic patterns
- **time_note_sequence.rs**: Flexible time-based sequencing

## Architecture
The sequence module provides multiple sequencing approaches:
- **FixedTimeNoteSequence**: Regular, predictable timing intervals
- **GridNoteSequence**: Step-based grid sequencing for drum patterns and rhythmic elements
- **TimeNoteSequence**: Flexible timing for complex musical phrases

All sequence types implement the common sequence trait for consistent interface.

## Dependencies
- Uses note module for musical content
- Integrates with meter module for timing calculations
- Works with track module for sequence playback
- Connects to DSL module for declarative sequence definition

## Usage Patterns
- Sequences organize notes in temporal order
- Different sequence types suit different musical styles and needs
- Grid sequences excel at rhythmic patterns and drum programming
- Time sequences provide flexibility for complex melodic phrases
- Fixed time sequences offer predictable, regular timing patterns