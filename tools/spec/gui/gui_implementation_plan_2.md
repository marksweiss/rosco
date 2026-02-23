# Rosco GUI Implementation Plan v2

## Revision History

| Version | Date       | Author        | Notes                                            |
|---------|------------|---------------|--------------------------------------------------|
| 2.0     | 2026-02-23 | Claude / Mark | Rewritten against current codebase with TUI prior art |

---

## 1. Executive Summary

### 1.1 Purpose

This document specifies the implementation plan for a native GPU-accelerated GUI for the Rosco music composition toolkit. It supersedes `gui_implementation_plan.md` (v1), which was written against an earlier codebase.

### 1.2 What Changed Since v1

The v1 plan proposed building from scratch. Since then, significant infrastructure has been built:

| Area | v1 Assumed | Current State |
|------|-----------|---------------|
| **UI–Audio bridge** | Proposed | `AudioBridge` with ringbuf + `AtomicF32` exists (`src/tui/audio_bridge.rs`) |
| **Track bridge** | Proposed | `TrackBridge` with `TrackData`/`TrackUpdate` exists (`src/tui/track_bridge.rs`) |
| **Pattern manager** | Proposed | `PatternManager` with `Pattern`/`PatternBank` exists (`src/tui/pattern_manager.rs`) |
| **Synth parameters** | Proposed | `SynthParameters` struct exists (`src/tui/app.rs:209-223`) |
| **Transport state** | Proposed | `TransportState` with `PlaybackPosition` exists (`src/tui/app.rs:225-261`) |
| **Focus/navigation** | Proposed | `FocusArea`/`SynthSection` enums exist (`src/tui/app.rs:148-163`) |
| **Config management** | Proposed | `TuiConfig` with `ColorTheme`/`LayoutPreferences` exists (`src/tui/config.rs`) |
| **Effects count** | 3 (delay, flanger, LFO) | 7: Delay, Flanger, LFO, Tremolo, Vibrato, Chorus, Equalizer |
| **Envelopes** | Basic ADSR | Per-segment `Linear`/`Exponential` curves with steepness (`src/envelope/envelope.rs`) |
| **Oscillators** | Simple | Band-limited wavetables with 6 waveforms and linear interpolation (`src/audio_gen/oscillator.rs`) |
| **Dependencies** | Proposed adding ringbuf, atomic_float, serde, toml, dirs | Already in `Cargo.toml` |

### 1.3 Technology Choices

| Component | Choice | Rationale |
|-----------|--------|-----------|
| **GUI framework** | [egui](https://github.com/emilk/egui) | Immediate-mode, Rust-native, built-in widgets, large ecosystem |
| **Window hosting** | [baseview](https://github.com/RustAudio/baseview) | Audio-focused windowing, designed for plugin UIs, cross-platform |
| **GPU backend** | [wgpu](https://wgpu.rs/) | Modern GPU abstraction, used by egui's renderer |
| **egui integration** | egui-baseview | Bridges egui into baseview windows |
| **Audio I/O** | cpal 0.16.0 | Already in use |
| **Lock-free comms** | ringbuf 0.3 + atomic_float 0.1 | Already in use (TUI `AudioBridge`) |
| **Serialization** | serde + toml + serde_json | Already in use (TUI `TuiConfig`) |

### 1.4 Key Design Principles

1. **Reuse, don't rewrite** — Extract TUI bridge/state code into a shared `src/bridge/` module
2. **Lock-free audio thread** — All UI-to-audio communication via ring buffers and atomics
3. **Real types** — GUI code directly uses Rosco types (`Waveform`, `Envelope`, `TrackEffects`, etc.)
4. **Effects-complete** — Full coverage of all 7 effects with accurate parameter ranges
5. **Incremental migration** — TUI continues to work; GUI is an alternative frontend

---

## 2. Architecture Overview

### 2.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    GUI Layer (egui)                      │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌───────────┐  │
│  │Oscillator│ │ Envelope │ │ Effects  │ │ Sequencer │  │
│  │  Panel   │ │  Graph   │ │   Rack   │ │   Grid    │  │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └─────┬─────┘  │
│       └─────────────┴────────────┴─────────────┘        │
│                         │                                │
│              ┌──────────▼──────────┐                     │
│              │   GuiParameterMap   │                     │
│              └──────────┬──────────┘                     │
└─────────────────────────┼───────────────────────────────┘
                          │ ParameterUpdate enum
              ┌───────────▼───────────┐
              │  src/bridge/ (shared) │
              │  ┌─────────────────┐  │
              │  │  AudioBridge    │  │  ringbuf<ParameterUpdate>
              │  │  TrackBridge    │  │  ringbuf<AudioFeedback>
              │  │  PatternManager │  │  AtomicF32 (freq, cutoff, vol)
              │  └────────┬────────┘  │
              └───────────┼───────────┘
                          │
              ┌───────────▼───────────┐
              │   Audio Thread        │
              │  ┌─────────────────┐  │
              │  │  PlaybackNote   │  │  apply_effects() chain
              │  │  OscillatorTables│ │  Band-limited wavetables
              │  │  TrackEffects   │  │  7 effects + 4 filters
              │  └─────────────────┘  │
              └───────────────────────┘
```

### 2.2 Shared Bridge Module Extraction

The existing TUI code in `src/tui/` contains UI-agnostic infrastructure that both TUI and GUI should share. The plan is to extract into `src/bridge/`:

```
src/bridge/
├── mod.rs                 # Re-exports
├── audio_bridge.rs        # From src/tui/audio_bridge.rs (AudioBridge, ParameterUpdate, AudioFeedback)
├── track_bridge.rs        # From src/tui/track_bridge.rs (TrackBridge, TrackData, TrackUpdate)
├── pattern_manager.rs     # From src/tui/pattern_manager.rs (PatternManager, Pattern, PatternBank)
└── parameter_update.rs    # Extended ParameterUpdate enum (new file)
```

The TUI module (`src/tui/`) will then import from `src/bridge/` instead of owning these types.

### 2.3 Extended ParameterUpdate Enum

The current `ParameterUpdate` enum (`src/tui/audio_bridge.rs:9-23`) has 13 variants covering basic oscillator, filter, envelope, and transport parameters. The GUI requires comprehensive coverage of all 7 effects.

**Current (13 variants):**
```rust
pub enum ParameterUpdate {
    OscillatorFrequency(f32),
    OscillatorVolume(f32),
    OscillatorWaveform(audio_gen::Waveform),
    FilterCutoff(f32),
    FilterResonance(f32),
    EnvelopeAttack(f32),
    EnvelopeDecay(f32),
    EnvelopeSustain(f32),
    EnvelopeRelease(f32),
    SequencerStep { track: u8, step: u8, enabled: bool },
    TransportPlay,
    TransportStop,
    TempoChange(f32),
}
```

**Extended (~50 variants):**
```rust
pub enum ParameterUpdate {
    // === Oscillator (3) ===
    OscillatorFrequency(f32),           // 20.0–20000.0 Hz
    OscillatorVolume(f32),              // 0.0–1.0
    OscillatorWaveform(Waveform),       // Sine, Saw, Square, Triangle, GaussianNoise, Noise

    // === Filter (4) ===
    FilterType(FilterKind),             // LowPass, HighPass, BandPass, Notch
    FilterCutoff(f32),                  // 20.0–20000.0 Hz (default: 1000.0)
    FilterResonance(f32),               // 0.0–20.0 (default: 0.0)
    FilterMix(f32),                     // 0.0–1.0 (default: 1.0)

    // === Envelope (10) ===
    EnvelopeAttackPosition(f32),        // 0.0–1.0 (default: 0.02)
    EnvelopeAttackVolume(f32),          // 0.0–1.0 (default: 1.0)
    EnvelopeDecayPosition(f32),         // 0.0–1.0 (default: 0.51)
    EnvelopeDecayVolume(f32),           // 0.0–1.0 (default: 1.0)
    EnvelopeSustainPosition(f32),       // 0.0–1.0 (default: 0.98)
    EnvelopeSustainVolume(f32),         // 0.0–1.0 (default: 1.0)
    EnvelopeCurve { segment: EnvelopeSegment, curve: EnvelopeCurve },
    EnvelopeSteepness(f32),             // 1.0–20.0 (default: 5.0)
    // Legacy aliases (kept for TUI compatibility)
    EnvelopeAttack(f32),
    EnvelopeDecay(f32),
    EnvelopeSustain(f32),
    EnvelopeRelease(f32),

    // === Delay (5) — src/effect/delay.rs ===
    DelayMix(f32),                      // 0.0–1.0 (default: 1.0)
    DelayDecay(f32),                    // 0.0–1.0 (default: 0.5)
    DelayIntervalMs(f32),               // 1.0–1000.0 (default: 100.0)
    DelayDurationMs(f32),               // 1.0–500.0 (default: 20.0)
    DelayNumRepeats(usize),             // 1–16 (default: 4)

    // === Flanger (5) — src/effect/flanger.rs ===
    FlangerDelayMs(f32),                // 1.0–10.0 (default: 5.0)
    FlangerDepthMs(f32),                // 0.1–10.0 (default: 4.0)
    FlangerRateHz(f32),                 // 0.01–10.0 (default: 0.25)
    FlangerMix(f32),                    // 0.0–1.0 (default: 0.5)
    FlangerFeedback(f32),               // 0.0–0.99 (default: 0.3)

    // === LFO (2) — src/effect/lfo.rs ===
    LfoFrequency(f32),                  // 0.01–NYQUIST (default: SAMPLE_RATE/10.0 = 4410.0)
    LfoAmplitude(f32),                  // 0.0–1.0 (default: 0.5)

    // === Tremolo (2) — src/effect/tremolo.rs ===
    TremoloModFreq(f32),                // 0.1–20.0 Hz (default: 5.0)
    TremoloModDepth(f32),               // 0.0–1.0 (default: 0.5)

    // === Vibrato (3) — src/effect/vibrato.rs ===
    VibratoAvgDelay(f32),               // 0.001–0.020 seconds (default: 0.007)
    VibratoModWidth(f32),               // 0.001–0.010 seconds (default: 0.003)
    VibratoModFreq(f32),                // 0.1–20.0 Hz (default: 5.0)

    // === Chorus (4) — src/effect/chorus.rs ===
    ChorusCount(usize),                 // 1–6 (default: 3)
    ChorusDryGain(f32),                 // 0.0–1.0 (default: 0.7)
    ChorusVoiceGain { voice: usize, gain: f32 },  // 0.0–1.0 per voice (default: 0.4)
    ChorusVoiceDelay { voice: usize, delay: f32 }, // seconds per voice

    // === Equalizer (2) — src/effect/equalizer.rs ===
    EqualizerBandGain { band: usize, gain_db: f32 }, // -12.0–12.0 dB per band
    EqualizerBandFreq { band: usize, freq: f32 },     // Hz per band

    // === Sequencer (1) ===
    SequencerStep { track: u8, step: u8, enabled: bool },

    // === Transport (3) ===
    TransportPlay,
    TransportStop,
    TempoChange(f32),                   // 20.0–300.0 BPM (default: 120.0)

    // === Track (3) ===
    TrackVolume { track: u8, volume: f32 },  // 0.0–1.0 (default: 1.0)
    TrackPan { track: u8, pan: f32 },        // -1.0–1.0 (default: 0.0)
    TrackMute { track: u8, muted: bool },
}
```

### 2.4 Extended AudioFeedback Enum

The current `AudioFeedback` enum (`src/tui/audio_bridge.rs:25-31`) is extended for the GUI's richer visualization needs:

```rust
pub enum AudioFeedback {
    // Existing
    LevelMeter { track: u8, level: f32 },
    PlaybackPosition(f32),
    CpuUsage(f32),
    BufferHealth(f32),

    // New for GUI
    WaveformData { track: u8, samples: Vec<f32> },      // For oscilloscope display
    SpectrumData { track: u8, bins: Vec<f32> },          // For spectrum analyzer
    EnvelopePosition { track: u8, position: f32 },       // For envelope graph cursor
    StepAdvance(usize),                                   // Current step in sequence
}
```

### 2.5 Effects Chain Processing Order

The GUI must respect the exact effects chain order defined in `PlaybackNote::apply_effects()` (`src/note/playback_note.rs:178-249`):

```
1. Envelopes        (note-level, then track-level)     lines 185-190
2. LFOs             (note-level, then track-level)     lines 192-198
3. Tremolos         (note-level, then track-level)     lines 200-205
4. Vibratos         (note-level, then track-level)     lines 207-212
5. Flangers         (note-level, then track-level)     lines 214-220
6. Delays           (note-level, then track-level)     lines 222-228
7. Choruses         (note-level, then track-level)     lines 230-235
8. Filters          (note-level only)                  lines 237-240
9. Equalizers       (note-level, then track-level)     lines 242-247
```

The GUI effects rack must display effects in this order and communicate any reordering as a structural change, not a parameter tweak.

---

## 3. Implementation Phases

### Phase 1: Foundation (Weeks 1–2)

**Goal:** Window opens, renders empty panels, sends/receives parameter updates over the bridge.

#### 3.1.1 Bridge Module Extraction

Extract UI-agnostic code from `src/tui/` into `src/bridge/`:

| Source | Destination | What moves |
|--------|-------------|------------|
| `src/tui/audio_bridge.rs` | `src/bridge/audio_bridge.rs` | `AudioBridge`, `ParameterUpdate`, `AudioFeedback` |
| `src/tui/track_bridge.rs` | `src/bridge/track_bridge.rs` | `TrackBridge`, `TrackData`, `TrackUpdate` |
| `src/tui/pattern_manager.rs` | `src/bridge/pattern_manager.rs` | `PatternManager`, `Pattern`, `PatternBank` |

Update `src/tui/` imports to use `crate::bridge::*`.

#### 3.1.2 New Dependencies

Add to `Cargo.toml`:

```toml
# GUI dependencies
egui = "0.29"
egui-baseview = "0.3"
baseview = "0.1"
egui-wgpu = "0.29"

# Visualization
spectrum-analyzer = "1.5"  # FFT for spectrum display
```

#### 3.1.3 GUI Binary Entry Point

Create `src/bin/gui.rs`:

```rust
use osc::bridge::AudioBridge;
use osc::gui::RoscoGuiApp;

fn main() {
    let audio_bridge = AudioBridge::new().expect("Failed to create audio bridge");
    let app = RoscoGuiApp::new(audio_bridge);
    app.run(); // Opens baseview window, runs egui event loop
}
```

Add to `Cargo.toml`:

```toml
[[bin]]
name = "rosco-gui"
path = "src/bin/gui.rs"
```

#### 3.1.4 GUI Module Structure

```
src/gui/
├── mod.rs              # RoscoGuiApp, module re-exports
├── panels/
│   ├── mod.rs
│   ├── oscillator.rs   # Waveform selector, frequency/volume knobs
│   ├── envelope.rs     # Interactive ADSR graph with curve type selectors
│   ├── effects.rs      # Effects rack with 7 effect sub-panels
│   ├── filter.rs       # Filter type, cutoff, resonance, mix
│   ├── sequencer.rs    # Step sequencer grid
│   ├── transport.rs    # Play/stop/record, tempo, position
│   └── mixer.rs        # Per-track volume, pan, mute, solo
├── widgets/
│   ├── mod.rs
│   ├── knob.rs         # Rotary knob widget (egui custom widget)
│   ├── slider.rs       # Vertical/horizontal sliders
│   ├── waveform.rs     # Oscilloscope/waveform display
│   ├── spectrum.rs     # Spectrum analyzer display
│   └── envelope_graph.rs  # Interactive envelope curve editor
└── theme.rs            # Color theme, extending TUI's ColorTheme concept
```

#### 3.1.5 Deliverables

- [ ] `src/bridge/` module with all shared types
- [ ] `src/tui/` updated to import from `src/bridge/`
- [ ] `src/gui/mod.rs` with `RoscoGuiApp` struct
- [ ] `src/bin/gui.rs` entry point
- [ ] Window opens with placeholder panels
- [ ] `ParameterUpdate` sent from GUI, received by audio thread
- [ ] `AudioFeedback` sent from audio thread, displayed in GUI status bar
- [ ] All existing tests pass (`cargo test`)

### Phase 2: Oscillator + Envelope Panels (Weeks 3–4)

**Goal:** Fully functional oscillator controls and interactive envelope editor.

#### 3.2.1 Oscillator Panel

Renders controls for the oscillator section, mapping to `SynthParameters` (`src/tui/app.rs:208-223`):

| Control | Widget | Maps To | Range |
|---------|--------|---------|-------|
| Waveform | Dropdown/buttons | `Waveform` enum (`src/audio_gen/oscillator.rs:11-19`) | Sine, Saw, Square, Triangle, GaussianNoise, Noise |
| Frequency | Knob (log-scale) | `OscillatorFrequency(f32)` | 20.0–20000.0 Hz |
| Volume | Vertical slider | `OscillatorVolume(f32)` | 0.0–1.0 |

Waveform buttons should show a visual preview of each waveform shape, generated from the band-limited wavetable data (`OscillatorTables` at `src/audio_gen/oscillator.rs:21-27`).

#### 3.2.2 Envelope Graph Panel

Interactive ADSR envelope editor reflecting the `Envelope` struct (`src/envelope/envelope.rs:20-48`):

**Draggable control points:**
- **Attack** — position (x: 0.0–1.0) and volume (y: 0.0–1.0)
- **Decay** — position and volume, constrained: `decay.0 >= attack.0`
- **Sustain** — position and volume, constrained: `sustain.0 >= decay.0`

**Fixed points:**
- **Start** — always (0.0, 0.0)
- **Release** — always (1.0, 0.0)

**Curve type selectors** (per segment):
- Each of the 4 segments (attack, decay, sustain, release) has a `EnvelopeCurve` toggle: `Linear` | `Exponential`
- Source: `src/envelope/envelope.rs:8-12`

**Steepness control:**
- Single knob/slider for `steepness` (default: 5.0, range: 1.0–20.0)
- Source: `src/envelope/envelope.rs:14` (`DEFAULT_EXP_STEEPNESS = 5.0`)
- Only active when at least one segment uses `Exponential` curve

**Visual rendering:**
- Linear segments: straight lines between control points
- Exponential segments: curved lines using the formula from `src/envelope/envelope.rs:146-152`:
  ```
  t = (position - start.0) / (end.0 - start.0)
  exp_t = 1.0 - exp(-steepness * t)
  exp_t_normalized = exp_t / (1.0 - exp(-steepness))
  value = start.1 + (end.1 - start.1) * exp_t_normalized
  ```

**Default envelope values** (from `default_envelope()` at `src/envelope/envelope.rs:88-102`):
- Attack: (0.02, 1.0)
- Decay: (0.51, 1.0)
- Sustain: (0.98, 1.0)
- All curves: Linear
- Steepness: 5.0

#### 3.2.3 Deliverables

- [ ] Waveform selector with visual previews for all 6 waveforms
- [ ] Log-scale frequency knob with real-time value display
- [ ] Volume slider with level meter
- [ ] Interactive envelope graph with draggable ADSR points
- [ ] Per-segment curve type toggle (Linear/Exponential)
- [ ] Steepness knob for exponential curves
- [ ] All parameter changes sent via `ParameterUpdate` to audio thread

### Phase 3: Effects Rack (Weeks 5–7)

**Goal:** Complete effects rack with all 7 effects, matching the processing chain order.

#### 3.3.1 Effects Rack Layout

The effects rack displays effects in the exact processing order from `PlaybackNote::apply_effects()` (`src/note/playback_note.rs:178-249`). Each effect is a collapsible sub-panel with enable/bypass toggle and controls.

#### 3.3.2 Delay Panel

Source: `src/effect/delay.rs`

| Parameter | Control | Default | Range |
|-----------|---------|---------|-------|
| `mix` | Knob | 1.0 | 0.0–1.0 |
| `decay` | Knob | 0.5 | 0.0–1.0 |
| `interval_ms` | Knob | 100.0 | 1.0–1000.0 ms |
| `duration_ms` | Knob | 20.0 | 1.0–500.0 ms |
| `num_repeats` | Stepper | 4 | 1–16 |

Maps to `ParameterUpdate`: `DelayMix`, `DelayDecay`, `DelayIntervalMs`, `DelayDurationMs`, `DelayNumRepeats`

#### 3.3.3 Flanger Panel

Source: `src/effect/flanger.rs`

| Parameter | Control | Default | Range |
|-----------|---------|---------|-------|
| `delay_ms` | Knob | 5.0 | 1.0–10.0 ms |
| `depth_ms` | Knob | 4.0 | 0.1–10.0 ms |
| `rate_hz` | Knob | 0.25 | 0.01–10.0 Hz |
| `mix` | Knob | 0.5 | 0.0–1.0 |
| `feedback` | Knob | 0.3 | 0.0–0.99 |

Maps to `ParameterUpdate`: `FlangerDelayMs`, `FlangerDepthMs`, `FlangerRateHz`, `FlangerMix`, `FlangerFeedback`

#### 3.3.4 LFO Panel

Source: `src/effect/lfo.rs`

| Parameter | Control | Default | Range | Notes |
|-----------|---------|---------|-------|-------|
| `frequency` | Knob (log) | 4410.0 | 0.01–22050.0 Hz | Must be <= Nyquist |
| `amplitude` | Knob | 0.5 | 0.0–1.0 | `DEFAULT_LFO_AMPLITUDE` from `src/common/constants.rs:12` |
| `waveforms` | Multi-select | [Sine] | Sine, Saw, Triangle, GaussianNoise, Noise | **No Square** (validated in builder, `src/effect/lfo.rs`) |

Maps to `ParameterUpdate`: `LfoFrequency`, `LfoAmplitude`

Note: The LFO waveform selector must exclude `Waveform::Square` — this constraint is enforced by `LFOBuilder::waveforms()`.

#### 3.3.5 Tremolo Panel

Source: `src/effect/tremolo.rs`

| Parameter | Control | Default | Range |
|-----------|---------|---------|-------|
| `mod_freq` | Knob | 5.0 | 0.1–20.0 Hz |
| `mod_depth` | Knob | 0.5 | 0.0–1.0 |

Maps to `ParameterUpdate`: `TremoloModFreq`, `TremoloModDepth`

Processing: `gain = 1.0 - mod_depth * (1.0 - lfo_value) / 2.0`

#### 3.3.6 Vibrato Panel

Source: `src/effect/vibrato.rs`

| Parameter | Control | Default | Range | Unit |
|-----------|---------|---------|-------|------|
| `avg_delay` | Knob | 0.007 | 0.001–0.020 | seconds |
| `mod_width` | Knob | 0.003 | 0.001–0.010 | seconds |
| `mod_freq` | Knob | 5.0 | 0.1–20.0 | Hz |

Maps to `ParameterUpdate`: `VibratoAvgDelay`, `VibratoModWidth`, `VibratoModFreq`

#### 3.3.7 Chorus Panel

Source: `src/effect/chorus.rs`

| Parameter | Control | Default | Range |
|-----------|---------|---------|-------|
| `chorus_count` | Stepper | 3 | 1–6 |
| `dry_gain` | Knob | 0.7 | 0.0–1.0 |
| Per-voice `gain` | Knob array | [0.4, 0.4, 0.4] | 0.0–1.0 |
| Per-voice `delay` | Knob array | [0.015, 0.020, 0.030] | 0.001–0.100 s |
| Per-voice `mod_freq` | Knob array | [0.25, 0.33, 0.40] | 0.01–10.0 Hz |
| Per-voice `mod_width` | Knob array | [0.003, 0.004, 0.005] | 0.001–0.020 s |

When `chorus_count` changes, the per-voice control arrays must resize. The builder validates all Vec lengths match (`ChorusBuilder::build()` at `src/effect/chorus.rs`).

Maps to `ParameterUpdate`: `ChorusCount`, `ChorusDryGain`, `ChorusVoiceGain`, `ChorusVoiceDelay`

#### 3.3.8 Equalizer Panel

Source: `src/effect/equalizer.rs`

**Default 8-band graphic EQ** with center frequencies:

| Band | Center Freq (Hz) | Filter Type |
|------|------------------|-------------|
| 0 | 63 | LowShelf |
| 1 | 125 | Peaking |
| 2 | 250 | Peaking |
| 3 | 500 | Peaking |
| 4 | 1000 | Peaking |
| 5 | 2000 | Peaking |
| 6 | 4000 | Peaking |
| 7 | 8000 | HighShelf |

Each band has:
- **Gain slider**: vertical, -12.0 to +12.0 dB (default: 0.0)
- **Frequency label**: shows center frequency

Q factor is fixed at `DEFAULT_Q = 1.414` (from `src/effect/equalizer.rs`).

Maps to `ParameterUpdate`: `EqualizerBandGain`, `EqualizerBandFreq`

Visual: Show frequency response curve above the sliders, computed from the biquad coefficients.

#### 3.3.9 Filter Sub-Panel

Source: `src/filter/low_pass_filter.rs` (and high_pass, band_pass, notch equivalents)

| Parameter | Control | Default | Range |
|-----------|---------|---------|-------|
| Type | Dropdown | LowPass | LowPass, HighPass, BandPass, Notch |
| `cutoff_frequency` | Knob (log) | 1000.0 | 20.0–20000.0 Hz |
| `resonance` | Knob | 0.0 | 0.0–20.0 |
| `mix` | Knob | 1.0 | 0.0–1.0 |

Maps to `ParameterUpdate`: `FilterType`, `FilterCutoff`, `FilterResonance`, `FilterMix`

#### 3.3.10 Deliverables

- [ ] Effects rack with collapsible panels in correct processing order
- [ ] All 7 effects with full parameter controls
- [ ] Filter sub-panel with 4 filter types
- [ ] Per-voice controls for Chorus (dynamic based on `chorus_count`)
- [ ] 8-band graphic EQ with visual frequency response curve
- [ ] Enable/bypass toggle per effect
- [ ] All parameter changes sent via extended `ParameterUpdate`

### Phase 4: Sequencer + Transport (Weeks 7–9)

**Goal:** Step sequencer grid and transport controls, reusing `TrackBridge` and `PatternManager`.

#### 3.4.1 Sequencer Grid

Reuses the data model from `SequencerGrid` (`src/tui/ui/widgets/grid.rs`):

- **8 tracks** with 16 steps per track
- Each step: enabled/disabled, velocity (0.0–1.0), note frequency (from `WesternPitch` enum)
- Track controls: volume (0.0–1.0), pan (-1.0–1.0), mute, solo
- Cursor navigation and selection for copy/paste
- Pattern management via `PatternManager` (`src/tui/pattern_manager.rs`)

**GUI-specific enhancements over TUI:**
- Click to toggle steps (vs keyboard-only in TUI)
- Drag to paint/erase multiple steps
- Right-click context menu for step properties (velocity, note)
- Visual playback cursor with smooth animation
- Zoom in/out on the grid

#### 3.4.2 Mixer Strip

Per-track controls mapping to `TrackData` (`src/tui/track_bridge.rs:12-19`):

| Control | Widget | Maps To | Range |
|---------|--------|---------|-------|
| Volume | Vertical fader | `TrackVolume { track, volume }` | 0.0–1.0 (default: 1.0) |
| Pan | Horizontal knob | `TrackPan { track, pan }` | -1.0–1.0 (default: 0.0) |
| Mute | Toggle button | `TrackMute { track, muted }` | bool |
| Solo | Toggle button | Track-level solo | bool |
| Level meter | LED meter | `AudioFeedback::LevelMeter` | Read-only |

#### 3.4.3 Transport Bar

Reuses `TransportState` (`src/tui/app.rs:225-254`):

| Control | Widget | Maps To |
|---------|--------|---------|
| Play | Toggle button | `TransportPlay` |
| Stop | Button | `TransportStop` |
| Tempo | Knob/input | `TempoChange(f32)` (20.0–300.0 BPM, default: 120.0) |
| Position | Display | `PlaybackPosition { measure, beat, tick }` |
| Step indicator | LED strip | `AudioFeedback::StepAdvance` |

#### 3.4.4 Pattern Browser

Reuses `PatternManager` and `PatternBank` (`src/tui/pattern_manager.rs`):

- Pattern list with name, length, created date
- Save/load patterns (serde + JSON, already supported)
- Import/export pattern banks (`.json` files)
- Drag patterns onto sequencer tracks

#### 3.4.5 Deliverables

- [ ] 8-track, 16-step sequencer grid with click/drag editing
- [ ] Per-track mixer strip (volume, pan, mute, solo, level meter)
- [ ] Transport bar (play, stop, tempo, position)
- [ ] Pattern browser with save/load/import/export
- [ ] Real-time step highlighting during playback
- [ ] All changes communicated via `ParameterUpdate` and `TrackUpdate`

### Phase 5: Polish + Integration (Weeks 9–10)

**Goal:** Visual polish, keyboard shortcuts, persistence, and release readiness.

#### 3.5.1 Visualization Panels

- **Oscilloscope**: Real-time waveform display from `AudioFeedback::WaveformData`
- **Spectrum Analyzer**: FFT display using `spectrum-analyzer` crate, from `AudioFeedback::SpectrumData`
- **Level Meters**: Per-track peak/RMS meters from `AudioFeedback::LevelMeter`

#### 3.5.2 Theme System

Extend the TUI's `ColorTheme` concept (`src/tui/config.rs:25-31`) into a full GUI theme:

```rust
pub struct GuiTheme {
    // Inherited from TUI concept
    pub name: String,
    pub focused_border: [f32; 4],    // RGBA
    pub unfocused_border: [f32; 4],
    pub highlight: [f32; 4],
    pub background: [f32; 4],

    // GUI-specific
    pub panel_bg: [f32; 4],
    pub knob_track: [f32; 4],
    pub knob_value: [f32; 4],
    pub meter_green: [f32; 4],
    pub meter_yellow: [f32; 4],
    pub meter_red: [f32; 4],
    pub grid_enabled: [f32; 4],
    pub grid_disabled: [f32; 4],
    pub grid_playing: [f32; 4],
    pub envelope_line: [f32; 4],
    pub envelope_point: [f32; 4],
    pub eq_curve: [f32; 4],
    pub text_primary: [f32; 4],
    pub text_secondary: [f32; 4],
}
```

Ship with two built-in themes: **Dark** (default) and **Light**.

#### 3.5.3 Keyboard Shortcuts

Map keyboard shortcuts consistent with the TUI where applicable:

| Action | Shortcut | TUI Equivalent |
|--------|----------|----------------|
| Play/Pause | Space | Space |
| Stop | Escape | Escape |
| Toggle step | Enter | Enter |
| Navigate panels | Tab/Shift+Tab | Tab/Shift+Tab |
| Undo | Ctrl+Z | N/A (new) |
| Redo | Ctrl+Shift+Z | N/A (new) |
| Save preset | Ctrl+S | N/A (new) |
| Load preset | Ctrl+O | N/A (new) |

#### 3.5.4 Persistence

Extend `TuiConfig` (`src/tui/config.rs`) into a shared `AppConfig`:

- **Session state**: last synth params, tempo, transport state (already in `SessionState` at `src/tui/config.rs:132-136`)
- **Window geometry**: position, size (GUI-specific)
- **Theme selection**: saved to `~/.config/rosco/gui_config.toml` (using `dirs` crate, already a dependency)
- **Recent files**: last opened DSL scripts
- **Effect presets**: named parameter sets for each effect

#### 3.5.5 DSL Integration

The GUI should be able to:
1. **Load** `.dsl` scripts and populate the sequencer/effects from the parsed `Vec<Track>`
2. **Export** the current GUI state as a `.dsl` script
3. **Live edit** — change parameters while a DSL composition plays

This leverages the existing DSL parser in `src/dsl/` which produces `Vec<Track<FixedTimeNoteSequence>>`.

#### 3.5.6 Deliverables

- [ ] Oscilloscope and spectrum analyzer panels
- [ ] Per-track level meters
- [ ] Dark and light themes
- [ ] Full keyboard shortcut system
- [ ] Config persistence (session state, window geometry, theme)
- [ ] DSL script load/export
- [ ] Effect preset save/load

---

## 4. Technical Specifications

### 4.1 Audio Constants

From `src/common/constants.rs`:

| Constant | Value | Location |
|----------|-------|----------|
| `SAMPLE_RATE` | 44100.0 Hz | `src/common/constants.rs:7` |
| `SAMPLES_PER_MS` | 44.1 | `src/common/constants.rs:8` |
| `NYQUIST_FREQUENCY` | 22050.0 Hz | `src/common/constants.rs:10` |
| `DEFAULT_LFO_AMPLITUDE` | 0.5 | `src/common/constants.rs:12` |

### 4.2 Wavetable Specifications

From `src/audio_gen/oscillator.rs`:

| Parameter | Value | Location |
|-----------|-------|----------|
| Table size | 1024 samples | `oscillator.rs:8` |
| Interpolation | Linear (between adjacent entries) | `oscillator.rs:118-125` |
| Max harmonics | `SAMPLE_RATE / 2.0 / 20.0` = 1102 | `oscillator.rs:52` |
| Band-limiting | Additive synthesis (harmonic series) | `oscillator.rs:49-116` |

### 4.3 Ring Buffer Sizing

From `src/tui/audio_bridge.rs:52-57`:

| Buffer | Size | Direction |
|--------|------|-----------|
| Parameter updates | 1024 entries | UI → Audio |
| Audio feedback | 1024 entries | Audio → UI |

The GUI may need larger buffers for the extended `ParameterUpdate` enum. Recommend 2048 for the parameter buffer.

### 4.4 Atomic Parameters

From `src/tui/audio_bridge.rs:60-62`:

| Atomic | Default | Used For |
|--------|---------|----------|
| `oscillator_freq` | 440.0 Hz | High-frequency knob updates |
| `filter_cutoff` | 8000.0 Hz | High-frequency knob updates |
| `master_volume` | 0.75 | High-frequency slider updates |

Additional atomics for the GUI:

| Atomic | Default | Used For |
|--------|---------|----------|
| `filter_resonance` | 0.0 | High-frequency knob updates |
| `delay_mix` | 1.0 | Real-time delay wet/dry |
| `flanger_rate` | 0.25 | Real-time flanger speed |

### 4.5 Window Specifications

| Property | Value |
|----------|-------|
| Minimum size | 1200 x 800 px |
| Default size | 1440 x 900 px |
| Resizable | Yes |
| Frame rate | 60 fps (egui repaint on change) |
| DPI awareness | Yes (egui handles scaling) |

---

## 5. UI Components Design

### 5.1 Window Layout

```
┌─────────────────────────────────────────────────────────────────┐
│ Menu Bar   [File] [Edit] [View] [Presets] [Help]                │
├────────────────────────┬────────────────────────────────────────┤
│   Oscillator Panel     │        Envelope Graph                  │
│  ┌─────────────────┐   │   1.0 ─┬──●attack──●decay──●sustain   │
│  │ [Sin][Saw][Sq]  │   │        │ /         \       |     \    │
│  │ [Tri][Nse]      │   │        │/           \      |      \   │
│  │                 │   │   0.0 ─┴─────────────────────────────  │
│  │  Freq: 440 Hz   │   │        0.0                       1.0  │
│  │  Vol:  0.75      │   │   Curves: [Lin|Exp] per segment       │
│  └─────────────────┘   │   Steepness: ●──────── 5.0            │
├────────────────────────┴────────────────────────────────────────┤
│   Effects Rack (processing order)                               │
│  ┌─────────┐┌─────────┐┌─────────┐┌─────────┐┌──────────────┐  │
│  │  LFO    ││ Tremolo ││ Vibrato ││ Flanger ││   Delay      │  │
│  │ freq    ││ freq    ││ delay   ││ delay   ││ mix  decay   │  │
│  │ amp     ││ depth   ││ width   ││ depth   ││ intv dur rpt │  │
│  └─────────┘└─────────┘└─────────┘└─────────┘└──────────────┘  │
│  ┌──────────────┐┌───────────────────────────┐┌──────────────┐  │
│  │   Chorus     ││       Equalizer           ││   Filter     │  │
│  │ voices: 3    ││  │ │ │ │ │ │ │ │          ││ type: LPF    │  │
│  │ dry gain     ││  63 125 250 500 1k 2k 4k 8k││ cutoff res  │  │
│  └──────────────┘└───────────────────────────┘└──────────────┘  │
├─────────────────────────────────────────────────────────────────┤
│   Sequencer Grid                     │ Mixer                    │
│   T1: ● ● · ● · · ● · ● ● · ● · · ● · │ V  P  M S            │
│   T2: · ● ● · ● · · ● · ● ● · ● · · ● │ ▊  ○  □ □            │
│   T3: ● · · ● · ● · · ● · · ● · ● · · │ ▊  ○  □ □            │
│   ...                                    │ ...                   │
├───────────────────┬─────────────────────────────────────────────┤
│ Transport         │ Visualizations                              │
│ [▶][■] ♩=120 BPM │ [Oscilloscope]        [Spectrum]            │
│ 001:01:000        │ ∼∼∼∼∼∼∼∼∼∼∼∼         ▊▊▊▊▊▊▊▊             │
└───────────────────┴─────────────────────────────────────────────┘
```

### 5.2 Custom Widgets

#### 5.2.1 Rotary Knob

A custom egui widget for parameter control:
- Circular track with value arc
- Mouse drag (vertical) to adjust
- Double-click to type exact value
- Right-click for parameter reset
- Tooltip shows current value with units
- Log-scale option for frequency parameters

#### 5.2.2 Envelope Graph Widget

Interactive bezier-based envelope display:
- Canvas size adapts to panel
- Control points drawn as filled circles (6px radius)
- Drag points to modify envelope parameters
- Per-segment curve rendering (linear = straight line, exponential = curve)
- Ghost line showing previous state during drag
- Grid overlay for position/volume reference

#### 5.2.3 Spectrum Analyzer Widget

Real-time FFT display:
- 512 or 1024 bin FFT (matching wavetable size)
- Logarithmic frequency axis (20 Hz – 20 kHz)
- dB scale vertical axis (-60 dB to 0 dB)
- Peak hold with decay
- Frequency labels at octave intervals matching EQ bands

### 5.3 Interaction Patterns

| Interaction | Behavior |
|-------------|----------|
| Knob drag | Vertical mouse movement adjusts value |
| Knob double-click | Opens text input for exact value |
| Knob right-click | Resets to default value |
| Knob scroll | Fine adjustment (1% of range per tick) |
| Knob Shift+drag | Fine control (10x precision) |
| Slider drag | Horizontal/vertical depending on orientation |
| Step click | Toggle enabled/disabled |
| Step right-click | Open note/velocity editor |
| Step drag | Paint mode (enable/disable sequence of steps) |
| Panel Tab | Cycle focus between panels |
| Ctrl+Z | Undo last parameter change |

---

## 6. Visual Design

### 6.1 Dark Theme (Default)

```rust
pub fn dark_theme() -> GuiTheme {
    GuiTheme {
        name: "Dark".to_string(),
        focused_border: [0.0, 0.8, 0.8, 1.0],      // Cyan (matching TUI)
        unfocused_border: [0.3, 0.3, 0.3, 1.0],     // Dark gray
        highlight: [1.0, 0.85, 0.0, 1.0],            // Gold
        background: [0.1, 0.1, 0.12, 1.0],           // Near-black

        panel_bg: [0.15, 0.15, 0.17, 1.0],
        knob_track: [0.25, 0.25, 0.28, 1.0],
        knob_value: [0.0, 0.7, 0.9, 1.0],            // Cyan
        meter_green: [0.0, 0.8, 0.2, 1.0],
        meter_yellow: [1.0, 0.85, 0.0, 1.0],
        meter_red: [1.0, 0.2, 0.1, 1.0],
        grid_enabled: [0.0, 0.9, 0.4, 1.0],          // Green
        grid_disabled: [0.3, 0.3, 0.35, 1.0],
        grid_playing: [1.0, 1.0, 0.0, 1.0],          // Yellow
        envelope_line: [0.0, 0.8, 0.8, 1.0],         // Cyan
        envelope_point: [1.0, 1.0, 1.0, 1.0],        // White
        eq_curve: [0.4, 0.8, 1.0, 1.0],              // Light blue
        text_primary: [0.9, 0.9, 0.92, 1.0],
        text_secondary: [0.5, 0.5, 0.55, 1.0],
    }
}
```

### 6.2 Responsive Layout

The window layout uses a constraint-based system similar to the TUI's `LayoutPreferences` (`src/tui/config.rs:34-39`):

| Region | Min Height | Default % | Collapse Behavior |
|--------|-----------|-----------|-------------------|
| Oscillator + Envelope | 150 px | 25% | Envelope hides first |
| Effects Rack | 120 px | 20% | Collapse to single row |
| Sequencer + Mixer | 200 px | 35% | Reduce visible tracks |
| Transport + Viz | 80 px | 20% | Viz hides, transport always visible |

Panels can be resized by dragging splitter bars. The layout is persisted in config.

---

## 7. Development Timeline

### Week-by-Week Schedule

| Week | Phase | Deliverables | Risk |
|------|-------|-------------|------|
| 1 | Foundation | Bridge extraction, `src/bridge/` module, TUI imports updated | Low — mechanical refactor |
| 2 | Foundation | egui + baseview window, empty panels, parameter bridge connected | Medium — baseview integration |
| 3 | Oscillator | Waveform selector, frequency knob, volume slider, knob widget | Low |
| 4 | Envelope | Interactive ADSR graph, curve selectors, steepness control | Medium — custom widget |
| 5 | Effects | Delay, Flanger, LFO panels with all parameters | Low |
| 6 | Effects | Tremolo, Vibrato, Chorus panels; per-voice controls | Medium — dynamic UI |
| 7 | Effects + Sequencer | Equalizer, Filter panels; sequencer grid basics | Medium |
| 8 | Sequencer | Full sequencer with click/drag editing, pattern manager | Medium |
| 9 | Transport + Mixer | Transport bar, mixer strip, level meters, visualizations | Low |
| 10 | Polish | Themes, keyboard shortcuts, persistence, DSL integration, testing | Low |

### Milestones

| Milestone | Week | Criteria |
|-----------|------|----------|
| **M1: Window** | 2 | GUI window opens, sends ParameterUpdate, receives AudioFeedback |
| **M2: Sound** | 4 | Can change oscillator waveform/freq and hear changes in real-time |
| **M3: Effects** | 7 | All 7 effects controllable from GUI |
| **M4: Sequencer** | 8 | Can create and play back a pattern from the GUI |
| **M5: Release** | 10 | Feature-complete, themed, with keyboard shortcuts and persistence |

---

## 8. Integration Points

### 8.1 Shared Bridge Modules

After extraction, both TUI and GUI import from `src/bridge/`:

```rust
// In src/tui/app.rs
use crate::bridge::{AudioBridge, ParameterUpdate, AudioFeedback};
use crate::bridge::{TrackBridge, TrackData, TrackUpdate};
use crate::bridge::{PatternManager, Pattern, PatternBank};

// In src/gui/mod.rs
use crate::bridge::{AudioBridge, ParameterUpdate, AudioFeedback};
use crate::bridge::{TrackBridge, TrackData, TrackUpdate};
use crate::bridge::{PatternManager, Pattern, PatternBank};
```

### 8.2 New Cargo Dependencies

```toml
# GUI dependencies (add to Cargo.toml)
egui = "0.29"
egui-baseview = "0.3"
baseview = "0.1"
egui-wgpu = "0.29"
spectrum-analyzer = "1.5"
```

These should be behind a `gui` feature flag to avoid pulling them in for TUI-only or headless builds:

```toml
[features]
default = ["tui"]
tui = ["ratatui", "crossterm"]
gui = ["egui", "egui-baseview", "baseview", "egui-wgpu", "spectrum-analyzer"]
```

### 8.3 Binary Targets

```toml
[[bin]]
name = "rosco"          # Headless composition runner (existing)
path = "src/main.rs"

[[bin]]
name = "rosco-tui"      # Terminal UI (existing)
path = "src/bin/tui.rs"

[[bin]]
name = "rosco-gui"      # Graphical UI (new)
path = "src/bin/gui.rs"
```

### 8.4 File Formats

| Format | Extension | Use | Serialization |
|--------|-----------|-----|---------------|
| DSL scripts | `.dsl` | Composition source | Custom parser (`src/dsl/`) |
| GUI config | `.toml` | App settings | serde + toml |
| Patterns | `.json` | Pattern bank export | serde_json |
| Effect presets | `.toml` | Named effect settings | serde + toml |
| WAV export | `.wav` | Audio file output | hound |

### 8.5 Audio Thread Contract

The audio thread must:
1. Drain the `ParameterUpdate` ring buffer every audio callback
2. Apply parameter changes to the corresponding effect/oscillator/envelope structs
3. Push `AudioFeedback` (level meters, position, waveform data) at 30–60 Hz
4. Never block — all communication is lock-free

The GUI thread must:
1. Send `ParameterUpdate` messages for every knob/slider change
2. Drain `AudioFeedback` ring buffer every frame (60 fps)
3. Update visualization displays from feedback data
4. Never access audio types directly — always go through the bridge

### 8.6 Future: Plugin Architecture

The bridge-based architecture enables a future plugin host (VST3/CLAP):

```
┌─────────┐     ┌──────────┐     ┌──────────┐
│ GUI/TUI │────▶│ Bridge   │────▶│ Audio    │
└─────────┘     │ (shared) │     │ Engine   │
                └──────────┘     └──────────┘
                     ▲
┌─────────┐          │
│ Plugin  │──────────┘
│ Host    │
└─────────┘
```

This is out of scope for v2 but the architecture supports it.

---

## 9. Testing Strategy

### 9.1 Unit Tests

- All new `src/bridge/` types: serialization round-trips, parameter range validation
- `ParameterUpdate` enum: ensure every variant can be sent through ring buffer
- `GuiTheme`: color value validation

### 9.2 Integration Tests

- Bridge extraction: run `cargo test` to verify TUI still works after refactor
- Parameter flow: send `ParameterUpdate` → verify audio engine receives it
- Audio feedback: verify `AudioFeedback` flows from audio thread to GUI

### 9.3 Manual Testing

- Each effect panel: verify all knobs send correct `ParameterUpdate` values
- Envelope graph: verify dragging points updates envelope correctly
- Sequencer: verify step toggling, pattern save/load
- Cross-reference with TUI: same parameters should produce same audio

### 9.4 Performance Targets

| Metric | Target |
|--------|--------|
| GUI frame time | < 16ms (60 fps) |
| Parameter latency | < 5ms (bridge round-trip) |
| Audio callback | < 2ms (at 44100 Hz, 512 sample buffer) |
| Memory | < 100 MB resident |
| CPU (idle) | < 5% |

---

## 10. Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| egui-baseview compatibility | Medium | High | Pin versions; fallback to egui-winit if needed |
| Ring buffer overflow | Low | Medium | Increase buffer to 2048; add overflow counter in `AudioFeedback` |
| Envelope graph interactivity | Medium | Low | Start with simple drag; add snapping/constraints later |
| Chorus dynamic voice count | Low | Medium | Cap at 6 voices; pre-allocate GUI controls |
| Theme consistency TUI↔GUI | Low | Low | Shared `ColorTheme` base; document color mappings |
| Large `ParameterUpdate` enum | Low | Low | Enum variants are small (max ~12 bytes); ring buffer handles it |

---

## Appendix A: Complete Type Reference

### A.1 Waveform Enum

Source: `src/audio_gen/oscillator.rs:11-19`

```rust
pub enum Waveform {
    GaussianNoise,
    Saw,
    Sine,
    Square,
    Triangle,
    Noise,
}
```

### A.2 EnvelopeCurve Enum

Source: `src/envelope/envelope.rs:8-12`

```rust
pub(crate) enum EnvelopeCurve {
    Linear,
    Exponential,
}
```

### A.3 TrackEffects Struct

Source: `src/track/track_effects.rs`

```rust
pub(crate) struct TrackEffects {
    pub(crate) envelopes: Vec<Envelope>,
    pub(crate) lfos: Vec<LFO>,
    pub(crate) flangers: Vec<Flanger>,
    pub(crate) delays: Vec<Delay>,
    pub(crate) tremolos: Vec<Tremolo>,
    pub(crate) vibratos: Vec<Vibrato>,
    pub(crate) choruses: Vec<Chorus>,
    pub(crate) equalizers: Vec<Equalizer>,
    pub(crate) panning: f32,       // -1.0 to 1.0
    pub(crate) num_channels: i8,   // 1 or 2
}
```

### A.4 Default Effect Values Quick Reference

| Effect | Parameter | Default |
|--------|-----------|---------|
| **Delay** | mix | 1.0 |
| | decay | 0.5 |
| | interval_ms | 100.0 |
| | duration_ms | 20.0 |
| | num_repeats | 4 |
| **Flanger** | delay_ms | 5.0 |
| | depth_ms | 4.0 |
| | rate_hz | 0.25 |
| | mix | 0.5 |
| | feedback | 0.3 |
| **LFO** | frequency | 4410.0 (SAMPLE_RATE/10) |
| | amplitude | 0.5 |
| **Tremolo** | mod_freq | 5.0 |
| | mod_depth | 0.5 |
| **Vibrato** | avg_delay | 0.007 s |
| | mod_width | 0.003 s |
| | mod_freq | 5.0 Hz |
| **Chorus** | chorus_count | 3 |
| | dry_gain | 0.7 |
| | voice gains | [0.4, 0.4, 0.4] |
| | voice delays | [0.015, 0.020, 0.030] s |
| | voice mod_freqs | [0.25, 0.33, 0.40] Hz |
| | voice mod_widths | [0.003, 0.004, 0.005] s |
| **Equalizer** | bands | 8 |
| | gains | [0.0; 8] dB |
| | center_freqs | [63, 125, 250, 500, 1k, 2k, 4k, 8k] Hz |
| | Q | 1.414 |
| **Filter** | cutoff | 1000.0 Hz |
| | resonance | 0.0 |
| | mix | 1.0 |

### A.5 Envelope Defaults

Source: `src/envelope/envelope.rs:88-102`

| Point | Position | Volume |
|-------|----------|--------|
| Start | 0.0 | 0.0 |
| Attack | 0.02 | 1.0 |
| Decay | 0.51 | 1.0 |
| Sustain | 0.98 | 1.0 |
| Release | 1.0 | 0.0 |

All curves: Linear. Steepness: 5.0.
