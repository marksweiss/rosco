# Rosco TUI (Terminal User Interface)

A terminal-based synthesizer and step sequencer interface for Rosco, built with [ratatui](https://ratatui.rs/) and [crossterm](https://github.com/crossterm-rs/crossterm).

## Launching the TUI

```bash
# From the project root directory
cargo run --bin rosco-tui
```

Or build first, then run:

```bash
cargo build --release
./target/release/rosco-tui
```

## Overview

The TUI provides real-time control over:
- **Synthesizer** - Oscillator, Filter, Envelope, and Effects controls
- **Step Sequencer** - 8-track × 16-step grid with per-step pitch control
- **Track Controls** - Volume, Pan, Mute, Solo per track
- **Transport** - Play/Stop and tempo control
- **Pattern Management** - Save, load, copy, and paste patterns

## Screen Layout

```
┌─────────────────────────────────────────────────────────────┐
│                      SYNTHESIZER                             │
│ ┌──────────┬──────────┬──────────┬──────────┐              │
│ │ 1-OSC    │ 2-FILTER │ 3-ENV    │ 4-FX     │              │
│ └──────────┴──────────┴──────────┴──────────┘              │
├─────────────────────────────────────────────────────────────┤
│ ┌─────────────────┬─────────────┬─────────────┐            │
│ │ 5-TRACK GRID    │ 6-VOLUME    │ 7-PANNING   │            │
│ │                 │             │             │            │
│ └─────────────────┴─────────────┴─────────────┘            │
│ ┌─────────────────────────────────────────────┐            │
│ │ 8-TRANSPORT                                  │            │
│ └─────────────────────────────────────────────┘            │
├─────────────────────────────────────────────────────────────┤
│ Status Bar                                                   │
└─────────────────────────────────────────────────────────────┘
```

## Keyboard Shortcuts

### Global Controls

| Key | Action |
|-----|--------|
| `F1` | Toggle help screen |
| `Esc` | Close overlay / Quit application |
| `q` | Quit application |
| `Tab` | Cycle through focus areas |
| `1`-`8` | Quick switch to section (Osc/Filter/Env/FX/Grid/Vol/Pan/Transport) |

### Navigation

| Key | Action |
|-----|--------|
| `↑` `↓` | Navigate between controls / tracks |
| `←` `→` | Adjust values / Navigate steps |
| `Enter` / `Space` | Activate / Toggle |

### Synthesizer Controls (Sections 1-4)

| Key | Action |
|-----|--------|
| `↑` `↓` | Move between parameters in section |
| `←` `→` | Adjust parameter value |
| `+` / `=` | Fine increment adjustment |
| `-` | Fine decrement adjustment |
| `r` | Reset parameter to default |

#### 1 - Oscillator Section
- **Waveform**: Sine, Square, Triangle, Sawtooth, Noise
- **Volume**: 0% - 100%

#### 2 - Filter Section
- **Type**: LowPass, HighPass, BandPass, Notch
- **Frequency/Cutoff**: 20 Hz - 20 kHz (logarithmic)
- **Bandwidth**: 10 Hz - 5 kHz (BandPass/Notch only)
- **Resonance**: 0% - 100%
- **Mix**: 0% - 100% (dry/wet)

#### 3 - Envelope Section (ADSR)
- **Attack Time/Level**
- **Decay Time/Level**
- **Sustain Time/Level**
- **Release Time/Level**

#### 4 - Effects Section
- **Delay**: Time, Feedback, Mix
- **Flanger**: Rate, Depth, Mix
- **LFO**: Rate, Depth

### Track Grid (Section 5)

| Key | Action |
|-----|--------|
| `↑` `↓` | Navigate between step/frequency rows |
| `←` `→` | Navigate between steps |
| `Tab` | Switch between Steps and Track Controls |
| `Enter` / `Space` | Toggle step (Steps mode) / Open frequency dropdown (Frequency mode) |
| `A`-`H` | Quick select tracks 1-8 |
| `1`-`9`, `0` | Quick select steps 1-10 |

#### Frequency Dropdown Mode
| Key | Action |
|-----|--------|
| `↑` `↓` | Change pitch (C, C#, D, D#, E, F, F#, G, G#, A, A#, B) |
| `Enter` / `Space` / `Esc` | Close dropdown |

### Track Volume (Section 6)

| Key | Action |
|-----|--------|
| `↑` `↓` | Navigate between tracks |
| `←` `→` | Adjust volume (±5%) |

### Track Panning (Section 7)

| Key | Action |
|-----|--------|
| `↑` `↓` | Navigate between tracks |
| `←` `→` | Adjust pan (±10%, -100% = Left, +100% = Right) |

### Transport (Section 8)

| Key | Action |
|-----|--------|
| `←` `→` | Switch between Play and Stop buttons |
| `Enter` / `Space` | Activate focused button |

### Pattern Management

| Key | Action |
|-----|--------|
| `F2` | Save current track as new pattern |
| `F3` | Open/close pattern browser |
| `Ctrl+P` | Toggle pattern browser (alternative) |
| `Alt+S` | Quick save pattern |
| `Alt+L` | Load most recent pattern to current track |

#### Pattern Browser
| Key | Action |
|-----|--------|
| `↑` `↓` | Navigate pattern list |
| `Enter` | Load selected pattern to current track |
| `Esc` / `F3` | Close browser |

### Copy/Paste Operations (in Track Grid)

| Key | Action |
|-----|--------|
| `Ctrl+C` | Copy current track (or selection if active) |
| `Ctrl+V` | Paste pattern at cursor position |
| `Ctrl+X` | Cut current track (copy + clear) |
| `Ctrl+S` | Start/clear selection |
| `Ctrl+A` | Select all steps in current track |
| `Alt+A` | Select all tracks at current step |
| `Ctrl+F` | Fill selection with enabled steps |
| `Ctrl+E` | Empty selection (disable all steps) |
| `Delete` | Clear selected steps (or current step) |
| `C` | Clear current track |

## Architecture

```
src/tui/
├── app.rs           # Main application state and event loop
├── audio_bridge.rs  # Communication bridge to audio engine
├── audio_engine.rs  # Real-time audio processing with cpal
├── config.rs        # Configuration management
├── events.rs        # Event handling
├── mod.rs           # Module exports and error types
├── pattern_manager.rs # Pattern storage and management
├── track_bridge.rs  # Track state synchronization
└── ui/
    ├── mod.rs
    ├── sequencer.rs # Sequencer panel logic
    ├── synthesizer.rs # Synthesizer panel logic
    ├── transport.rs # Transport controls
    └── widgets/     # Custom UI widgets
        ├── grid.rs  # Step sequencer grid
        ├── meter.rs # Level meters
        ├── selector.rs # Dropdown selectors
        └── slider.rs # Parameter sliders
```

## Real-Time Audio

The TUI uses a lock-free audio architecture:
- **AudioBridge**: MPSC channels for parameter updates
- **AudioState**: Atomic values for thread-safe real-time access
- **cpal**: Cross-platform audio I/O

Parameter changes have <10ms latency from UI to audio output.

## Default Patterns

The pattern manager comes pre-loaded with 4 default patterns:
- **Four on Floor Kick** - Classic kick pattern (steps 1, 5, 9, 13)
- **Backbeat Snare** - Snare on steps 5, 13
- **Eighth Note Hi-Hat** - Hi-hat on every other step
- **Syncopated Bass** - Syncopated bass line

Access these via `F3` (Pattern Browser) or `Alt+L` (load most recent).

## Tips

1. **Quick Navigation**: Use number keys `1`-`8` to jump directly to sections
2. **Fine Adjustment**: Use `+`/`-` for precise parameter changes
3. **Copy Patterns**: Press `Ctrl+C` on any track to copy all 16 steps
4. **Batch Operations**: Use selection (`Ctrl+S`) to modify multiple steps at once
5. **Visual Feedback**: Watch the status bar for real-time parameter values
