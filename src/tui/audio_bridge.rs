use crate::tui::TuiError;
use crate::audio_gen;
use crate::tui::audio_engine::{AudioEngine, AudioState};
use std::sync::{Arc, mpsc};
use atomic_float::AtomicF32;
use std::sync::atomic::Ordering;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum AudioFeedback {
    LevelMeter { track: u8, level: f32 },
    PlaybackPosition(f32),
    CpuUsage(f32),
    BufferHealth(f32),
}

pub struct AudioBridge {
    // Audio engine integration
    _audio_engine: AudioEngine,
    audio_state: Arc<AudioState>,
    
    // Communication channels
    param_tx: mpsc::Sender<ParameterUpdate>,
    feedback_rx: mpsc::Receiver<AudioFeedback>,
}

impl AudioBridge {
    pub fn new() -> Result<Self, TuiError> {
        println!("Creating audio engine...");
        let (audio_engine, audio_state, param_tx, feedback_rx) = AudioEngine::new()?;
        
        println!("AudioBridge initialized with real audio engine");
        Ok(Self {
            _audio_engine: audio_engine,
            audio_state,
            param_tx,
            feedback_rx,
        })
    }
    
    pub fn send_parameter_update(&mut self, update: ParameterUpdate) -> Result<(), TuiError> {
        self.param_tx.send(update)
            .map_err(|e| TuiError::Audio(format!("Failed to send parameter update: {}", e)))
    }
    
    pub fn receive_audio_feedback(&mut self) -> Vec<AudioFeedback> {
        let mut feedback = Vec::new();
        while let Ok(fb) = self.feedback_rx.try_recv() {
            feedback.push(fb);
        }
        feedback
    }
    
    pub fn get_audio_state(&self) -> Arc<AudioState> {
        Arc::clone(&self.audio_state)
    }
    
    pub fn get_oscillator_frequency(&self) -> f32 {
        self.audio_state.osc_frequency.load(Ordering::Relaxed)
    }
    
    pub fn get_master_volume(&self) -> f32 {
        self.audio_state.osc_volume.load(Ordering::Relaxed)
    }
}