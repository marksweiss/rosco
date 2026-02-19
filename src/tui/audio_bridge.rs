use crate::tui::TuiError;
use crate::audio_gen;
use crate::tui::audio_engine::{AudioEngine, AudioState};
use std::sync::{Arc, mpsc};
use std::sync::atomic::Ordering;

#[derive(Debug, Clone)]
pub enum ParameterUpdate {
    // Oscillator parameters
    OscillatorVolume(f32),
    OscillatorWaveform(audio_gen::Waveform),

    // Filter parameters
    FilterType(crate::tui::ui::widgets::selector::FilterType),
    FilterCutoff(f32),
    FilterCenterFrequency(f32),
    FilterBandwidth(f32),
    FilterResonance(f32),
    FilterMix(f32),

    // Envelope parameters
    EnvelopeAttackTime(f32),
    EnvelopeAttackLevel(f32),
    EnvelopeDecayTime(f32),
    EnvelopeDecayLevel(f32),
    EnvelopeSustainTime(f32),
    EnvelopeSustainLevel(f32),
    EnvelopeReleaseTime(f32),
    EnvelopeReleaseLevel(f32),

    // Delay effect parameters
    DelayTime(f32),       // 0.0-1.0s delay time
    DelayFeedback(f32),   // 0.0-1.0 feedback amount
    DelayMix(f32),        // 0.0-1.0 dry/wet mix
    DelayEnabled(bool),

    // Flanger effect parameters
    FlangerRate(f32),     // 0.1-10.0 Hz LFO rate
    FlangerDepth(f32),    // 0.0-1.0 modulation depth
    FlangerMix(f32),      // 0.0-1.0 dry/wet mix
    FlangerEnabled(bool),

    // LFO parameters
    LfoRate(f32),         // 0.1-20.0 Hz
    LfoDepth(f32),        // 0.0-1.0
    LfoTarget(LfoTarget), // What parameter LFO modulates
    LfoEnabled(bool),

    // Sequencer parameters
    SequencerStep { track: u8, step: u8, enabled: bool },

    // Transport parameters
    TransportPlay,
    TransportStop,
    TempoChange(f32),
}

/// LFO modulation target
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LfoTarget {
    FilterCutoff,
    Volume,
    Pan,
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