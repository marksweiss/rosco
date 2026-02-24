use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{HeapConsumer, HeapProducer};

use crate::audio_gen::oscillator::{self, OscillatorTables, Waveform};
use crate::common::constants::SAMPLE_RATE;
use crate::tui::audio_bridge::{AudioFeedback, ParameterUpdate};

const NUM_CHAINS: usize = 8;
const NUM_TRACKS: usize = 8;
const NUM_STEPS: usize = 16;

// --- Voice / track state structs (live inside the cpal callback closure) ---

struct ChainVoice {
    waveforms: Vec<Waveform>,
    frequency: f32,
    level: f32,
}

impl Default for ChainVoice {
    fn default() -> Self {
        Self {
            waveforms: Vec::new(),
            frequency: 440.0,
            level: 1.0,
        }
    }
}

struct TrackState {
    steps: [bool; NUM_STEPS],
    velocities: [f32; NUM_STEPS],
    volume: f32,
    pan: f32,
    mute: bool,
    solo: bool,
}

impl Default for TrackState {
    fn default() -> Self {
        Self {
            steps: [false; NUM_STEPS],
            velocities: [0.8; NUM_STEPS],
            volume: 0.8,
            pan: 0.0,
            mute: false,
            solo: false,
        }
    }
}

struct NoteState {
    active: bool,
    sample_start: u64,
    step_duration_samples: u64,
}

impl Default for NoteState {
    fn default() -> Self {
        Self {
            active: false,
            sample_start: 0,
            step_duration_samples: 0,
        }
    }
}

struct EnvelopeParams {
    attack: f32,  // 0.0-1.0 position within step
    decay: f32,
    sustain: f32,
}

impl Default for EnvelopeParams {
    fn default() -> Self {
        Self {
            attack: 0.02,
            decay: 0.51,
            sustain: 0.98,
        }
    }
}

struct SynthState {
    is_playing: bool,
    tempo: f32,
    current_step: usize,
    sample_clock: u64,
    step_sample_counter: f32,

    chains: [ChainVoice; NUM_CHAINS],
    tracks: [TrackState; NUM_TRACKS],
    active_notes: [NoteState; NUM_TRACKS],

    envelope: EnvelopeParams,
    master_volume: f32,

    osc_tables: OscillatorTables,
}

impl SynthState {
    fn new() -> Self {
        // Default chain 0 has a Sine oscillator (matching GUI default)
        let mut chains: [ChainVoice; NUM_CHAINS] = Default::default();
        chains[0].waveforms.push(Waveform::Sine);

        Self {
            is_playing: false,
            tempo: 120.0,
            current_step: 0,
            sample_clock: 0,
            step_sample_counter: 0.0,
            chains,
            tracks: Default::default(),
            active_notes: Default::default(),
            envelope: EnvelopeParams::default(),
            master_volume: 0.75,
            osc_tables: OscillatorTables::new(),
        }
    }

    fn samples_per_step(&self) -> f32 {
        SAMPLE_RATE * 60.0 / self.tempo / 4.0
    }

    fn any_solo(&self) -> bool {
        self.tracks.iter().any(|t| t.solo)
    }

    fn track_audible(&self, track_idx: usize) -> bool {
        let track = &self.tracks[track_idx];
        if track.mute {
            return false;
        }
        if self.any_solo() && !track.solo {
            return false;
        }
        true
    }

    /// Compute a simple ADSR envelope value for a note.
    /// The envelope positions (attack, decay, sustain) are fractions of the step duration.
    fn envelope_value(&self, note: &NoteState) -> f32 {
        if !note.active || note.step_duration_samples == 0 {
            return 0.0;
        }
        let elapsed = self.sample_clock.saturating_sub(note.sample_start) as f32;
        let duration = note.step_duration_samples as f32;
        let t = (elapsed / duration).min(1.0); // 0.0 to 1.0 through the step

        let atk = self.envelope.attack;
        let dec = self.envelope.decay;
        let sus = self.envelope.sustain;

        if t < atk {
            // Attack: ramp 0 -> 1
            if atk > 0.0 { t / atk } else { 1.0 }
        } else if t < dec {
            // Decay: hold at 1.0 (peak) — decay point determines when sustain level begins
            1.0
        } else if t < sus {
            // Sustain: hold at sustain level (0.7 default)
            0.7
        } else {
            // Release: ramp from sustain level down to 0
            let release_len = 1.0 - sus;
            if release_len > 0.0 {
                let release_t = (t - sus) / release_len;
                0.7 * (1.0 - release_t)
            } else {
                0.0
            }
        }
    }

    fn drain_updates(&mut self, consumer: &mut HeapConsumer<ParameterUpdate>) {
        while let Some(update) = consumer.pop() {
            match update {
                ParameterUpdate::TransportPlay => {
                    self.is_playing = true;
                    self.step_sample_counter = 0.0;
                    self.current_step = 0;
                    self.sample_clock = 0;
                    // Activate notes for step 0
                    self.activate_notes_for_current_step();
                }
                ParameterUpdate::TransportStop => {
                    self.is_playing = false;
                    self.current_step = 0;
                    for note in self.active_notes.iter_mut() {
                        note.active = false;
                    }
                }
                ParameterUpdate::TempoChange(t) => {
                    self.tempo = t;
                }
                ParameterUpdate::OscillatorChainUpdate { chain, oscillators } => {
                    if (chain as usize) < NUM_CHAINS {
                        self.chains[chain as usize].waveforms = oscillators;
                    }
                }
                ParameterUpdate::OscillatorChainFrequency { chain, frequency } => {
                    if (chain as usize) < NUM_CHAINS {
                        self.chains[chain as usize].frequency = frequency;
                    }
                }
                ParameterUpdate::OscillatorChainLevel { chain, level } => {
                    if (chain as usize) < NUM_CHAINS {
                        self.chains[chain as usize].level = level;
                    }
                }
                ParameterUpdate::SequencerStep { track, step, enabled } => {
                    if (track as usize) < NUM_TRACKS && (step as usize) < NUM_STEPS {
                        self.tracks[track as usize].steps[step as usize] = enabled;
                    }
                }
                ParameterUpdate::TrackVolume { track, volume } => {
                    if (track as usize) < NUM_TRACKS {
                        self.tracks[track as usize].volume = volume;
                    }
                }
                ParameterUpdate::TrackPan { track, pan } => {
                    if (track as usize) < NUM_TRACKS {
                        self.tracks[track as usize].pan = pan;
                    }
                }
                ParameterUpdate::TrackMute { track, muted } => {
                    if (track as usize) < NUM_TRACKS {
                        self.tracks[track as usize].mute = muted;
                    }
                }
                ParameterUpdate::EnvelopeAttack(v) => {
                    self.envelope.attack = v;
                }
                ParameterUpdate::EnvelopeDecay(v) => {
                    self.envelope.decay = v;
                }
                ParameterUpdate::EnvelopeSustain(v) => {
                    self.envelope.sustain = v;
                }
                ParameterUpdate::OscillatorVolume(v) => {
                    self.master_volume = v;
                }
                // Ignored for now (effects not in audio path yet)
                _ => {}
            }
        }
    }

    fn activate_notes_for_current_step(&mut self) {
        let step_samples = self.samples_per_step() as u64;
        for track_idx in 0..NUM_TRACKS {
            if self.tracks[track_idx].steps[self.current_step] {
                self.active_notes[track_idx] = NoteState {
                    active: true,
                    sample_start: self.sample_clock,
                    step_duration_samples: step_samples,
                };
            } else {
                self.active_notes[track_idx].active = false;
            }
        }
    }

    /// Generate one stereo frame of audio.
    fn generate_frame(&mut self, feedback: &mut HeapProducer<AudioFeedback>) -> (f32, f32) {
        if !self.is_playing {
            self.sample_clock += 1;
            return (0.0, 0.0);
        }

        // Step advancement
        self.step_sample_counter += 1.0;
        let sps = self.samples_per_step();
        if self.step_sample_counter >= sps {
            self.step_sample_counter -= sps;
            self.current_step = (self.current_step + 1) % NUM_STEPS;
            // Send step advance feedback to GUI
            let _ = feedback.push(AudioFeedback::StepAdvance(self.current_step));
            // Activate notes for the new step
            self.activate_notes_for_current_step();
        }

        // Mix all tracks
        let mut left = 0.0_f32;
        let mut right = 0.0_f32;

        for track_idx in 0..NUM_TRACKS {
            if !self.track_audible(track_idx) {
                continue;
            }

            let note = &self.active_notes[track_idx];
            if !note.active {
                continue;
            }

            let chain = &self.chains[track_idx];
            if chain.waveforms.is_empty() {
                continue;
            }

            // Generate sample from chain oscillators (additive mix)
            let mut sample = 0.0_f32;
            let osc_count = chain.waveforms.len() as f32;
            for waveform in &chain.waveforms {
                let osc_sample = match waveform {
                    Waveform::Sine => {
                        oscillator::get_sample(
                            &self.osc_tables.sine_table,
                            chain.frequency,
                            self.sample_clock,
                        )
                    }
                    Waveform::Saw => {
                        oscillator::get_sample(
                            &self.osc_tables.saw_table,
                            chain.frequency,
                            self.sample_clock,
                        )
                    }
                    Waveform::Square => {
                        oscillator::get_sample(
                            &self.osc_tables.square_table,
                            chain.frequency,
                            self.sample_clock,
                        )
                    }
                    Waveform::Triangle => {
                        oscillator::get_sample(
                            &self.osc_tables.triangle_table,
                            chain.frequency,
                            self.sample_clock,
                        )
                    }
                    Waveform::Noise | Waveform::GaussianNoise => {
                        oscillator::get_gaussian_noise_sample()
                    }
                };
                sample += osc_sample;
            }
            // Normalize by oscillator count to prevent clipping
            if osc_count > 0.0 {
                sample /= osc_count;
            }

            // Apply envelope
            let env = self.envelope_value(note);

            // Apply velocity, track volume, chain level, envelope, master volume
            let velocity = self.tracks[track_idx].velocities[self.current_step];
            let track_vol = self.tracks[track_idx].volume;
            let amplitude = sample * velocity * track_vol * chain.level * env * self.master_volume;

            // Pan: -1.0 = full left, 0.0 = center, 1.0 = full right
            let pan = self.tracks[track_idx].pan;
            let left_gain = ((1.0 - pan) * 0.5 + 0.5).min(1.0);
            let right_gain = ((1.0 + pan) * 0.5).min(1.0); // equivalent: (pan * 0.5 + 0.5)

            left += amplitude * left_gain;
            right += amplitude * right_gain;
        }

        self.sample_clock += 1;

        // Clamp to prevent distortion
        (left.clamp(-1.0, 1.0), right.clamp(-1.0, 1.0))
    }
}

// --- AudioEngine: owns the cpal stream ---

pub struct AudioEngine {
    _stream: cpal::Stream,
}

impl AudioEngine {
    pub fn new(
        mut param_consumer: HeapConsumer<ParameterUpdate>,
        mut feedback_producer: HeapProducer<AudioFeedback>,
    ) -> Result<Self, String> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| "No audio output device available".to_string())?;
        let config = device
            .default_output_config()
            .map_err(|e| format!("Failed to get default output config: {}", e))?;

        let stream_config: cpal::StreamConfig = config.into();
        let channels = stream_config.channels as usize;

        let mut state = SynthState::new();

        let stream = device
            .build_output_stream(
                &stream_config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    // Drain parameter updates from the GUI
                    state.drain_updates(&mut param_consumer);

                    // Generate audio frame-by-frame
                    for frame in data.chunks_mut(channels) {
                        let (left, right) = state.generate_frame(&mut feedback_producer);
                        frame[0] = left;
                        if channels > 1 {
                            frame[1] = right;
                        }
                    }
                },
                |err| eprintln!("Audio stream error: {}", err),
                None,
            )
            .map_err(|e| format!("Failed to build audio stream: {}", e))?;

        stream
            .play()
            .map_err(|e| format!("Failed to start audio stream: {}", e))?;

        Ok(Self { _stream: stream })
    }
}
