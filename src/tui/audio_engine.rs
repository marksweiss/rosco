use std::sync::{Arc, atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering}};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use atomic_float::AtomicF32;

use crate::audio_gen::oscillator::{OscillatorTables, Waveform, get_sample, get_gaussian_noise_sample};
use crate::common::constants::SAMPLE_RATE;
use crate::tui::audio_bridge::{ParameterUpdate, AudioFeedback};
use crate::tui::TuiError;

/// Filter type enum for audio thread (mirrors UI FilterType)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AudioFilterType {
    LowPass = 0,
    HighPass = 1,
    BandPass = 2,
    Notch = 3,
}

impl From<u32> for AudioFilterType {
    fn from(value: u32) -> Self {
        match value {
            0 => AudioFilterType::LowPass,
            1 => AudioFilterType::HighPass,
            2 => AudioFilterType::BandPass,
            3 => AudioFilterType::Notch,
            _ => AudioFilterType::LowPass,
        }
    }
}

/// Real-time safe biquad filter for use in audio callback
/// Uses Direct Form II implementation
#[derive(Debug)]
pub struct RealtimeFilter {
    // Filter coefficients
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    // Filter state (history)
    x_history: [f32; 2],
    y_history: [f32; 2],
    // Current parameters for recalculation
    filter_type: AudioFilterType,
    frequency: f32,
    bandwidth: f32,
    resonance: f32,
    mix: f32,
}

impl Default for RealtimeFilter {
    fn default() -> Self {
        let mut filter = Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            x_history: [0.0; 2],
            y_history: [0.0; 2],
            filter_type: AudioFilterType::LowPass,
            frequency: 1000.0,
            bandwidth: 200.0,
            resonance: 0.3,
            mix: 0.8,
        };
        filter.update_coefficients();
        filter
    }
}

impl RealtimeFilter {
    /// Update filter parameters and recalculate coefficients
    pub fn update_params(&mut self, filter_type: AudioFilterType, frequency: f32, bandwidth: f32, resonance: f32, mix: f32) {
        self.filter_type = filter_type;
        self.frequency = frequency.clamp(20.0, SAMPLE_RATE / 2.0 * 0.99);
        self.bandwidth = bandwidth.clamp(10.0, 5000.0);
        self.resonance = resonance.clamp(0.0, 1.0);
        self.mix = mix.clamp(0.0, 1.0);
        self.update_coefficients();
    }

    /// Recalculate filter coefficients based on current parameters
    fn update_coefficients(&mut self) {
        let omega = 2.0 * std::f32::consts::PI * self.frequency / SAMPLE_RATE;
        let sin_w = omega.sin();
        let cos_w = omega.cos();

        // Calculate Q factor from resonance
        let q = if self.resonance > 0.0 {
            1.0 / (2.0 * self.resonance)
        } else {
            0.707 // Butterworth response
        };

        let alpha = sin_w / (2.0 * q);

        // Calculate coefficients based on filter type
        let (b0, b1, b2, a0, a1, a2) = match self.filter_type {
            AudioFilterType::LowPass => {
                let b0 = (1.0 - cos_w) / 2.0;
                let b1 = 1.0 - cos_w;
                let b2 = (1.0 - cos_w) / 2.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            AudioFilterType::HighPass => {
                let b0 = (1.0 + cos_w) / 2.0;
                let b1 = -(1.0 + cos_w);
                let b2 = (1.0 + cos_w) / 2.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            AudioFilterType::BandPass => {
                // For band-pass, use bandwidth to calculate Q
                let bw_omega = 2.0 * std::f32::consts::PI * self.bandwidth / SAMPLE_RATE;
                let bw_alpha = sin_w * (bw_omega / 2.0).sinh();
                let b0 = bw_alpha;
                let b1 = 0.0;
                let b2 = -bw_alpha;
                let a0 = 1.0 + bw_alpha;
                let a1 = -2.0 * cos_w;
                let a2 = 1.0 - bw_alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            AudioFilterType::Notch => {
                // Notch filter (band-reject)
                let bw_omega = 2.0 * std::f32::consts::PI * self.bandwidth / SAMPLE_RATE;
                let bw_alpha = sin_w * (bw_omega / 2.0).sinh();
                let b0 = 1.0;
                let b1 = -2.0 * cos_w;
                let b2 = 1.0;
                let a0 = 1.0 + bw_alpha;
                let a1 = -2.0 * cos_w;
                let a2 = 1.0 - bw_alpha;
                (b0, b1, b2, a0, a1, a2)
            }
        };

        // Normalize by a0
        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }

    /// Process a single sample through the filter
    pub fn process(&mut self, sample: f32) -> f32 {
        // Direct Form II implementation
        let w = sample - self.a1 * self.x_history[0] - self.a2 * self.x_history[1];
        let filtered = self.b0 * w + self.b1 * self.x_history[0] + self.b2 * self.x_history[1];

        // Update history
        self.x_history[1] = self.x_history[0];
        self.x_history[0] = w;
        self.y_history[1] = self.y_history[0];
        self.y_history[0] = filtered;

        // Apply mix (dry/wet blend)
        sample * (1.0 - self.mix) + filtered * self.mix
    }

    /// Reset filter state
    pub fn reset(&mut self) {
        self.x_history = [0.0; 2];
        self.y_history = [0.0; 2];
    }
}

/// Real-time envelope generator for use in audio callback
#[derive(Debug)]
pub struct RealtimeEnvelope {
    // ADSR parameters (times in seconds, levels 0.0-1.0)
    pub attack_time: f32,
    pub attack_level: f32,
    pub decay_time: f32,
    pub decay_level: f32,
    pub sustain_time: f32,
    pub sustain_level: f32,
    pub release_time: f32,
    pub release_level: f32,
}

impl Default for RealtimeEnvelope {
    fn default() -> Self {
        Self {
            attack_time: 0.01,
            attack_level: 1.0,
            decay_time: 0.1,
            decay_level: 0.8,
            sustain_time: 0.5,
            sustain_level: 0.7,
            release_time: 0.2,
            release_level: 0.0,
        }
    }
}

impl RealtimeEnvelope {
    /// Calculate the envelope amplitude at a given position (0.0 to 1.0)
    pub fn amplitude_at(&self, position: f32) -> f32 {
        // Convert ADSR times to normalized positions
        let total_time = self.attack_time + self.decay_time + self.sustain_time + self.release_time;
        if total_time <= 0.0 {
            return 1.0;
        }

        let attack_end = self.attack_time / total_time;
        let decay_end = attack_end + self.decay_time / total_time;
        let sustain_end = decay_end + self.sustain_time / total_time;
        // release_end is 1.0

        if position < attack_end {
            // Attack phase: 0 -> attack_level
            if attack_end > 0.0 {
                let t = position / attack_end;
                t * self.attack_level
            } else {
                self.attack_level
            }
        } else if position < decay_end {
            // Decay phase: attack_level -> decay_level
            if decay_end > attack_end {
                let t = (position - attack_end) / (decay_end - attack_end);
                self.attack_level + t * (self.decay_level - self.attack_level)
            } else {
                self.decay_level
            }
        } else if position < sustain_end {
            // Sustain phase: decay_level -> sustain_level
            if sustain_end > decay_end {
                let t = (position - decay_end) / (sustain_end - decay_end);
                self.decay_level + t * (self.sustain_level - self.decay_level)
            } else {
                self.sustain_level
            }
        } else {
            // Release phase: sustain_level -> release_level
            if position < 1.0 {
                let t = (position - sustain_end) / (1.0 - sustain_end);
                self.sustain_level + t * (self.release_level - self.sustain_level)
            } else {
                self.release_level
            }
        }
    }
}

/// Real-time audio engine that integrates with the TUI
pub struct AudioEngine {
    // Control channels
    #[allow(dead_code)]
    parameter_rx: Receiver<ParameterUpdate>,
    #[allow(dead_code)]
    feedback_tx: Sender<AudioFeedback>,
    
    // Audio thread control
    is_running: Arc<AtomicBool>,
    _audio_thread: thread::JoinHandle<()>,
    
    // Stream handle (kept alive)
    _stream: cpal::Stream,
}

/// Shared audio state accessible from audio callback
#[derive(Debug)]
pub struct AudioState {
    // Transport state
    pub is_playing: AtomicBool,
    pub current_step: AtomicUsize, // 0-15
    pub tempo: AtomicF32,          // BPM

    // Oscillator parameters
    pub osc_frequency: AtomicF32,
    pub osc_volume: AtomicF32,
    pub osc_waveform: AtomicU32, // Waveform as u32

    // Filter parameters (atomic for thread-safe updates)
    pub filter_type: AtomicU32,      // AudioFilterType as u32
    pub filter_frequency: AtomicF32, // Cutoff/center frequency
    pub filter_bandwidth: AtomicF32, // Bandwidth for BP/Notch
    pub filter_resonance: AtomicF32, // Q factor
    pub filter_mix: AtomicF32,       // Dry/wet mix
    pub filter_enabled: AtomicBool,  // Enable/disable filter

    // Envelope parameters (atomic for thread-safe updates)
    pub env_attack_time: AtomicF32,
    pub env_attack_level: AtomicF32,
    pub env_decay_time: AtomicF32,
    pub env_decay_level: AtomicF32,
    pub env_sustain_time: AtomicF32,
    pub env_sustain_level: AtomicF32,
    pub env_release_time: AtomicF32,
    pub env_release_level: AtomicF32,
    pub env_enabled: AtomicBool, // Enable/disable envelope

    // Delay effect parameters
    pub delay_time: AtomicF32,     // 0.0-1.0s
    pub delay_feedback: AtomicF32, // 0.0-1.0
    pub delay_mix: AtomicF32,      // 0.0-1.0
    pub delay_enabled: AtomicBool,

    // Flanger effect parameters
    pub flanger_rate: AtomicF32,   // 0.1-10.0 Hz
    pub flanger_depth: AtomicF32,  // 0.0-1.0
    pub flanger_mix: AtomicF32,    // 0.0-1.0
    pub flanger_enabled: AtomicBool,

    // LFO parameters
    pub lfo_rate: AtomicF32,       // 0.1-20.0 Hz
    pub lfo_depth: AtomicF32,      // 0.0-1.0
    pub lfo_target: AtomicU32,     // LfoTarget as u32
    pub lfo_enabled: AtomicBool,

    // Sample timing
    pub sample_count: AtomicUsize,
    pub last_step_time: Arc<parking_lot::Mutex<Instant>>,

    // Sequencer data - fixed for proper step frequency support
    pub track_steps: [AtomicBool; 8 * 16], // 8 tracks × 16 steps
    pub track_volumes: [AtomicF32; 8],
    pub step_frequencies: [AtomicF32; 8 * 16], // One frequency per step (8 tracks × 16 steps)
}

impl Default for AudioState {
    fn default() -> Self {
        // Initialize track steps array
        let track_steps: [AtomicBool; 8 * 16] = std::array::from_fn(|_| AtomicBool::new(false));
        let track_volumes: [AtomicF32; 8] = std::array::from_fn(|_| AtomicF32::new(0.8));

        // Initialize step frequencies - each step gets a default frequency
        // Initialize with C3 (261.63 Hz) for all steps
        let step_frequencies: [AtomicF32; 8 * 16] = std::array::from_fn(|_| {
            AtomicF32::new(261.63) // Default to C3
        });

        Self {
            is_playing: AtomicBool::new(false),
            current_step: AtomicUsize::new(0),
            tempo: AtomicF32::new(120.0),
            osc_frequency: AtomicF32::new(440.0),
            osc_volume: AtomicF32::new(0.75),
            osc_waveform: AtomicU32::new(Waveform::Sine as u32),

            // Filter defaults
            filter_type: AtomicU32::new(AudioFilterType::LowPass as u32),
            filter_frequency: AtomicF32::new(1000.0),
            filter_bandwidth: AtomicF32::new(200.0),
            filter_resonance: AtomicF32::new(0.3),
            filter_mix: AtomicF32::new(0.8),
            filter_enabled: AtomicBool::new(true),

            // Envelope defaults
            env_attack_time: AtomicF32::new(0.01),
            env_attack_level: AtomicF32::new(1.0),
            env_decay_time: AtomicF32::new(0.1),
            env_decay_level: AtomicF32::new(0.8),
            env_sustain_time: AtomicF32::new(0.5),
            env_sustain_level: AtomicF32::new(0.7),
            env_release_time: AtomicF32::new(0.2),
            env_release_level: AtomicF32::new(0.0),
            env_enabled: AtomicBool::new(true),

            // Delay effect defaults
            delay_time: AtomicF32::new(0.25),     // 250ms default
            delay_feedback: AtomicF32::new(0.3),  // 30% feedback
            delay_mix: AtomicF32::new(0.3),       // 30% wet
            delay_enabled: AtomicBool::new(false),

            // Flanger effect defaults
            flanger_rate: AtomicF32::new(0.5),    // 0.5 Hz
            flanger_depth: AtomicF32::new(0.5),   // 50% depth
            flanger_mix: AtomicF32::new(0.5),     // 50% wet
            flanger_enabled: AtomicBool::new(false),

            // LFO defaults
            lfo_rate: AtomicF32::new(1.0),        // 1 Hz
            lfo_depth: AtomicF32::new(0.5),       // 50% depth
            lfo_target: AtomicU32::new(0),        // FilterCutoff
            lfo_enabled: AtomicBool::new(false),

            sample_count: AtomicUsize::new(0),
            last_step_time: Arc::new(parking_lot::Mutex::new(Instant::now())),
            track_steps,
            track_volumes,
            step_frequencies,
        }
    }
}

impl AudioEngine {
    pub fn new() -> Result<(Self, Arc<AudioState>, mpsc::Sender<ParameterUpdate>, mpsc::Receiver<AudioFeedback>), TuiError> {
        // Create communication channels
        let (param_tx, parameter_rx) = mpsc::channel::<ParameterUpdate>();
        let (feedback_tx, feedback_rx) = mpsc::channel::<AudioFeedback>();

        // Create shared audio state
        let audio_state = Arc::new(AudioState::default());

        // Initialize audio
        let host = cpal::default_host();
        let device = host.default_output_device()
            .ok_or_else(|| TuiError::Audio("No output device available".to_string()))?;
        let config = device.default_output_config()
            .map_err(|e| TuiError::Audio(format!("Failed to get default config: {}", e)))?;

        let config: cpal::StreamConfig = config.into();

        // Create oscillator tables
        let osc_tables = OscillatorTables::new();

        // Clone state and feedback sender for audio callback
        let audio_state_callback = Arc::clone(&audio_state);
        let feedback_tx_callback = feedback_tx.clone();

        // Create filter and envelope instances for the audio callback
        // These are created inside the closure to maintain their state
        let mut filter_left = RealtimeFilter::default();
        let mut filter_right = RealtimeFilter::default();
        let envelope = RealtimeEnvelope::default();

        // Track filter params to detect changes
        let mut last_filter_type = AudioFilterType::LowPass;
        let mut last_filter_freq = 1000.0f32;
        let mut last_filter_bw = 200.0f32;
        let mut last_filter_res = 0.3f32;
        let mut last_filter_mix = 0.8f32;

        // Create audio stream
        let stream = device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                audio_callback_with_effects(
                    data,
                    &audio_state_callback,
                    &osc_tables,
                    &feedback_tx_callback,
                    &mut filter_left,
                    &mut filter_right,
                    &envelope,
                    &mut last_filter_type,
                    &mut last_filter_freq,
                    &mut last_filter_bw,
                    &mut last_filter_res,
                    &mut last_filter_mix,
                );
            },
            |err| eprintln!("Audio stream error: {}", err),
            None,
        ).map_err(|e| TuiError::Audio(format!("Failed to build audio stream: {}", e)))?;

        // Start the stream
        stream.play()
            .map_err(|e| TuiError::Audio(format!("Failed to start audio stream: {}", e)))?;

        // Create control flags
        let is_running = Arc::new(AtomicBool::new(true));
        let is_running_thread = Arc::clone(&is_running);
        let audio_state_thread = Arc::clone(&audio_state);

        // Start parameter processing thread
        let audio_thread = thread::spawn(move || {
            audio_parameter_thread(parameter_rx, is_running_thread, audio_state_thread);
        });

        let engine = AudioEngine {
            parameter_rx: mpsc::channel().1, // Dummy receiver, real one is in thread
            feedback_tx,
            is_running,
            _audio_thread: audio_thread,
            _stream: stream,
        };

        Ok((engine, audio_state, param_tx, feedback_rx))
    }
    
    /// Create a parameter sender for the TUI to send updates
    pub fn create_parameter_sender(&self) -> Sender<ParameterUpdate> {
        // This is a bit hacky for Phase 1 - in a real implementation
        // we'd store the sender in the engine. For now, we'll create
        // a new channel pair and let the caller handle it.
        mpsc::channel().0
    }
}

impl Drop for AudioEngine {
    fn drop(&mut self) {
        self.is_running.store(false, Ordering::Relaxed);
        // Audio thread will exit on next iteration
    }
}

/// Audio callback function with filter and envelope processing - runs in real-time audio thread
#[allow(clippy::too_many_arguments)]
fn audio_callback_with_effects(
    data: &mut [f32],
    audio_state: &AudioState,
    osc_tables: &OscillatorTables,
    feedback_tx: &Sender<AudioFeedback>,
    filter_left: &mut RealtimeFilter,
    filter_right: &mut RealtimeFilter,
    _envelope: &RealtimeEnvelope, // Kept for API compatibility, but we read from AudioState
    last_filter_type: &mut AudioFilterType,
    last_filter_freq: &mut f32,
    last_filter_bw: &mut f32,
    last_filter_res: &mut f32,
    last_filter_mix: &mut f32,
) {
    let channels = 2; // Stereo
    let is_playing = audio_state.is_playing.load(Ordering::Relaxed);

    if !is_playing {
        // Fill with silence when not playing
        for sample in data.iter_mut() {
            *sample = 0.0;
        }
        // Reset filter state when stopped to prevent artifacts on restart
        filter_left.reset();
        filter_right.reset();
        return;
    }

    // Get current parameters
    let current_step = audio_state.current_step.load(Ordering::Relaxed);
    let tempo = audio_state.tempo.load(Ordering::Relaxed);
    let master_volume = audio_state.osc_volume.load(Ordering::Relaxed);
    let waveform_int = audio_state.osc_waveform.load(Ordering::Relaxed);
    let waveform = match waveform_int {
        0 => Waveform::GaussianNoise,
        1 => Waveform::Saw,
        2 => Waveform::Sine,
        3 => Waveform::Square,
        4 => Waveform::Triangle,
        _ => Waveform::Sine,
    };

    // Get filter parameters and check if they changed
    let filter_enabled = audio_state.filter_enabled.load(Ordering::Relaxed);
    let filter_type = AudioFilterType::from(audio_state.filter_type.load(Ordering::Relaxed));
    let filter_freq = audio_state.filter_frequency.load(Ordering::Relaxed);
    let filter_bw = audio_state.filter_bandwidth.load(Ordering::Relaxed);
    let filter_res = audio_state.filter_resonance.load(Ordering::Relaxed);
    let filter_mix = audio_state.filter_mix.load(Ordering::Relaxed);

    // Update filter coefficients if parameters changed
    if filter_type != *last_filter_type
        || (filter_freq - *last_filter_freq).abs() > 0.01
        || (filter_bw - *last_filter_bw).abs() > 0.01
        || (filter_res - *last_filter_res).abs() > 0.001
        || (filter_mix - *last_filter_mix).abs() > 0.001
    {
        filter_left.update_params(filter_type, filter_freq, filter_bw, filter_res, filter_mix);
        filter_right.update_params(filter_type, filter_freq, filter_bw, filter_res, filter_mix);
        *last_filter_type = filter_type;
        *last_filter_freq = filter_freq;
        *last_filter_bw = filter_bw;
        *last_filter_res = filter_res;
        *last_filter_mix = filter_mix;
    }

    // Get envelope parameters and create a dynamic envelope
    let env_enabled = audio_state.env_enabled.load(Ordering::Relaxed);
    let dynamic_envelope = RealtimeEnvelope {
        attack_time: audio_state.env_attack_time.load(Ordering::Relaxed),
        attack_level: audio_state.env_attack_level.load(Ordering::Relaxed),
        decay_time: audio_state.env_decay_time.load(Ordering::Relaxed),
        decay_level: audio_state.env_decay_level.load(Ordering::Relaxed),
        sustain_time: audio_state.env_sustain_time.load(Ordering::Relaxed),
        sustain_level: audio_state.env_sustain_level.load(Ordering::Relaxed),
        release_time: audio_state.env_release_time.load(Ordering::Relaxed),
        release_level: audio_state.env_release_level.load(Ordering::Relaxed),
    };

    // Calculate timing for step advancement
    let samples_per_step = (SAMPLE_RATE * 60.0 / tempo) as usize;

    for frame in data.chunks_mut(channels) {
        let sample_count = audio_state.sample_count.fetch_add(1, Ordering::Relaxed);

        // Check if we should advance to next step
        if sample_count % samples_per_step == 0 && sample_count > 0 {
            let new_step = (current_step + 1) % 16;
            audio_state.current_step.store(new_step, Ordering::Relaxed);

            // Send step position feedback to TUI (non-blocking)
            let _ = feedback_tx.send(AudioFeedback::PlaybackPosition(new_step as f32));
        }

        let current_step = audio_state.current_step.load(Ordering::Relaxed);

        // Calculate envelope position within current step (0.0 to 1.0)
        let step_sample_position = sample_count % samples_per_step;
        let step_progress = step_sample_position as f32 / samples_per_step as f32;

        // Generate audio for all active tracks at current step
        let mut left_sample = 0.0f32;
        let mut right_sample = 0.0f32;

        for track_idx in 0..8 {
            let step_index = track_idx * 16 + current_step;
            let is_step_active = audio_state.track_steps[step_index].load(Ordering::Relaxed);

            if is_step_active {
                let track_volume = audio_state.track_volumes[track_idx].load(Ordering::Relaxed);
                let step_frequency = audio_state.step_frequencies[step_index].load(Ordering::Relaxed);

                // Generate sample based on waveform using step-specific frequency
                let sample = match waveform {
                    Waveform::GaussianNoise | Waveform::Noise => get_gaussian_noise_sample(),
                    Waveform::Sine => get_sample(&osc_tables.sine_table, step_frequency, sample_count as u64),
                    Waveform::Saw => get_sample(&osc_tables.saw_table, step_frequency, sample_count as u64),
                    Waveform::Square => get_sample(&osc_tables.square_table, step_frequency, sample_count as u64),
                    Waveform::Triangle => get_sample(&osc_tables.triangle_table, step_frequency, sample_count as u64),
                };

                // Apply envelope if enabled
                let enveloped_sample = if env_enabled {
                    sample * dynamic_envelope.amplitude_at(step_progress)
                } else {
                    sample
                };

                let final_sample = enveloped_sample * track_volume * master_volume * 0.1; // Scale down to prevent clipping

                left_sample += final_sample;
                right_sample += final_sample;
            }
        }

        // Apply filter if enabled
        if filter_enabled {
            left_sample = filter_left.process(left_sample);
            right_sample = filter_right.process(right_sample);
        }

        // Apply simple limiting to prevent clipping
        left_sample = left_sample.clamp(-1.0, 1.0);
        right_sample = right_sample.clamp(-1.0, 1.0);

        // Write to output buffer (interleaved stereo)
        if frame.len() >= 2 {
            frame[0] = left_sample;  // Left channel
            frame[1] = right_sample; // Right channel
        }
    }
}

/// Parameter processing thread
fn audio_parameter_thread(
    parameter_rx: Receiver<ParameterUpdate>,
    is_running: Arc<AtomicBool>,
    audio_state: Arc<AudioState>,
) {
    while is_running.load(Ordering::Relaxed) {
        // Process parameter updates with timeout
        match parameter_rx.recv_timeout(Duration::from_millis(10)) {
            Ok(update) => {
                process_parameter_update(update, &audio_state);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // Normal timeout, continue loop
                continue;
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                // Channel disconnected, exit thread
                break;
            }
        }
    }
}

/// Process a single parameter update
fn process_parameter_update(update: ParameterUpdate, audio_state: &AudioState) {
    match update {
        // Transport controls
        ParameterUpdate::TransportPlay => {
            audio_state.is_playing.store(true, Ordering::Relaxed);
            let mut last_step_time = audio_state.last_step_time.lock();
            *last_step_time = Instant::now();
        }
        ParameterUpdate::TransportStop => {
            audio_state.is_playing.store(false, Ordering::Relaxed);
        }
        ParameterUpdate::TempoChange(tempo) => {
            audio_state.tempo.store(tempo, Ordering::Relaxed);
        }

        // Oscillator controls
        ParameterUpdate::OscillatorVolume(volume) => {
            audio_state.osc_volume.store(volume, Ordering::Relaxed);
        }
        ParameterUpdate::OscillatorWaveform(waveform) => {
            let waveform_int = waveform as u32;
            audio_state.osc_waveform.store(waveform_int, Ordering::Relaxed);
        }

        // Filter controls
        ParameterUpdate::FilterType(filter_type) => {
            use crate::tui::ui::widgets::selector::FilterType;
            let filter_int = match filter_type {
                FilterType::LowPass => AudioFilterType::LowPass as u32,
                FilterType::HighPass => AudioFilterType::HighPass as u32,
                FilterType::BandPass => AudioFilterType::BandPass as u32,
                FilterType::Notch => AudioFilterType::Notch as u32,
            };
            audio_state.filter_type.store(filter_int, Ordering::Relaxed);
        }
        ParameterUpdate::FilterCutoff(cutoff) => {
            audio_state.filter_frequency.store(cutoff, Ordering::Relaxed);
        }
        ParameterUpdate::FilterCenterFrequency(center_freq) => {
            audio_state.filter_frequency.store(center_freq, Ordering::Relaxed);
        }
        ParameterUpdate::FilterBandwidth(bandwidth) => {
            audio_state.filter_bandwidth.store(bandwidth, Ordering::Relaxed);
        }
        ParameterUpdate::FilterResonance(resonance) => {
            audio_state.filter_resonance.store(resonance, Ordering::Relaxed);
        }
        ParameterUpdate::FilterMix(mix) => {
            audio_state.filter_mix.store(mix, Ordering::Relaxed);
        }

        // Envelope controls
        ParameterUpdate::EnvelopeAttackTime(time) => {
            audio_state.env_attack_time.store(time, Ordering::Relaxed);
        }
        ParameterUpdate::EnvelopeAttackLevel(level) => {
            audio_state.env_attack_level.store(level, Ordering::Relaxed);
        }
        ParameterUpdate::EnvelopeDecayTime(time) => {
            audio_state.env_decay_time.store(time, Ordering::Relaxed);
        }
        ParameterUpdate::EnvelopeDecayLevel(level) => {
            audio_state.env_decay_level.store(level, Ordering::Relaxed);
        }
        ParameterUpdate::EnvelopeSustainTime(time) => {
            audio_state.env_sustain_time.store(time, Ordering::Relaxed);
        }
        ParameterUpdate::EnvelopeSustainLevel(level) => {
            audio_state.env_sustain_level.store(level, Ordering::Relaxed);
        }
        ParameterUpdate::EnvelopeReleaseTime(time) => {
            audio_state.env_release_time.store(time, Ordering::Relaxed);
        }
        ParameterUpdate::EnvelopeReleaseLevel(level) => {
            audio_state.env_release_level.store(level, Ordering::Relaxed);
        }

        // Delay effect controls
        ParameterUpdate::DelayTime(time) => {
            audio_state.delay_time.store(time, Ordering::Relaxed);
        }
        ParameterUpdate::DelayFeedback(feedback) => {
            audio_state.delay_feedback.store(feedback, Ordering::Relaxed);
        }
        ParameterUpdate::DelayMix(mix) => {
            audio_state.delay_mix.store(mix, Ordering::Relaxed);
        }
        ParameterUpdate::DelayEnabled(enabled) => {
            audio_state.delay_enabled.store(enabled, Ordering::Relaxed);
        }

        // Flanger effect controls
        ParameterUpdate::FlangerRate(rate) => {
            audio_state.flanger_rate.store(rate, Ordering::Relaxed);
        }
        ParameterUpdate::FlangerDepth(depth) => {
            audio_state.flanger_depth.store(depth, Ordering::Relaxed);
        }
        ParameterUpdate::FlangerMix(mix) => {
            audio_state.flanger_mix.store(mix, Ordering::Relaxed);
        }
        ParameterUpdate::FlangerEnabled(enabled) => {
            audio_state.flanger_enabled.store(enabled, Ordering::Relaxed);
        }

        // LFO controls
        ParameterUpdate::LfoRate(rate) => {
            audio_state.lfo_rate.store(rate, Ordering::Relaxed);
        }
        ParameterUpdate::LfoDepth(depth) => {
            audio_state.lfo_depth.store(depth, Ordering::Relaxed);
        }
        ParameterUpdate::LfoTarget(target) => {
            audio_state.lfo_target.store(target as u32, Ordering::Relaxed);
        }
        ParameterUpdate::LfoEnabled(enabled) => {
            audio_state.lfo_enabled.store(enabled, Ordering::Relaxed);
        }

        // Sequencer controls
        ParameterUpdate::SequencerStep { track, step, enabled } => {
            if (track as usize) < 8 && (step as usize) < 16 {
                let index = (track as usize) * 16 + (step as usize);
                audio_state.track_steps[index].store(enabled, Ordering::Relaxed);
            }
        }
    }
}