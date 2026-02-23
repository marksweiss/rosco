use crate::tui::TuiError;
use crate::audio_gen;
use ringbuf::{HeapRb, HeapProducer, HeapConsumer};
use std::sync::Arc;
use atomic_float::AtomicF32;
use std::sync::atomic::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum FilterKind {
    LowPass,
    HighPass,
    BandPass,
    Notch,
}

#[derive(Debug, Clone)]
pub enum ParameterUpdate {
    // Oscillator
    OscillatorFrequency(f32),
    OscillatorVolume(f32),
    OscillatorWaveform(audio_gen::Waveform),

    // Filter
    FilterType(FilterKind),
    FilterCutoff(f32),
    FilterResonance(f32),
    FilterMix(f32),

    // Envelope (legacy simple variants)
    EnvelopeAttack(f32),
    EnvelopeDecay(f32),
    EnvelopeSustain(f32),
    EnvelopeRelease(f32),

    // Delay
    DelayMix(f32),
    DelayDecay(f32),
    DelayIntervalMs(f32),
    DelayDurationMs(f32),
    DelayNumRepeats(usize),

    // Flanger
    FlangerDelayMs(f32),
    FlangerDepthMs(f32),
    FlangerRateHz(f32),
    FlangerMix(f32),
    FlangerFeedback(f32),

    // LFO
    LfoFrequency(f32),
    LfoAmplitude(f32),

    // Tremolo
    TremoloModFreq(f32),
    TremoloModDepth(f32),

    // Vibrato
    VibratoAvgDelay(f32),
    VibratoModWidth(f32),
    VibratoModFreq(f32),

    // Chorus
    ChorusCount(usize),
    ChorusDryGain(f32),
    ChorusVoiceGain { voice: usize, gain: f32 },
    ChorusVoiceDelay { voice: usize, delay: f32 },

    // Equalizer
    EqualizerBandGain { band: usize, gain_db: f32 },
    EqualizerBandFreq { band: usize, freq: f32 },

    // Sequencer
    SequencerStep { track: u8, step: u8, enabled: bool },

    // Track
    TrackVolume { track: u8, volume: f32 },
    TrackPan { track: u8, pan: f32 },
    TrackMute { track: u8, muted: bool },

    // Transport
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
    WaveformData(Box<[f32; 512]>),
    SpectrumData(Box<[f32; 256]>),
    StepAdvance(usize),
}

pub struct AudioBridge {
    // Parameter update channel (UI → Audio)
    param_producer: HeapProducer<ParameterUpdate>,
    param_consumer: HeapConsumer<ParameterUpdate>,
    
    // Audio feedback channel (Audio → UI)
    feedback_producer: HeapProducer<AudioFeedback>,
    feedback_consumer: HeapConsumer<AudioFeedback>,
    
    // Shared atomic parameters for high-frequency updates
    oscillator_freq: Arc<AtomicF32>,
    filter_cutoff: Arc<AtomicF32>,
    master_volume: Arc<AtomicF32>,
}

impl AudioBridge {
    pub fn new() -> Result<Self, TuiError> {
        println!("Creating parameter ring buffer...");
        // Create ring buffers for lock-free communication
        let param_rb = HeapRb::<ParameterUpdate>::new(1024);
        let (param_producer, param_consumer) = param_rb.split();
        
        println!("Creating feedback ring buffer...");
        let feedback_rb = HeapRb::<AudioFeedback>::new(1024);
        let (feedback_producer, feedback_consumer) = feedback_rb.split();
        
        println!("Creating atomic floats...");
        let oscillator_freq = Arc::new(AtomicF32::new(440.0));
        let filter_cutoff = Arc::new(AtomicF32::new(8000.0));
        let master_volume = Arc::new(AtomicF32::new(0.75));
        
        println!("Constructing AudioBridge struct...");
        Ok(Self {
            param_producer,
            param_consumer,
            feedback_producer,
            feedback_consumer,
            oscillator_freq,
            filter_cutoff,
            master_volume,
        })
    }
    
    pub fn send_parameter_update(&mut self, update: ParameterUpdate) -> Result<(), TuiError> {
        // Handle high-frequency parameters via atomics
        match &update {
            ParameterUpdate::OscillatorFrequency(freq) => {
                self.oscillator_freq.store(*freq, Ordering::Relaxed);
            }
            ParameterUpdate::FilterCutoff(cutoff) => {
                self.filter_cutoff.store(*cutoff, Ordering::Relaxed);
            }
            _ => {
                // Send other parameters via ring buffer
                if self.param_producer.push(update).is_err() {
                    return Err(TuiError::Audio("Parameter update buffer full".to_string()));
                }
            }
        }
        Ok(())
    }
    
    pub fn receive_parameter_updates(&mut self) -> Vec<ParameterUpdate> {
        let mut updates = Vec::new();
        while let Some(update) = self.param_consumer.pop() {
            updates.push(update);
        }
        updates
    }
    
    pub fn send_audio_feedback(&mut self, feedback: AudioFeedback) -> Result<(), TuiError> {
        if self.feedback_producer.push(feedback).is_err() {
            return Err(TuiError::Audio("Feedback buffer full".to_string()));
        }
        Ok(())
    }
    
    pub fn receive_audio_feedback(&mut self) -> Vec<AudioFeedback> {
        let mut feedback = Vec::new();
        while let Some(fb) = self.feedback_consumer.pop() {
            feedback.push(fb);
        }
        feedback
    }
    
    pub fn get_oscillator_frequency(&self) -> f32 {
        self.oscillator_freq.load(Ordering::Relaxed)
    }
    
    pub fn get_filter_cutoff(&self) -> f32 {
        self.filter_cutoff.load(Ordering::Relaxed)
    }
    
    pub fn get_master_volume(&self) -> f32 {
        self.master_volume.load(Ordering::Relaxed)
    }
    
    pub fn get_atomic_refs(&self) -> (Arc<AtomicF32>, Arc<AtomicF32>, Arc<AtomicF32>) {
        (
            Arc::clone(&self.oscillator_freq),
            Arc::clone(&self.filter_cutoff),
            Arc::clone(&self.master_volume),
        )
    }
}