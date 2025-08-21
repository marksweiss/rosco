use std::sync::{Arc, atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering}};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use atomic_float::AtomicF32;

use crate::audio_gen::oscillator::{OscillatorTables, Waveform, get_sample, get_gaussian_noise_sample};
use crate::audio_gen::get_sample::get_note_sample;
use crate::note::playback_note::{PlaybackNote, PlaybackNoteBuilder, NoteType};
use crate::note::note::{Note, NoteBuilder};
use crate::note::scales::WesternPitch;
use crate::common::constants::SAMPLE_RATE;
use crate::tui::audio_bridge::{ParameterUpdate, AudioFeedback};
use crate::tui::track_bridge::TrackData;
use crate::tui::TuiError;

/// Real-time audio engine that integrates with the TUI
pub struct AudioEngine {
    // Control channels
    parameter_rx: Receiver<ParameterUpdate>,
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
        
        // Create audio stream
        let stream = device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                audio_callback(data, &audio_state_callback, &osc_tables, &feedback_tx_callback);
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

/// Audio callback function - this runs in real-time audio thread
fn audio_callback(data: &mut [f32], audio_state: &AudioState, osc_tables: &OscillatorTables, feedback_tx: &Sender<AudioFeedback>) {
    let channels = 2; // Stereo
    let is_playing = audio_state.is_playing.load(Ordering::Relaxed);
    
    if !is_playing {
        // Fill with silence when not playing
        for sample in data.iter_mut() {
            *sample = 0.0;
        }
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
                
                let final_sample = sample * track_volume * master_volume * 0.1; // Scale down to prevent clipping
                
                left_sample += final_sample;
                right_sample += final_sample;
            }
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
        ParameterUpdate::OscillatorFrequency(freq) => {
            audio_state.osc_frequency.store(freq, Ordering::Relaxed);
        }
        ParameterUpdate::OscillatorVolume(volume) => {
            audio_state.osc_volume.store(volume, Ordering::Relaxed);
        }
        ParameterUpdate::OscillatorWaveform(waveform) => {
            let waveform_int = waveform as u32;
            audio_state.osc_waveform.store(waveform_int, Ordering::Relaxed);
        }
        ParameterUpdate::SequencerStep { track, step, enabled } => {
            if (track as usize) < 8 && (step as usize) < 16 {
                let index = (track as usize) * 16 + (step as usize);
                audio_state.track_steps[index].store(enabled, Ordering::Relaxed);
            }
        }
        _ => {
            // Handle other parameter updates as needed
        }
    }
}