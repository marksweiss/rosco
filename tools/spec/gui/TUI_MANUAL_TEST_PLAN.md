# TUI Manual Test Plan - Phases 1-3
## Rosco Synthesizer Terminal User Interface

### Document Overview
This manual test plan covers testing for the first three phases of the Rosco TUI implementation:
- **Phase 1**: Basic Synth Controls (Oscillator section)
- **Phase 2**: 8-Track Sequencer Interface  
- **Phase 3**: Effects and Advanced Features (Filter, Envelope sections)

### Prerequisites
- Rosco TUI binary built with `cargo build`
- Terminal with minimum 80x24 characters (120x40+ recommended)
- Audio output device configured
- Testing performed on target platforms (macOS, Linux, Windows)

---

## PHASE 1: Basic Synth Controls Testing

### Setup and Initialization Tests

#### T1.1: Application Startup
**Objective**: Verify TUI starts correctly and displays initial interface
**Steps**:
1. Run `cargo run --bin tui` (or equivalent TUI executable)
2. Verify terminal switches to alternate screen
3. Check initial layout displays:
   - SYNTHESIZER section (top 40%)
   - 8-TRACK SEQUENCER section (middle 55%) 
   - Status bar (bottom 3%)
4. Confirm focus starts on Oscillator section
5. Verify status bar shows: "Ready | OSC:Waveform | 1-4:Sections +/-:Adjust R:Reset F1:Help ESC:Quit"

**Expected Results**: 
- Clean layout render with no visual artifacts
- Cursor positioned on Oscillator waveform control
- Status information accurate

#### T1.2: Help System
**Objective**: Test help overlay functionality
**Steps**:
1. Press `F1` to open help
2. Verify help overlay covers entire screen
3. Read through help content for accuracy
4. Press `F1` again to close help
5. Confirm return to normal interface

**Expected Results**:
- Help text displays correctly with proper formatting
- Help can be toggled on/off
- Interface state preserved when returning from help

### Oscillator Section Tests

#### T1.3: Waveform Selection
**Objective**: Test waveform parameter control
**Steps**:
1. Ensure focus on Oscillator → Waveform (shows "◄" indicator)
2. Press `Left Arrow` to cycle through waveforms
3. Press `Right Arrow` to cycle forward through waveforms  
4. Test each waveform: Sine, Square, Triangle, Sawtooth, Noise
5. Verify current waveform displays correctly (e.g., "Wave: Sine ◄")
6. Press `Enter` to expand/collapse waveform selector (if implemented)

**Expected Results**:
- Waveform changes reflected in display immediately
- All waveform types accessible via navigation
- Visual focus indicator shows current selection
- Parameter updates sent to audio bridge

#### T1.4: Frequency Control  
**Objective**: Test frequency parameter adjustment
**Steps**:
1. Press `Down Arrow` or `Tab` to focus frequency control
2. Verify display shows "Freq: ████████ 440.0 Hz ◄"
3. Press `Right Arrow` to increase frequency
4. Press `Left Arrow` to decrease frequency
5. Test logarithmic scaling behavior (should change by ~5% per step)
6. Press `+` key for fine adjustment
7. Press `-` key for fine adjustment
8. Verify frequency range (20 Hz - 20 kHz)
9. Press `r` to reset to default (440 Hz)

**Expected Results**:
- Frequency adjusts logarithmically with arrow keys
- Visual bar representation updates in real-time
- Numeric display shows accurate frequency values
k Reset function returns to 440 Hz
- Status message confirms "Frequency reset to 440 Hz"

#### T1.5: Volume Control
**Objective**: Test volume parameter adjustment  
**Steps**:
1. Press `Down Arrow` to focus volume control
2. Verify display shows "Vol: ████████ 75% ◄"
3. Press `Right Arrow` to increase volume
4. Press `Left Arrow` to decrease volume
5. Test linear scaling (should change by 5% per step)
6. Verify volume range (0% - 100%)
7. Test at minimum and maximum values
8. Press `r` to reset to default (75%)

**Expected Results**:
- Volume adjusts linearly with arrow keys
- Visual bar representation correlates with percentage
- No audio clipping at maximum volume
- Reset function returns to 75%
- Status message confirms "Volume reset to 75%"

### Audio Integration Tests

#### T1.6: Real-Time Audio Response
**Objective**: Verify audio output responds to parameter changes
**Steps**:
1. Ensure audio output device active
2. Set volume to audible level (~50%)
3. Set frequency to 440 Hz, waveform to Sine
4. Listen for clean sine wave output
5. Change waveform to Square - verify audio changes
6. Adjust frequency - verify pitch changes in real-time  
7. Adjust volume - verify amplitude changes in real-time
8. Test latency: parameter change to audio change should be <10ms

**Expected Results**:
- Audio output matches visual parameter settings
- Parameter changes reflected in audio with minimal latency
- No audio dropouts or glitches during parameter adjustment
- Clean waveforms without distortion at moderate volumes

#### T1.7: Parameter Update Performance
**Objective**: Test real-time parameter update system
**Steps**:
1. Hold down `Right Arrow` on frequency control
2. Observe smooth frequency sweeps
3. Hold down `Right Arrow` on volume control  
4. Observe smooth volume changes
5. Rapidly toggle between waveforms
6. Monitor for any UI lag or audio artifacts

**Expected Results**:
- Smooth parameter sweeps without stepping artifacts
- UI remains responsive during continuous parameter changes
- Audio bridge handles rapid parameter updates without buffer overruns
- Status messages update appropriately

### Navigation and Focus Tests

#### T1.8: Section Navigation
**Objective**: Test navigation between synthesizer sections
**Steps**:
1. Press `1` - verify focus moves to Oscillator section
2. Press `2` - verify focus moves to Filter section (placeholder)
3. Press `3` - verify focus moves to Envelope section (placeholder)
4. Press `4` - verify focus moves to Effects section (placeholder)
5. Press `Tab` to cycle through focus areas
6. Test `Shift+Tab` for reverse cycling

**Expected Results**:
- Number keys provide quick access to synthesizer sections
- Tab navigation cycles: Osc→Filter→Env→FX→Sequencer→Transport→Osc
- Focus indicators update correctly ([FOCUSED] in section titles)
- Status bar reflects current focus area

#### T1.9: Control Navigation
**Objective**: Test navigation within oscillator section
**Steps**:
1. Focus on Oscillator section
2. Use `Up/Down` arrows to navigate between waveform, frequency, volume
3. Verify focus indicator "◄" moves correctly
4. Test wraparound (Volume → Waveform, Waveform → Volume)

**Expected Results**:
- Arrow key navigation works correctly within section
- Focus indicator shows current control
- Navigation wraps around at boundaries

---

## PHASE 2: 8-Track Sequencer Interface Testing

### Sequencer Grid Tests

#### T2.1: Sequencer Focus and Layout
**Objective**: Test sequencer section display and focus
**Steps**:
1. Press `Tab` to navigate to Sequencer section
2. Verify "8-TRACK SEQUENCER [FOCUSED]" title
3. Confirm grid layout shows:
   - 8 tracks (numbered 1-8)
   - 16 steps per track
   - Track controls (volume, pan, mute, solo)
   - Transport controls at bottom
4. Verify cursor positioning (should start at track 1, step 1)

**Expected Results**:
- Clean sequencer grid layout
- Track numbers and step indicators visible
- Cursor position clearly indicated
- All 8 tracks visible simultaneously

#### T2.2: Step Editing
**Objective**: Test basic step sequencer functionality
**Steps**:
1. Navigate to step grid area
2. Press `Space` or `Enter` to toggle current step
3. Verify step toggles between enabled (●) and disabled (│)
4. Navigate with arrow keys to different steps
5. Toggle several steps to create a pattern
6. Navigate to different tracks and repeat

**Expected Results**:
- Steps toggle correctly between enabled/disabled states
- Visual representation updates immediately
- Cursor movement accurate across grid
- Pattern changes reflected in step display

#### T2.3: Grid Navigation
**Objective**: Test cursor movement and navigation
**Steps**:
1. Use `Arrow Keys` to move cursor around grid
2. Test boundary conditions:
   - Top row → wraps to bottom
   - Bottom row → wraps to top  
   - Leftmost step → wraps to rightmost
   - Rightmost step → wraps to leftmost
3. Test quick navigation:
   - `A-H` keys to jump to tracks 1-8
   - `1-9, 0` keys to jump to steps 1-10

**Expected Results**:
- Smooth cursor movement in all directions
- Proper boundary wrapping behavior
- Quick navigation shortcuts work correctly
- Cursor position always visible and accurate

#### T2.4: Track Controls
**Objective**: Test per-track parameter controls
**Steps**:
1. Navigate to track controls area (`Tab` within sequencer)
2. Test volume adjustment:
   - Use `+/-` keys to adjust volume
   - Verify visual slider updates
   - Test range (0-100%)
3. Test pan control:
   - Adjust pan with `+/-` keys
   - Verify center detent at 0
   - Test range (-100% to +100%)
4. Test mute functionality:
   - Press `Enter` on mute control
   - Verify visual state change
5. Test solo functionality:
   - Press `Enter` on solo control
   - Verify visual state change

**Expected Results**:
- Volume and pan controls respond smoothly
- Visual feedback updates in real-time
- Mute/solo states toggle correctly
- Parameter ranges enforced properly

### Pattern Management Tests

#### T2.5: Copy/Paste Operations
**Objective**: Test pattern clipboard functionality
**Steps**:
1. Create a pattern on track 1 (enable several steps)
2. Press `Ctrl+C` to copy pattern
3. Navigate to track 2
4. Press `Ctrl+V` to paste pattern
5. Verify track 2 now matches track 1 pattern
6. Test cut operation:
   - Select track 1
   - Press `Ctrl+X` to cut
   - Verify track 1 cleared
   - Paste to track 3, verify pattern restored

**Expected Results**:
- Copy operation preserves pattern data
- Paste operation accurately reproduces pattern
- Cut operation clears source and preserves clipboard
- Status messages confirm operations

#### T2.6: Track Clear Operations
**Objective**: Test track clearing functionality
**Steps**:
1. Create patterns on multiple tracks
2. Press `C` to clear current track
3. Verify only current track cleared
4. Test `Delete` key on individual steps
5. Verify single step deletion

**Expected Results**:
- Track clear removes all steps from target track only
- Individual step deletion works correctly
- Other tracks remain unaffected

#### T2.7: Selection Operations
**Objective**: Test advanced selection features
**Steps**:
1. Press `Ctrl+S` to start selection
2. Use arrow keys to extend selection
3. Verify selection highlighted visually
4. Test bulk operations on selection:
   - `Ctrl+F` to fill selection
   - `Ctrl+E` to empty selection
5. Test select all operations:
   - `Ctrl+A` to select entire track
   - `Alt+A` to select entire step column

**Expected Results**:
- Selection highlighting clear and accurate
- Bulk operations affect only selected areas
- Select all operations work correctly

### Transport Controls Tests

#### T2.8: Play/Stop Functionality
**Objective**: Test basic transport controls
**Steps**:
1. Navigate to Transport section (`Tab` to focus)
2. Press `Enter` or `Space` to start playback
3. Verify play indicator changes (▶ → ■)
4. Watch for playback position indicator
5. Press `Enter` or `Space` again to stop
6. Verify stop indicator and position reset

**Expected Results**:
- Transport state changes reflected in UI
- Playback position updates during playback
- Start/stop operations work reliably

#### T2.9: Tempo Control
**Objective**: Test tempo adjustment
**Steps**:
1. Focus on tempo control in transport
2. Use `+/-` or arrow keys to adjust tempo
3. Test tempo range (60-200 BPM typically)
4. Verify tempo display updates
5. If playing, verify tempo changes affect playback speed

**Expected Results**:
- Tempo adjusts smoothly within valid range
- Display shows accurate BPM values
- Playback speed reflects tempo changes

---

## PHASE 3: Effects and Advanced Features Testing

### Filter Section Tests

#### T3.1: Filter Type Selection
**Objective**: Test filter type parameter
**Steps**:
1. Press `2` to focus Filter section
2. Navigate to filter type control
3. Test available filter types:
   - LowPass
   - HighPass  
   - BandPass
   - Notch
4. Verify type changes reflected in display

**Expected Results**:
- All filter types accessible
- Current filter type displayed correctly
- Type changes sent to audio bridge

#### T3.2: Filter Cutoff Control
**Objective**: Test filter cutoff frequency
**Steps**:
1. Focus on cutoff control
2. Test logarithmic scaling (20Hz - 20kHz)
3. Adjust with arrow keys and +/- keys
4. Verify smooth parameter changes
5. Listen for audio filtering effects
6. Test reset functionality (`r` key)

**Expected Results**:
- Cutoff frequency adjusts logarithmically
- Audio filtering audible when applied
- Parameter range properly enforced
- Reset returns to default value

#### T3.3: Filter Resonance Control
**Objective**: Test filter resonance/Q factor
**Steps**:
1. Focus on resonance control
2. Test range (typically 0.0 - 1.0)
3. Adjust with linear scaling
4. Listen for resonance effects on audio
5. Test reset functionality

**Expected Results**:
- Resonance adjusts linearly
- Audio resonance effects audible
- No instability at high resonance values
- Reset functionality works

#### T3.4: Filter Mix Control
**Objective**: Test dry/wet mix control
**Steps**:
1. Focus on mix control
2. Test range (0% = dry, 100% = wet)
3. Verify audio blending between dry and filtered signal
4. Test at extremes (0% and 100%)

**Expected Results**:
- Mix control provides smooth blend
- 0% bypasses filter completely
- 100% provides full filtered signal
- Intermediate values blend appropriately

### Envelope Section Tests  

#### T3.5: ADSR Parameters
**Objective**: Test envelope parameter controls
**Steps**:
1. Press `3` to focus Envelope section
2. Test Attack parameter:
   - Range: 0.01s - 10s (typical)
   - Time scaling (linear or logarithmic)
3. Test Decay parameter:
   - Similar range and scaling
4. Test Sustain parameter:
   - Range: 0% - 100% level
5. Test Release parameter:
   - Time range similar to Attack/Decay

**Expected Results**:
- All ADSR parameters adjustable
- Time parameters use appropriate scaling
- Sustain level parameter works correctly
- Parameter changes affect audio envelope

#### T3.6: Envelope Visualization
**Objective**: Test envelope shape display (if implemented)
**Steps**:
1. Adjust ADSR parameters
2. Observe envelope visualization
3. Verify shape reflects parameter values
4. Test with extreme parameter values

**Expected Results**:
- Envelope shape updates with parameter changes
- Visualization accurately represents ADSR curve
- Clear visual feedback for all parameter values

### Performance and Integration Tests

#### T3.7: Multi-Parameter Real-Time Updates
**Objective**: Test system performance with multiple simultaneous parameter changes
**Steps**:
1. Rapidly adjust multiple parameters simultaneously:
   - Oscillator frequency + filter cutoff
   - Volume + resonance + envelope attack
2. Hold down multiple keys for continuous adjustment
3. Monitor for audio dropouts or UI lag
4. Check parameter update latency

**Expected Results**:
- System handles multiple simultaneous updates
- No audio dropouts during intensive parameter changes
- UI remains responsive
- Parameter updates maintain <10ms latency

#### T3.8: Session State Management
**Objective**: Test parameter state persistence
**Steps**:
1. Set various parameters across all sections
2. Test parameter reset functionality (`r` key)
3. Verify state consistency across UI and audio
4. Test section-to-section navigation with state preservation

**Expected Results**:
- Parameter states maintained correctly
- Reset functionality works for all parameters
- No state corruption during navigation
- Audio output matches UI parameter displays

### Error Handling and Edge Cases

#### T3.9: Terminal Resize Handling
**Objective**: Test UI behavior with terminal size changes
**Steps**:
1. Resize terminal to minimum size (80x24)
2. Verify layout adapts appropriately
3. Resize to very large size
4. Test UI responsiveness during resize
5. Verify no UI corruption

**Expected Results**:
- UI adapts gracefully to size changes
- Minimum size supported without corruption
- Large sizes utilize space effectively
- No crashes or visual artifacts

#### T3.10: Audio System Error Handling
**Objective**: Test behavior when audio system unavailable
**Steps**:
1. Start TUI without audio device
2. Verify graceful degradation
3. Test parameter updates still work in UI
4. Check error messages are informative

**Expected Results**:
- Application starts even without audio
- Clear error messages displayed
- UI functionality remains available
- No crashes due to audio system errors

---

## Test Environment Requirements

### Hardware
- **CPU**: Multi-core processor (audio thread separation)
- **Memory**: Minimum 4GB RAM
- **Audio**: Working audio output device
- **Display**: Terminal supporting 256 colors (recommended)

### Software
- **OS**: macOS 10.15+, Linux (Ubuntu 20.04+), Windows 10+
- **Terminal**: Modern terminal emulator (iTerm2, GNOME Terminal, Windows Terminal)
- **Rust**: 1.70+ (for building from source)

### Performance Benchmarks
- **Parameter Update Latency**: <5ms (measured from key press to audio change)
- **UI Refresh Rate**: 60 FPS sustained
- **Memory Usage**: <100MB during normal operation
- **CPU Usage**: <10% for UI operations (separate from audio thread)

---

## Test Execution Notes

### Preparation
1. Build TUI with `cargo build --release` for performance testing
2. Ensure audio system properly configured
3. Test in quiet environment for audio verification
4. Document terminal emulator and OS versions

### Test Execution
1. Execute tests in order (Phase 1 → 2 → 3)
2. Note any deviations from expected results
3. Record performance measurements where specified
4. Test each major feature completely before proceeding
5. Document any crashes or error conditions

### Results Documentation
For each test, record:
- **PASS/FAIL** status
- Performance measurements (where applicable) 
- Any notable observations
- Deviation details for failed tests
- System configuration for performance tests

This test plan ensures comprehensive coverage of the first three phases of TUI implementation, focusing on user interaction, audio integration, and real-time performance characteristics essential for a professional music production interface.
