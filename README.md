# Rosco - Music Composition Tools in Rust

A Rust-based music composition and audio generation toolkit.

## Features

- Audio generation and synthesis
- MIDI support
- Note sequencing and composition
- Effects processing (delay, flanger, LFO)
- Envelope generation
- Meter and timing utilities
- DSL for music composition

## Getting Started

### Prerequisites

- Rust (latest stable version)
- Cargo

### Installation

1. Clone the repository:
```bash
git clone <your-repo-url>
cd rosco
```

2. Initialize and update vendored tools:
```bash
git submodule update --init --recursive
./tools/update_vendored.sh
```

3. Build the project:
```bash
cargo build
```

### Running

```bash
cargo run
```

## Project Structure

- `src/` - Main source code
  - `audio_gen/` - Audio generation and synthesis
  - `common/` - Common utilities and constants
  - `composition/` - Composition utilities
  - `compositions/` - Pre-built compositions
  - `dsl/` - Domain-specific language for music
  - `effect/` - Audio effects processing
  - `envelope/` - Envelope generation
  - `meter/` - Timing and meter utilities
  - `midi/` - MIDI support
  - `note/` - Note handling and scales
  - `sequence/` - Note sequencing
  - `track/` - Track management
- `tools/` - Development and utility tools (including vendored third-party scripts)

## Vendored Tools

This project uses git submodules to vendor third-party tools in the `tools/` directory. These tools are kept separate from the main codebase for easier maintenance and updates.

### Managing Vendored Tools

To update all vendored tools:
```bash
./tools/update_vendored.sh
```

To add a new vendored tool:
```bash
git submodule add <repository-url> tools/<tool-name>
git submodule update --init --recursive
```

To remove a vendored tool:
```bash
git submodule deinit tools/<tool-name>
git rm tools/<tool-name>
rm -rf .git/modules/tools/<tool-name>
```

## Development

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Documentation

```bash
cargo doc --open
```

## License

[Add your license information here]

## Contributing

[Add contribution guidelines here] 