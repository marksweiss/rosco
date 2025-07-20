# Domain-Specific Language (DSL) Module

## Purpose
Implements a custom domain-specific language for music composition, allowing users to write musical compositions in a high-level, declarative syntax rather than imperative Rust code.

## Key Components
- **parser.rs**: Main DSL parsing logic and syntax processing
- **README.md**: Documentation for DSL syntax and usage
- **test_filter.dsl**: Example DSL script demonstrating filter usage
- **test_data/**: Test assets including sample audio files

## Architecture
The DSL parser processes script input to create a `Vec<Track>` structure:
1. Processes macro declarations (`let identifier = expression`) at script top
2. Parses outer blocks containing sequence definitions, envelopes, effects, and note declarations
3. Creates `FixedTimeNoteSequence` and `TrackEffects` for each outer block
4. Builds tracks with sequences and effects, returning a `TrackGrid`

## Dependencies
- Uses regex for parsing DSL syntax
- Integrates with track, sequence, note, and effect modules
- Works with filter module for audio processing effects

## Usage Patterns
- DSL scripts use `.dsl` file extension
- Supports macro system for reusable composition elements
- Provides declarative syntax for note sequences, effects, and envelopes
- Note syntax: `osc:waveform:frequency:volume:step_index` for oscillators, `samp:file_path:volume:step_index` for samples