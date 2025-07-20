# TUI Implementation Plan for Rosco Synthesizer Interface

## Executive Summary

This document outlines a comprehensive implementation plan for building a Terminal User Interface (TUI) version of the Rosco synthesizer interface. The TUI will provide real-time control over synthesis parameters, an 8-track step sequencer, and seamless integration with Rosco's existing modular architecture. The implementation prioritizes low latency, cross-platform compatibility, and efficient resource usage while maintaining professional music production workflow capabilities.

## 1. Architecture Overview

### 1.1 Integration with Existing Rosco Modules

The TUI will integrate seamlessly with Rosco's current modular architecture:

```rust
// TUI-specific modules extending existing architecture
src/
├── tui/                          # New TUI module
│   ├── mod.rs                   # TUI module exports
│   ├── app.rs                   # Main application state
│   ├── ui/                      # UI components
│   │   ├── mod.rs
│   │   ├── synthesizer.rs       # Synth controls layout
│   │   ├── sequencer.rs         # 8-track sequencer grid
│   │   ├── transport.rs         # Transport controls
│   │   └── widgets/             # Custom TUI widgets
│   │       ├── mod.rs
│   │       ├── slider.rs        # Parameter sliders
│   │       ├── selector.rs      # Option selectors
│   │       ├── meter.rs         # Level meters
│   │       └── grid.rs          # Step sequencer grid
│   ├── events.rs               # Event handling system
│   ├── audio_bridge.rs         # TUI-Audio thread communication
│   └── config.rs               # TUI configuration
├── audio_gen/                   # Existing audio modules
├── track/                       # Existing track system
├── filter/                      # Existing filter system
├── envelope/                    # Existing envelope system
└── effect/                      # Existing effects system
```

**Key Integration Points:**
- Direct use of existing `Track<SequenceType>` structures
- Leverage current `TrackEffects` and parameter systems
- Utilize existing `Envelope`, `Filter`, and `Effect` modules
- Build upon `audio_gen` module for real-time synthesis

### 1.2 TUI Framework Selection: Ratatui

**Selected Framework:** `ratatui` with `crossterm` backend

**Rationale:**
- Actively maintained and mature TUI framework
- Excellent real-time performance characteristics
- Strong community and extensive widget ecosystem
- Proven integration with audio applications
- Cross-platform terminal compatibility
- Low resource overhead ideal for audio applications

**Dependencies to Add:**
```toml
[dependencies]
# TUI dependencies
ratatui = "0.26"
crossterm = { version = "0.27", features = ["event-stream"] }
tokio = { version = "1.0", features = ["full"] }
tokio-util = "0.7"

# Audio threading and communication
ringbuf = "0.3"
atomic_float = "0.1"
```

### 1.3 Component Organization and Data Flow

```rust
// Main application architecture
pub struct RoscoTuiApp {
    // UI State
    ui_state: UiState,
    current_focus: FocusArea,
    
    // Audio Engine Integration
    audio_bridge: AudioBridge,
    
    // Synthesizer State
    synth_params: SynthParameters,
    
    // Sequencer State
    sequencer: Sequencer,
    tracks: Vec<Track<FixedTimeNoteSequence>>,
    
    // Transport State
    transport: TransportState,
}

pub enum FocusArea {
    Synthesizer(SynthSection),
    Sequencer,
    Transport,
}

pub enum SynthSection {
    Oscillator,
    Filter,
    Envelope,
    Effects,
}
```

**Data Flow Pattern:**
1. **Input Events** → Event Handler → UI State Updates
2. **Parameter Changes** → Audio Bridge → Real-time Audio Thread
3. **Audio Feedback** → UI State → Visual Updates
4. **Sequencer Events** → Track System → Audio Generation

## 2. Implementation Phases

### Phase 1: Basic Synth Controls (Weeks 1-2)
**Goal:** Implement core synthesizer parameter controls with real-time audio integration

**Week 1 Objectives:**
- Set up basic TUI framework with ratatui
- Implement main application loop and event handling
- Create basic layout with synthesizer and sequencer areas
- Integrate with existing `audio_gen` module

**Week 1 Deliverables:**
```rust
// Basic TUI app structure
impl RoscoTuiApp {
    pub fn new() -> Result<Self, TuiError> { /* ... */ }
    pub fn run(&mut self) -> Result<(), TuiError> { /* ... */ }
    fn handle_events(&mut self) -> Result<(), TuiError> { /* ... */ }
    fn update_ui(&mut self, frame: &mut Frame) { /* ... */ }
}

// Basic synthesizer controls
struct OscillatorControls {
    waveform: WaveformSelector,
    frequency: FloatSlider,
    volume: FloatSlider,
}
```

**Week 2 Objectives:**
- Implement oscillator section (waveform, frequency, volume)
- Create real-time parameter update system
- Add basic audio output integration
- Implement keyboard navigation

**Week 2 Deliverables:**
- Functional oscillator controls with real-time audio feedback
- Basic keyboard shortcuts (Tab navigation, arrow keys, Enter)
- Audio parameter updates with <10ms latency

### Phase 2: 8-Track Sequencer Interface (Weeks 3-4)
**Goal:** Build comprehensive step sequencer with track management

**Week 3 Objectives:**
- Design and implement step sequencer grid component
- Create track strip controls (volume, pan, mute, solo)
- Integrate with existing `Track` and sequence structures
- Implement step editing functionality

**Week 3 Deliverables:**
```rust
// Sequencer grid widget
pub struct SequencerGrid {
    tracks: [TrackStrip; 8],
    steps: usize,
    cursor: GridCursor,
    playing_step: Option<usize>,
}

// Track controls
pub struct TrackStrip {
    volume: FloatSlider,
    pan: FloatSlider,
    mute: bool,
    solo: bool,
    track_number: u8,
}
```

**Week 4 Objectives:**
- Implement transport controls (play, stop, tempo)
- Add pattern management (clear, copy, paste)
- Integrate sequencer with audio playback system
- Add visual playback indicators

**Week 4 Deliverables:**
- Fully functional 8-track step sequencer
- Transport controls with tempo adjustment
- Pattern management operations
- Real-time playback with visual feedback

### Phase 3: Effects and Advanced Features (Weeks 5-6)
**Goal:** Complete synthesizer feature set with effects and envelopes

**Week 5 Objectives:**
- Implement filter section (type, cutoff, resonance, mix)
- Add envelope controls (ADSR parameters)
- Create effects section (delay, flanger, LFO)
- Implement per-track effects routing

**Week 5 Deliverables:**
```rust
// Filter controls
pub struct FilterControls {
    filter_type: FilterTypeSelector,
    cutoff: LogSlider,
    resonance: FloatSlider,
    mix: FloatSlider,
}

// ADSR envelope controls
pub struct EnvelopeControls {
    attack: TimeSlider,
    decay: TimeSlider,
    sustain: LevelSlider,
    release: TimeSlider,
}
```

**Week 6 Objectives:**
- Add visual feedback systems (level meters, parameter displays)
- Implement preset management (save/load configurations)
- Add advanced sequencer features (velocity, step length)
- Optimize real-time performance

**Week 6 Deliverables:**
- Complete synthesizer parameter control
- Per-track effects processing
- Visual level meters and parameter feedback
- Preset save/load functionality

### Phase 4: Performance Optimization and Polish (Weeks 7-8)
**Goal:** Optimize performance, add polish features, and comprehensive testing

**Week 7 Objectives:**
- Performance optimization for real-time audio
- Memory usage optimization
- Terminal compatibility testing
- Advanced keyboard shortcuts and workflow features

**Week 8 Objectives:**
- Comprehensive testing across platforms
- Documentation and user guide
- Integration testing with existing Rosco compositions
- Final polish and bug fixes

## 3. Technical Specifications

### 3.1 Screen Layout Design

```
┌─ Rosco Synthesizer v0.1.0 ────────────────────────────────────────────────┐
│ ┌─ OSCILLATOR ──┐ ┌─ FILTER ────────┐ ┌─ ENVELOPE ──┐ ┌─ EFFECTS ─────────┐ │
│ │ Wave: Sine   ▼│ │ Type: LowPass ▼ │ │ A: 0.1 ████ │ │ ┌─ DELAY ────────┐ │
│ │ Freq: ████████│ │ Cut:  ██████████ │ │ D: 0.5 ████ │ │ │ Mix:  ███ 30% │ │
│ │       440 Hz  │ │       8.0 kHz   │ │ S: 0.7 ████ │ │ │ Time: ███ 0.2s│ │
│ │ Vol:  ████████│ │ Res:  ████      │ │ R: 2.0 ████ │ │ └────────────────┘ │
│ │       75%     │ │       0.3       │ │             │ │ ┌─ FLANGER ──────┐ │
│ └───────────────┘ │ Mix:  ████████  │ └─────────────┘ │ │ Mix:  ██  20%  │ │
│                   │       80%       │                 │ │ Rate: ███ 2.5Hz│ │
│                   └─────────────────┘                 │ └────────────────┘ │
├─────────────────────────────────────────────────────────────────────────────┤
│ ┌─ 8-TRACK SEQUENCER ──────────────────────────────────────────────────────┐ │
│ │ 1 │●│ │●│ │●│ │ │●│ │ │●│ │ │●│ │ │  Vol:████ Pan:◄──► │M││S│ │
│ │ 2 │ │●│ │●│ │●│ │ │●│ │ │●│ │ │●│ │  Vol:████ Pan:◄──► │M││S│ │
│ │ 3 │●│●│ │ │ │●│●│ │ │ │●│●│ │ │ │  Vol:████ Pan:◄──► │M││S│ │
│ │ 4 │ │ │ │ │ │ │ │ │ │ │ │ │ │ │ │  Vol:████ Pan:◄──► │M││S│ │
│ │ 5 │ │ │ │ │ │ │ │ │ │ │ │ │ │ │ │  Vol:████ Pan:◄──► │M││S│ │
│ │ 6 │ │ │ │ │ │ │ │ │ │ │ │ │ │ │ │  Vol:████ Pan:◄──► │M││S│ │
│ │ 7 │ │ │ │ │ │ │ │ │ │ │ │ │ │ │ │  Vol:████ Pan:◄──► │M││S│ │
│ │ 8 │ │ │ │ │ │ │ │ │ │ │ │ │ │ │ │  Vol:████ Pan:◄──► │M││S│ │
│ │     1   2   3   4   5   6   7   8   9  10  11  12  13  14  15  16       │ │
│ │ ┌─ TRANSPORT ────────────────────────────────────────────────────────────┐ │ │
│ │ │ ▶ ■ ●   Tempo: 120 BPM   Position: 1.2.1   CPU: 15%   Buffer: ████  │ │ │
│ │ └────────────────────────────────────────────────────────────────────────┘ │ │
│ └───────────────────────────────────────────────────────────────────────────┘ │
│ Ready | 44.1kHz | Audio OK | F1:Help F2:Save F3:Load TAB:Focus ESC:Quit      │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Layout Specifications:**
- **Minimum Terminal Size:** 80x24 characters
- **Optimal Size:** 120x40+ characters
- **Responsive Design:** Adapt to larger terminals with expanded controls
- **Focus Indicators:** Clear visual indication of active control area
- **Status Bar:** Real-time system information and help

### 3.2 Keyboard Navigation and Shortcuts

**Primary Navigation:**
- `Tab` / `Shift+Tab`: Cycle through focus areas
- `Arrow Keys`: Navigate within focus area
- `Enter` / `Space`: Activate control or toggle step
- `Esc`: Return to previous focus level or quit

**Synthesizer Controls:**
- `1-4`: Quick switch between synth sections (Osc/Filter/Env/FX)
- `+/-` or `=/-`: Adjust focused parameter
- `Shift + +/-`: Fine adjustment (0.01 increments)
- `Home/End`: Set parameter to min/max
- `R`: Reset parameter to default

**Sequencer Controls:**
- `A-H`: Quick select tracks 1-8
- `1-9, 0`: Quick jump to steps 1-10
- `Space`: Toggle current step
- `C`: Clear track/pattern
- `X`: Cut pattern
- `V`: Paste pattern
- `Delete`: Clear current step

**Transport Controls:**
- `F5` / `P`: Play/Pause
- `F6` / `S`: Stop
- `F7` / `R`: Record
- `T`: Tap tempo
- `[/]`: Decrease/increase tempo

**Global Shortcuts:**
- `F1`: Help/keyboard shortcuts
- `F2`: Save session
- `F3`: Load session
- `F4`: Toggle full-screen mode
- `Ctrl+Q`: Quit application

### 3.3 Real-Time Audio Parameter Updates

**Communication Architecture:**
```rust
// Lock-free parameter communication
pub struct AudioBridge {
    // Parameter update channel (UI → Audio)
    param_sender: Sender<ParameterUpdate>,
    param_receiver: Receiver<ParameterUpdate>,
    
    // Audio feedback channel (Audio → UI)
    feedback_sender: Sender<AudioFeedback>,
    feedback_receiver: Receiver<AudioFeedback>,
    
    // Shared atomic parameters for high-frequency updates
    oscillator_freq: Arc<AtomicF32>,
    filter_cutoff: Arc<AtomicF32>,
    master_volume: Arc<AtomicF32>,
}

pub enum ParameterUpdate {
    OscillatorFrequency(f32),
    OscillatorVolume(f32),
    OscillatorWaveform(Waveform),
    FilterCutoff(f32),
    FilterResonance(f32),
    FilterType(FilterType),
    EnvelopeAttack(f32),
    EnvelopeDecay(f32),
    EnvelopeSustain(f32),
    EnvelopeRelease(f32),
    SequencerStep { track: u8, step: u8, enabled: bool },
    TransportPlay,
    TransportStop,
    TempoChange(f32),
}

pub enum AudioFeedback {
    LevelMeter { track: u8, level: f32 },
    PlaybackPosition(f32),
    CpuUsage(f32),
    BufferHealth(f32),
}
```

**Performance Requirements:**
- **Parameter Update Latency:** <5ms from UI input to audio processing
- **UI Refresh Rate:** 60 FPS for smooth real-time updates
- **Audio Thread Priority:** Ensure UI operations never block audio processing
- **Buffer Management:** Use ring buffers for real-time safe communication

### 3.4 State Management and Persistence

**Configuration System:**
```rust
#[derive(Serialize, Deserialize)]
pub struct TuiConfig {
    // Display preferences
    pub theme: ColorTheme,
    pub layout: LayoutPreferences,
    
    // Audio settings
    pub audio_device: Option<String>,
    pub sample_rate: u32,
    pub buffer_size: u32,
    
    // Keyboard mappings
    pub key_bindings: HashMap<String, Action>,
    
    // Synthesizer defaults
    pub default_synth_params: SynthParameters,
}

#[derive(Serialize, Deserialize)]
pub struct SessionState {
    pub synth_params: SynthParameters,
    pub tracks: Vec<TrackData>,
    pub transport_state: TransportState,
    pub tempo: f32,
}
```

**File Locations:**
- **Config:** `~/.config/rosco/tui_config.toml`
- **Sessions:** `~/.local/share/rosco/sessions/`
- **Presets:** `~/.local/share/rosco/presets/`

## 4. UI Components Design

### 4.1 Oscillator Section

```rust
pub struct OscillatorControls {
    waveform_selector: WaveformSelector,
    frequency_slider: LogSlider,
    volume_slider: LinearSlider,
    frequency_display: NumericDisplay,
}

impl OscillatorControls {
    fn render(&self, area: Rect, buf: &mut Buffer, focused: bool) {
        // Render waveform selector dropdown
        // Render frequency slider with logarithmic scale
        // Render volume slider with linear scale
        // Show current frequency value in Hz
    }
    
    fn handle_input(&mut self, key: KeyEvent) -> Option<ParameterUpdate> {
        // Handle navigation and parameter adjustment
        // Return parameter updates for audio thread
    }
}

pub struct WaveformSelector {
    options: Vec<Waveform>,
    selected: usize,
    expanded: bool,
}

pub struct LogSlider {
    min: f32,
    max: f32,
    value: f32,
    position: usize,
    width: usize,
}
```

### 4.2 Filter Section

```rust
pub struct FilterControls {
    filter_type: FilterTypeSelector,
    cutoff_slider: LogSlider,
    resonance_slider: LinearSlider,
    mix_slider: LinearSlider,
}

impl FilterControls {
    fn render(&self, area: Rect, buf: &mut Buffer, focused: bool) {
        // Render filter type selector
        // Render cutoff frequency with logarithmic scale (20Hz - 20kHz)
        // Render resonance control (Q factor)
        // Render dry/wet mix control
    }
}

pub enum FilterType {
    LowPass,
    HighPass,
    BandPass,
    Notch,
}
```

### 4.3 Envelope Section

```rust
pub struct EnvelopeControls {
    attack_slider: TimeSlider,
    decay_slider: TimeSlider,
    sustain_slider: LevelSlider,
    release_slider: TimeSlider,
    envelope_display: EnvelopeVisualization,
}

impl EnvelopeControls {
    fn render(&self, area: Rect, buf: &mut Buffer, focused: bool) {
        // Render ADSR parameter sliders
        // Show visual envelope shape using ASCII art
        // Display current values with units (ms, %)
    }
}

pub struct EnvelopeVisualization {
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    width: usize,
    height: usize,
}

impl EnvelopeVisualization {
    fn render_ascii_envelope(&self) -> Vec<String> {
        // Generate ASCII art representation of ADSR envelope
        // Example:
        //     ●──●
        //    ╱    ╲
        //   ╱      ●───●
        //  ╱           ╲
        // ●             ●
    }
}
```

### 4.4 8-Track Sequencer Grid

```rust
pub struct SequencerGrid {
    tracks: [TrackStrip; 8],
    steps_per_track: usize,
    cursor: GridCursor,
    playing_step: Option<usize>,
    selection: Option<GridSelection>,
}

pub struct TrackStrip {
    track_number: u8,
    volume_slider: LinearSlider,
    pan_slider: PanSlider,
    mute_button: Toggle,
    solo_button: Toggle,
    steps: Vec<StepCell>,
}

pub struct StepCell {
    enabled: bool,
    velocity: u8,
    note: Option<Note>,
    highlighted: bool,
}

impl SequencerGrid {
    fn render(&self, area: Rect, buf: &mut Buffer, focused: bool) {
        // Render track numbers and controls
        // Render step grid with current cursor position
        // Highlight playing step during playback
        // Show selection if active
    }
    
    fn handle_input(&mut self, key: KeyEvent) -> Vec<SequencerUpdate> {
        // Handle cursor movement
        // Step toggle/edit operations
        // Track parameter adjustments
        // Selection and copy/paste operations
    }
}

pub struct GridCursor {
    track: u8,
    step: u8,
}

pub struct GridSelection {
    start: GridCursor,
    end: GridCursor,
}
```

### 4.5 Transport Controls

```rust
pub struct TransportPanel {
    play_button: Button,
    stop_button: Button,
    record_button: Button,
    tempo_slider: TempoSlider,
    position_display: PositionDisplay,
    status_indicators: StatusIndicators,
}

impl TransportPanel {
    fn render(&self, area: Rect, buf: &mut Buffer, focused: bool) {
        // Render transport buttons with state indicators
        // Show tempo with BPM value
        // Display playback position (measure.beat.tick)
        // Show system status (CPU, buffer health)
    }
}

pub struct PositionDisplay {
    measure: u32,
    beat: u8,
    tick: u16,
    format: PositionFormat,
}

pub enum PositionFormat {
    MeasureBeatTick,
    TimeMinutesSeconds,
    SamplePosition,
}

pub struct StatusIndicators {
    cpu_usage: f32,
    buffer_health: f32,
    audio_device_status: AudioStatus,
}
```

### 4.6 Status Displays

```rust
pub struct LevelMeter {
    level: f32,
    peak: f32,
    peak_hold_time: Duration,
    orientation: MeterOrientation,
    width: usize,
}

impl LevelMeter {
    fn render_ascii_meter(&self) -> String {
        // Generate ASCII meter representation
        // Example: "████████░░ -12dB"
        let filled_chars = (self.level * self.width as f32) as usize;
        let meter: String = "█".repeat(filled_chars) + &"░".repeat(self.width - filled_chars);
        format!("{} {:+.1}dB", meter, 20.0 * self.level.log10())
    }
}

pub enum MeterOrientation {
    Horizontal,
    Vertical,
}
```

## 5. Development Timeline

### Week-by-Week Breakdown

**Week 1: Foundation Setup**
- [ ] Set up project structure and dependencies
- [ ] Implement basic TUI application framework
- [ ] Create main event loop with crossterm integration
- [ ] Design and implement basic layout system
- [ ] Integrate with existing Rosco audio_gen module
- [ ] Create placeholder UI components

**Week 2: Core Synthesizer Controls**
- [ ] Implement oscillator controls (waveform, frequency, volume)
- [ ] Create real-time parameter update system
- [ ] Add keyboard navigation framework
- [ ] Implement basic audio parameter integration
- [ ] Add visual feedback for parameter changes
- [ ] Create logarithmic and linear slider widgets

**Week 3: Sequencer Foundation**
- [ ] Design and implement step sequencer grid
- [ ] Create track strip controls
- [ ] Implement cursor navigation and step editing
- [ ] Integrate with existing Track structures
- [ ] Add basic pattern storage and management
- [ ] Implement grid selection system

**Week 4: Sequencer Completion**
- [ ] Add transport controls (play, stop, tempo)
- [ ] Implement real-time playback with visual feedback
- [ ] Add pattern management (clear, copy, paste)
- [ ] Create per-track volume and pan controls
- [ ] Implement mute and solo functionality
- [ ] Add tempo adjustment with tap tempo

**Week 5: Advanced Synthesizer Features**
- [ ] Implement filter controls (type, cutoff, resonance, mix)
- [ ] Add ADSR envelope controls with visualization
- [ ] Create effects section (delay, flanger, LFO)
- [ ] Implement per-track effects routing
- [ ] Add filter parameter automation support
- [ ] Create envelope visualization component

**Week 6: Effects and Polish**
- [ ] Complete effects parameter controls
- [ ] Add visual level meters for tracks
- [ ] Implement preset save/load system
- [ ] Add advanced sequencer features (velocity editing)
- [ ] Create status indicators and system monitoring
- [ ] Optimize real-time performance

**Week 7: Performance and Compatibility**
- [ ] Optimize memory usage and CPU performance
- [ ] Test terminal compatibility across platforms
- [ ] Implement advanced keyboard shortcuts
- [ ] Add configuration system and user preferences
- [ ] Create comprehensive error handling
- [ ] Performance profiling and optimization

**Week 8: Testing and Documentation**
- [ ] Comprehensive testing across platforms and terminals
- [ ] Integration testing with existing Rosco compositions
- [ ] Create user documentation and help system
- [ ] Final bug fixes and polish
- [ ] Performance validation and benchmarking
- [ ] Prepare for release

### Dependencies and Dependencies

**External Dependencies:**
- Rust stable 1.70+ for latest async features
- Terminal with 256 color support (recommended)
- Audio system compatible with CPAL (ALSA/PulseAudio/CoreAudio/WASAPI)

**Internal Dependencies:**
- Phase 1 must complete before Phase 2 (sequencer needs basic audio integration)
- Filter controls depend on existing filter module architecture
- Effects integration requires completed parameter update system
- Preset system depends on serialization of all parameter types

**Risk Mitigation:**
- Implement audio bridge early to validate real-time performance
- Create comprehensive test suite for parameter update latency
- Develop fallback UI layouts for smaller terminal sizes
- Plan for graceful degradation on terminals with limited capabilities

### Testing and Validation Milestones

**Performance Benchmarks:**
- Parameter update latency <5ms (measured end-to-end)
- UI refresh rate 60 FPS with full feature set active
- Memory usage <100MB under normal operation
- CPU overhead <10% for UI operations

**Compatibility Testing:**
- Terminal emulators: iTerm2, Terminal.app, Windows Terminal, GNOME Terminal
- Operating systems: macOS, Linux (Ubuntu/Fedora), Windows 10/11
- Screen sizes: 80x24 (minimum) through 200x50+ (large displays)

**Audio Integration Testing:**
- Real-time parameter changes without audio dropouts
- Sequencer timing accuracy (sub-millisecond precision)
- Multiple track playback without performance degradation
- Effects processing integration with existing audio pipeline

**User Experience Testing:**
- Keyboard navigation efficiency measurements
- Workflow speed compared to DAW interfaces
- Learning curve assessment for new users
- Accessibility testing for visual and motor impairments

## 6. Integration Points

### 6.1 Connection to Rosco's audio_gen Module

**Integration Strategy:**
```rust
// Extend existing audio_gen with TUI control
impl audio_gen::AudioEngine {
    pub fn set_tui_bridge(&mut self, bridge: AudioBridge) {
        self.tui_bridge = Some(bridge);
    }
    
    pub fn process_tui_updates(&mut self) {
        while let Ok(update) = self.tui_bridge.param_receiver.try_recv() {
            match update {
                ParameterUpdate::OscillatorFrequency(freq) => {
                    self.oscillator.set_frequency(freq);
                }
                ParameterUpdate::FilterCutoff(cutoff) => {
                    self.filter.set_cutoff(cutoff);
                }
                // ... handle all parameter types
            }
        }
    }
}
```

**Real-Time Safety:**
- Use lock-free ring buffers for parameter updates
- Atomic operations for high-frequency parameter changes
- Separate UI thread from audio callback thread
- Batch parameter updates to minimize audio thread interruption

### 6.2 Track and Sequence Management

**Integration with Existing Track System:**
```rust
// Extend Track struct with TUI-specific functionality
impl<T> Track<T> {
    pub fn get_tui_state(&self) -> TrackUiState {
        TrackUiState {
            volume: self.volume,
            effects: self.effects.clone(),
            sequence_length: self.sequence.len(),
            // ... other UI-relevant state
        }
    }
    
    pub fn update_from_tui(&mut self, update: TrackUpdate) {
        match update {
            TrackUpdate::Volume(vol) => self.volume = vol,
            TrackUpdate::Effects(fx) => self.effects = fx,
            // ... handle all track parameter updates
        }
    }
}
```

**Sequence Integration:**
- Leverage existing `FixedTimeNoteSequence` for step sequencer
- Extend with TUI-specific metadata (selection state, visual indicators)
- Integrate pattern management with existing sequence operations
- Support for real-time sequence modification during playback

### 6.3 DSL Integration for Saving/Loading

**Configuration Format:**
```rust
// Extend existing DSL with TUI session format
#[derive(Serialize, Deserialize)]
pub struct TuiSession {
    version: String,
    synth_params: SynthParameters,
    tracks: Vec<TrackSession>,
    transport: TransportSession,
    created: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct TrackSession {
    track_number: u8,
    volume: f32,
    pan: f32,
    mute: bool,
    solo: bool,
    sequence: Vec<StepData>,
    effects: TrackEffects,
}
```

**File Format Integration:**
- Save TUI sessions as enhanced DSL files
- Support loading existing DSL compositions into TUI
- Export TUI sessions to standard DSL format
- Maintain compatibility with existing composition workflow

**DSL Extensions:**
```dsl
# TUI-specific session format
session tui_session_v1 {
    tempo 120
    
    synth {
        oscillator { waveform sine, frequency 440, volume 0.8 }
        filter { type lowpass, cutoff 8000, resonance 0.3, mix 0.8 }
        envelope { attack 0.1, decay 0.5, sustain 0.7, release 2.0 }
    }
    
    track 1 {
        volume 0.9
        pan 0.0
        steps [1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0]
        effects { delay { mix 0.3, time 0.2 } }
    }
    
    # ... additional tracks
}
```

### 6.4 Performance Integration

**Audio Thread Integration:**
```rust
// Integration with existing cpal audio callback
fn audio_callback(
    data: &mut [f32],
    info: &cpal::OutputCallbackInfo,
    audio_engine: &mut AudioEngine,
) {
    // Process TUI parameter updates
    audio_engine.process_tui_updates();
    
    // Generate audio with updated parameters
    audio_engine.fill_buffer(data);
    
    // Send feedback to TUI
    audio_engine.send_tui_feedback();
}
```

**Threading Architecture:**
- **Main Thread:** TUI rendering and event handling
- **Audio Thread:** Real-time audio processing (existing)
- **Event Thread:** Async event processing for terminal input
- **Communication:** Lock-free channels and atomic shared state

## Conclusion

This implementation plan provides a comprehensive roadmap for building a professional-quality TUI interface for the Rosco synthesizer. The plan prioritizes integration with existing Rosco modules, maintains real-time performance requirements, and delivers a complete 8-track synthesizer and sequencer interface.

The phased approach ensures steady progress with testable milestones, while the detailed technical specifications provide clear implementation guidance. The focus on keyboard-driven workflow and terminal compatibility makes this interface ideal for power users, remote development scenarios, and resource-constrained environments.

The integration points leverage Rosco's existing modular architecture, ensuring that the TUI becomes a natural extension of the current system rather than a separate implementation. This approach minimizes code duplication and maximizes the benefit of Rosco's proven audio processing capabilities.

Upon completion, the TUI will provide a fast, efficient, and powerful interface for music creation that complements Rosco's existing DSL and programmatic composition capabilities.