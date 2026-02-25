use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{HeapConsumer, HeapProducer};

use crate::audio_gen::oscillator::{self, OscillatorTables, Waveform};
use crate::common::constants::SAMPLE_RATE;
use crate::effect::chorus::ChorusBuilder;
use crate::effect::delay::DelayBuilder;
use crate::effect::equalizer::{Equalizer, default_equalizer};
use crate::effect::flanger::FlangerBuilder;
use crate::effect::lfo::{LFOBuilder, LFO};
use crate::effect::tremolo::TremoloBuilder;
use crate::effect::vibrato::VibratoBuilder;
use crate::filter::band_pass_filter::BandPassFilterBuilder;
use crate::filter::high_pass_filter::HighPassFilterBuilder;
use crate::filter::low_pass_filter::LowPassFilterBuilder;
use crate::filter::notch_filter::NotchFilterBuilder;
use crate::tui::audio_bridge::{AudioFeedback, FilterKind, ParameterUpdate};
use super::effect_chains::EffectInstance;

const NUM_CHAINS: usize = 8;
const NUM_TRACKS: usize = 8;
const NUM_STEPS: usize = 16;
const MAX_UPDATES_PER_CALLBACK: usize = 32;

// --- Effect chain types for real-time processing ---

enum LiveEffect {
    Lfo(LFO),
    Tremolo(crate::effect::tremolo::Tremolo),
    Vibrato(crate::effect::vibrato::Vibrato),
    Flanger(crate::effect::flanger::Flanger),
    Chorus(crate::effect::chorus::Chorus),
    Delay(crate::effect::delay::Delay),
    LowPass(crate::filter::low_pass_filter::LowPassFilter),
    HighPass(crate::filter::high_pass_filter::HighPassFilter),
    BandPass(crate::filter::band_pass_filter::BandPassFilter),
    Notch(crate::filter::notch_filter::NotchFilter),
}

impl LiveEffect {
    fn process_sample(&mut self, sample: f32, sample_clock: u64) -> f32 {
        match self {
            LiveEffect::Lfo(e) => e.apply_effect(sample, sample_clock),
            LiveEffect::Tremolo(e) => e.apply_effect(sample, sample_clock as f32),
            LiveEffect::Vibrato(e) => e.apply_effect(sample, sample_clock as f32),
            LiveEffect::Flanger(e) => e.apply_effect(sample, sample_clock as f32),
            LiveEffect::Chorus(e) => e.apply_effect(sample, sample_clock as f32),
            LiveEffect::Delay(e) => e.apply_effect(sample, sample_clock as f32),
            LiveEffect::LowPass(e) => e.apply_effect(sample, sample_clock as f32),
            LiveEffect::HighPass(e) => e.apply_effect(sample, sample_clock as f32),
            LiveEffect::BandPass(e) => e.apply_effect(sample, sample_clock as f32),
            LiveEffect::Notch(e) => e.apply_effect(sample, sample_clock as f32),
        }
    }
}

struct EffectChainState {
    effects: Vec<LiveEffect>,
    enabled: Vec<bool>,
    dry_wet: f32,
}

impl Default for EffectChainState {
    fn default() -> Self {
        Self {
            effects: Vec::new(),
            enabled: Vec::new(),
            dry_wet: 0.5,
        }
    }
}

fn build_live_effects(instances: &[EffectInstance]) -> (Vec<LiveEffect>, Vec<bool>) {
    let mut effects = Vec::new();
    let mut enabled = Vec::new();

    for inst in instances {
        let (effect, is_enabled) = match inst {
            EffectInstance::Tremolo(s) => {
                let e = TremoloBuilder::default()
                    .mod_freq(s.mod_freq)
                    .mod_depth(s.mod_depth)
                    .build()
                    .unwrap();
                (LiveEffect::Tremolo(e), s.enabled)
            }
            EffectInstance::Vibrato(s) => {
                let e = VibratoBuilder::default()
                    .avg_delay(s.avg_delay)
                    .mod_width(s.mod_width)
                    .mod_freq(s.mod_freq)
                    .build()
                    .unwrap();
                (LiveEffect::Vibrato(e), s.enabled)
            }
            EffectInstance::Flanger(s) => {
                let e = FlangerBuilder::default()
                    .delay_ms(s.delay_ms)
                    .depth_ms(s.depth_ms)
                    .rate_hz(s.rate_hz)
                    .mix(s.mix)
                    .feedback(s.feedback)
                    .build()
                    .unwrap();
                (LiveEffect::Flanger(e), s.enabled)
            }
            EffectInstance::Chorus(s) => {
                let count = s.chorus_count;
                // Generate default mod_freqs and mod_widths (not in GUI state)
                let default_mod_freqs = [0.25, 0.33, 0.40, 0.50, 0.60, 0.70];
                let default_mod_widths = [0.003, 0.004, 0.005, 0.003, 0.004, 0.005];
                let mod_freqs: Vec<f32> = default_mod_freqs.iter().copied().take(count).collect();
                let mod_widths: Vec<f32> = default_mod_widths.iter().copied().take(count).collect();

                let e = ChorusBuilder::default()
                    .chorus_count(count)
                    .dry_gain(s.dry_gain)
                    .chorus_gains(s.voice_gains.clone())
                    .chorus_delays(s.voice_delays.clone())
                    .mod_freqs(mod_freqs)
                    .mod_widths(mod_widths)
                    .build()
                    .unwrap();
                (LiveEffect::Chorus(e), s.enabled)
            }
            EffectInstance::Delay(s) => {
                let e = DelayBuilder::default()
                    .mix(s.mix)
                    .decay(s.decay)
                    .interval_ms(s.interval_ms)
                    .duration_ms(s.duration_ms)
                    .num_repeats(s.num_repeats)
                    .build()
                    .unwrap();
                (LiveEffect::Delay(e), s.enabled)
            }
            EffectInstance::Lfo(s) => {
                let freq = s.frequency.clamp(0.01, 22050.0);
                let e = LFOBuilder::default()
                    .frequency(freq)
                    .amplitude(s.amplitude)
                    .build()
                    .unwrap();
                (LiveEffect::Lfo(e), s.enabled)
            }
            EffectInstance::Filter(s) => {
                let effect = match s.kind {
                    FilterKind::LowPass => {
                        let e = LowPassFilterBuilder::default()
                            .cutoff_frequency(s.cutoff)
                            .resonance(s.resonance)
                            .mix(s.mix)
                            .build_with_coefficients()
                            .unwrap();
                        LiveEffect::LowPass(e)
                    }
                    FilterKind::HighPass => {
                        let e = HighPassFilterBuilder::default()
                            .cutoff_frequency(s.cutoff)
                            .resonance(s.resonance)
                            .mix(s.mix)
                            .build_with_coefficients()
                            .unwrap();
                        LiveEffect::HighPass(e)
                    }
                    FilterKind::BandPass => {
                        let e = BandPassFilterBuilder::default()
                            .center_frequency(s.cutoff)
                            .bandwidth(s.bandwidth)
                            .resonance(s.resonance)
                            .mix(s.mix)
                            .build_with_coefficients()
                            .unwrap();
                        LiveEffect::BandPass(e)
                    }
                    FilterKind::Notch => {
                        let e = NotchFilterBuilder::default()
                            .center_frequency(s.cutoff)
                            .bandwidth(s.bandwidth)
                            .resonance(s.resonance)
                            .mix(s.mix)
                            .build_with_coefficients()
                            .unwrap();
                        LiveEffect::Notch(e)
                    }
                };
                (effect, s.enabled)
            }
        };
        effects.push(effect);
        enabled.push(is_enabled);
    }

    (effects, enabled)
}

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

    envelopes: [EnvelopeParams; NUM_TRACKS],
    master_volume: f32,

    osc_tables: OscillatorTables,
    effect_chains: [EffectChainState; NUM_CHAINS],
    equalizers: [Equalizer; NUM_CHAINS],
    eq_enabled: [bool; NUM_CHAINS],
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
            envelopes: std::array::from_fn(|_| EnvelopeParams::default()),
            master_volume: 0.75,
            osc_tables: OscillatorTables::new(),
            effect_chains: Default::default(),
            equalizers: std::array::from_fn(|_| default_equalizer()),
            eq_enabled: [false; NUM_CHAINS],
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
    fn envelope_value(&self, note: &NoteState, track_idx: usize) -> f32 {
        if !note.active || note.step_duration_samples == 0 {
            return 0.0;
        }
        let elapsed = self.sample_clock.saturating_sub(note.sample_start) as f32;
        let duration = note.step_duration_samples as f32;
        let t = (elapsed / duration).min(1.0); // 0.0 to 1.0 through the step

        let atk = self.envelopes[track_idx].attack;
        let dec = self.envelopes[track_idx].decay;
        let sus = self.envelopes[track_idx].sustain;

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
        for _ in 0..MAX_UPDATES_PER_CALLBACK {
            let update = match consumer.pop() {
                Some(u) => u,
                None => break,
            };
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
                ParameterUpdate::EnvelopeAttack { track, value } => {
                    if (track as usize) < NUM_TRACKS {
                        self.envelopes[track as usize].attack = value;
                    }
                }
                ParameterUpdate::EnvelopeDecay { track, value } => {
                    if (track as usize) < NUM_TRACKS {
                        self.envelopes[track as usize].decay = value;
                    }
                }
                ParameterUpdate::EnvelopeSustain { track, value } => {
                    if (track as usize) < NUM_TRACKS {
                        self.envelopes[track as usize].sustain = value;
                    }
                }
                ParameterUpdate::OscillatorVolume(v) => {
                    self.master_volume = v;
                }
                ParameterUpdate::EffectChainUpdate { chain, instances } => {
                    if (chain as usize) < NUM_CHAINS {
                        let (effects, enabled) = build_live_effects(&instances);
                        self.effect_chains[chain as usize].effects = effects;
                        self.effect_chains[chain as usize].enabled = enabled;
                    }
                }
                ParameterUpdate::EffectChainDryWet { chain, dry_wet } => {
                    if (chain as usize) < NUM_CHAINS {
                        self.effect_chains[chain as usize].dry_wet = dry_wet;
                    }
                }
                ParameterUpdate::EqualizerBandGain { chain, band, gain_db } => {
                    if (chain as usize) < NUM_CHAINS {
                        self.equalizers[chain as usize].set_band_gain(band, gain_db);
                    }
                }
                ParameterUpdate::EqualizerBandFreq { chain, band, freq } => {
                    let _ = (chain, band, freq); // Future: update center frequency
                }
                ParameterUpdate::EqualizerEnabled { chain, enabled } => {
                    if (chain as usize) < NUM_CHAINS {
                        self.eq_enabled[chain as usize] = enabled;
                    }
                }
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
            let env = self.envelope_value(note, track_idx);
            sample *= env;

            // Apply effect chain for this track
            let sample_clock = self.sample_clock;
            let ec = &mut self.effect_chains[track_idx];
            if !ec.effects.is_empty() && ec.dry_wet > 0.0 {
                let dry_sample = sample;
                let mut wet_sample = sample;
                for (i, effect) in ec.effects.iter_mut().enumerate() {
                    if ec.enabled[i] {
                        wet_sample = effect.process_sample(wet_sample, sample_clock);
                    }
                }
                sample = dry_sample * (1.0 - ec.dry_wet) + wet_sample * ec.dry_wet;
            }

            // Apply per-chain equalizer
            if self.eq_enabled[track_idx] {
                sample = self.equalizers[track_idx].apply_effect(sample, self.sample_clock as f32);
            }

            // Apply velocity, track volume, chain level, master volume
            let velocity = self.tracks[track_idx].velocities[self.current_step];
            let track_vol = self.tracks[track_idx].volume;
            let amplitude = sample * velocity * track_vol * chain.level * self.master_volume;

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
