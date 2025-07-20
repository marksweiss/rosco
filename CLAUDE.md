# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Build and Run
```bash
cargo build           # Build the project
cargo run             # Run the main application (currently plays dsl_1 composition)
```

### Testing
```bash
cargo test                                    # Run all tests
cargo test filter::low_pass_filter::tests    # Run specific filter tests
cargo test filter::high_pass_filter::tests   # Run high-pass filter tests
cargo test filter::band_pass_filter::tests   # Run band-pass filter tests
cargo test filter::notch_filter::tests       # Run notch filter tests
```

### Documentation
```bash
cargo doc --open      # Generate and open documentation
```

### Vendored Tools Management
```bash
git submodule update --init --recursive   # Initialize submodules
./tools/update_vendored.sh                # Update all vendored tools
```

## Architecture Overview

Rosco is a Rust-based music composition and audio generation toolkit with a modular architecture:

### Core Modules
- **audio_gen/**: Audio synthesis and generation engine with oscillators
- **note/**: Note representation, scales, and note pools (PlaybackNote for oscillator-based notes, SampledNote for audio samples)
- **sequence/**: Note sequencing with FixedTimeNoteSequence, GridNoteSequence, and TimeNoteSequence
- **track/**: Track management with effects processing and grid-based organization
- **effect/**: Audio effects (delay, flanger, LFO)
- **filter/**: IIR filters (low-pass, high-pass, band-pass, notch) with builder pattern
- **envelope/**: ADSR envelope generation
- **midi/**: MIDI support and integration
- **meter/**: Timing, duration, and meter utilities
- **composition/**: High-level composition utilities
- **dsl/**: Domain-specific language for music composition with macro support

### DSL Architecture
The DSL parser creates a `Vec<Track>` from script input:
1. Processes macro declarations (`let identifier = expression`) at script top
2. Parses outer blocks containing sequence definitions, envelopes, effects, and note declarations
3. Creates `FixedTimeNoteSequence` and `TrackEffects` for each outer block
4. Builds tracks with sequences and effects, returning a `TrackGrid`

### Filter System
All filters use Direct Form II IIR structure with:
- Builder pattern construction via derive_builder
- Real-time processing capabilities
- Mix control (dry/wet blend)
- Resonance control (Q factor)
- Thread-safe history management

### Note Types
- **OSC Notes**: `osc:waveform:frequency:volume:step_index` (sine, square, triangle, sawtooth, noise)
- **Sample Notes**: `samp:file_path:volume:step_index` (audio file playback)

### Track Effects
Tracks support:
- Multiple filters per track
- Stereo panning (-1.0 to 1.0)
- Effects processing (delay, flanger, LFO)
- ADSR envelopes

## Composition Workflow

1. **Main Entry**: `src/main.rs` runs compositions from `compositions/` module
2. **DSL Scripts**: Use `.dsl` files with the custom music composition language
3. **Audio Output**: Generated audio is output via cpal audio library
4. **Sample Support**: Use hound for WAV file processing

## Key Dependencies
- `cpal`: Cross-platform audio I/O
- `hound`: WAV file reading/writing
- `derive_builder`: Builder pattern generation
- `nodi`: MIDI processing
- `regex`: DSL parsing
- `rand`: Random number generation for effects