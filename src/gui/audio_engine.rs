use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{HeapConsumer, HeapProducer};

use crate::audio_gen::oscillator::{self, OscillatorTables, Waveform};
use crate::common::constants::SAMPLE_RATE;
use crate::note::constants::PITCH_TO_FREQ_HZ;
use crate::effect::chorus::ChorusBuilder;
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
const TABLE_SIZE: usize = 1024; // Must match oscillator wavetable size

fn table_lookup(table: &[f32], phase: f64) -> f32 {
    let index = (phase as usize) % TABLE_SIZE;
    let frac = phase.fract() as f32;
    let next_index = (index + 1) % TABLE_SIZE;
    table[index] + frac * (table[next_index] - table[index])
}

// --- Feedback delay line for real-time processing ---

struct DelayLine {
    buffer: Vec<f32>,
    write_pos: usize,
    delay_samples: usize,
    feedback: f32,
    mix: f32,
}

impl DelayLine {
    fn new(delay_ms: f32, feedback: f32, mix: f32) -> Self {
        // Max buffer ~2 seconds at 44.1kHz
        let max_samples = (SAMPLE_RATE * 2.0) as usize;
        let delay_samples = ((delay_ms / 1000.0) * SAMPLE_RATE).round() as usize;
        let buf_size = delay_samples.max(1).min(max_samples);
        Self {
            buffer: vec![0.0; buf_size],
            write_pos: 0,
            delay_samples: delay_samples.min(buf_size),
            feedback: feedback.clamp(0.0, 0.95),
            mix: mix.clamp(0.0, 1.0),
        }
    }

    fn process_sample(&mut self, input: f32) -> f32 {
        let buf_len = self.buffer.len();
        let read_pos = (self.write_pos + buf_len - self.delay_samples) % buf_len;
        let delayed = self.buffer[read_pos];
        self.buffer[self.write_pos] = input + self.feedback * delayed;
        self.write_pos = (self.write_pos + 1) % buf_len;
        input + self.mix * delayed
    }

    fn update_params(&mut self, delay_ms: f32, feedback: f32, mix: f32) {
        let max_samples = (SAMPLE_RATE * 2.0) as usize;
        let new_delay_samples = ((delay_ms / 1000.0) * SAMPLE_RATE).round() as usize;
        let new_delay_samples = new_delay_samples.max(1).min(max_samples);

        // Grow buffer if needed, preserving existing content
        if new_delay_samples > self.buffer.len() {
            self.buffer.resize(new_delay_samples, 0.0);
        }
        self.delay_samples = new_delay_samples.min(self.buffer.len());
        self.feedback = feedback.clamp(0.0, 0.95);
        self.mix = mix.clamp(0.0, 1.0);
    }

    fn delay_ms(&self) -> f32 {
        (self.delay_samples as f32 / SAMPLE_RATE) * 1000.0
    }
}

// --- Effect chain types for real-time processing ---

enum LiveEffect {
    Lfo(LFO),
    Tremolo(crate::effect::tremolo::Tremolo),
    Vibrato(crate::effect::vibrato::Vibrato),
    Flanger(crate::effect::flanger::Flanger),
    Chorus(crate::effect::chorus::Chorus),
    Delay(DelayLine),
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
            LiveEffect::Delay(d) => d.process_sample(sample),
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
                let d = DelayLine::new(s.interval_ms, s.decay, s.mix);
                (LiveEffect::Delay(d), s.enabled)
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
    level: f32,
}

impl Default for ChainVoice {
    fn default() -> Self {
        Self {
            waveforms: Vec::new(),
            level: 1.0,
        }
    }
}

struct TrackState {
    steps: [bool; NUM_STEPS],
    velocities: [f32; NUM_STEPS],
    pitches: [u8; NUM_STEPS],
    volume: f32,
    pan: f32,
    mute: bool,
    solo: bool,
    octave: u8,
}

impl Default for TrackState {
    fn default() -> Self {
        Self {
            steps: [false; NUM_STEPS],
            velocities: [0.8; NUM_STEPS],
            pitches: [0; NUM_STEPS],
            volume: 0.8,
            pan: 0.0,
            mute: false,
            solo: false,
            octave: 3,
        }
    }
}

struct NoteState {
    active: bool,
    sample_start: u64,
    step_duration_samples: u64,
    frequency: f32,
}

impl Default for NoteState {
    fn default() -> Self {
        Self {
            active: false,
            sample_start: 0,
            step_duration_samples: 0,
            frequency: 0.0,
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

/// Number of samples for the global fade-out when transport stops (~5.8ms at 44.1kHz).
const STOP_FADE_SAMPLES: u64 = 256;

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
    track_phases: [f64; NUM_TRACKS],

    /// When > 0, a stop-fade is in progress. Counts down from STOP_FADE_SAMPLES to 0.
    stop_fade_remaining: u64,
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
            track_phases: [0.0; NUM_TRACKS],
            stop_fade_remaining: 0,
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
    /// Includes an anti-click fade at the end to guarantee a smooth transition to zero,
    /// preventing clicks caused by the note being replaced before its final sample renders.
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

        let sustain_level = 0.7_f32;

        let base = if t < atk {
            // Attack: ramp 0 -> 1
            if atk > 0.0 { t / atk } else { 1.0 }
        } else if t < dec {
            // Decay: ramp from 1.0 down to sustain level
            let decay_len = dec - atk;
            if decay_len > 0.0 {
                let decay_t = (t - atk) / decay_len;
                1.0 + (sustain_level - 1.0) * decay_t
            } else {
                sustain_level
            }
        } else if t < sus {
            // Sustain: hold at sustain level
            sustain_level
        } else {
            // Release: ramp from sustain level down to 0
            let release_len = 1.0 - sus;
            if release_len > 0.0 {
                let release_t = (t - sus) / release_len;
                sustain_level * (1.0 - release_t)
            } else {
                0.0
            }
        };

        // Anti-click fade: guarantee the signal reaches zero at the note boundary.
        // The step transition replaces the note before its final t=1.0 frame renders,
        // leaving a residual amplitude that hard-cuts to zero. This fade smoothly
        // brings the output to zero over the last ~5.8ms (256 samples at 44.1kHz),
        // capped at 25% of step duration to avoid dominating short notes.
        const ANTI_CLICK_SAMPLES: f32 = 256.0;
        let fade_window = ANTI_CLICK_SAMPLES.min(duration * 0.25);
        let remaining = (duration - elapsed).max(0.0);
        let end_fade = if remaining < fade_window {
            remaining / fade_window
        } else {
            1.0
        };

        base * end_fade
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
                    self.track_phases = [0.0; NUM_TRACKS];
                    // Activate notes for step 0
                    self.activate_notes_for_current_step();
                }
                ParameterUpdate::TransportStop => {
                    // Start a fade-out instead of hard-cutting to prevent clicks
                    self.stop_fade_remaining = STOP_FADE_SAMPLES;
                    self.current_step = 0;
                }
                ParameterUpdate::TempoChange(t) => {
                    self.tempo = t;
                }
                ParameterUpdate::OscillatorChainUpdate { chain, oscillators } => {
                    if (chain as usize) < NUM_CHAINS {
                        self.chains[chain as usize].waveforms = oscillators;
                    }
                }
                ParameterUpdate::OscillatorChainFrequency { .. } => {
                    // Frequency is now per-step via pitch+octave, ignore legacy updates
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
                ParameterUpdate::SequencerStepPitch { track, step, pitch } => {
                    if (track as usize) < NUM_TRACKS && (step as usize) < NUM_STEPS {
                        self.tracks[track as usize].pitches[step as usize] = pitch;
                    }
                }
                ParameterUpdate::TrackOctave { track, octave } => {
                    if (track as usize) < NUM_TRACKS {
                        self.tracks[track as usize].octave = octave;
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
                        let (new_effects, new_enabled) = build_live_effects(&instances);
                        let ec = &mut self.effect_chains[chain as usize];
                        let new_len = new_effects.len();

                        for (i, new_effect) in new_effects.into_iter().enumerate() {
                            if i < ec.effects.len() {
                                // Preserve delay line buffer when only params changed
                                if let (LiveEffect::Delay(existing), LiveEffect::Delay(ref new_dl)) =
                                    (&mut ec.effects[i], &new_effect)
                                {
                                    existing.update_params(new_dl.delay_ms(), new_dl.feedback, new_dl.mix);
                                    ec.enabled[i] = new_enabled[i];
                                    continue;
                                }
                                ec.effects[i] = new_effect;
                                ec.enabled[i] = new_enabled[i];
                            } else {
                                ec.effects.push(new_effect);
                                ec.enabled.push(new_enabled[i]);
                            }
                        }
                        ec.effects.truncate(new_len);
                        ec.enabled.truncate(new_len);
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
                let pitch = self.tracks[track_idx].pitches[self.current_step];
                let octave = self.tracks[track_idx].octave;
                let freq_idx = (octave as usize) * 12 + (pitch as usize);
                let frequency = PITCH_TO_FREQ_HZ[freq_idx.min(127)] as f32;
                self.active_notes[track_idx] = NoteState {
                    active: true,
                    sample_start: self.sample_clock,
                    step_duration_samples: step_samples,
                    frequency,
                };
            } else {
                self.active_notes[track_idx].active = false;
            }
        }
    }

    /// Generate one stereo frame of audio.
    fn generate_frame(&mut self, feedback: &mut HeapProducer<AudioFeedback>) -> (f32, f32) {
        // Handle stop-fade: continue rendering with a linear fade-out, then go silent
        if self.stop_fade_remaining > 0 {
            self.stop_fade_remaining -= 1;
            if self.stop_fade_remaining == 0 {
                // Fade complete — fully stop playback and deactivate notes
                self.is_playing = false;
                for note in self.active_notes.iter_mut() {
                    note.active = false;
                }
                self.sample_clock += 1;
                return (0.0, 0.0);
            }
            // Fall through to normal rendering; apply stop_fade_gain at the end
        }

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
                // Still process effect chains with silence so delay echo tails ring out
                let sample_clock = self.sample_clock;
                let ec = &mut self.effect_chains[track_idx];
                if !ec.effects.is_empty() && ec.dry_wet > 0.0 {
                    let mut wet_sample = 0.0_f32;
                    for (i, effect) in ec.effects.iter_mut().enumerate() {
                        if ec.enabled[i] {
                            wet_sample = effect.process_sample(wet_sample, sample_clock);
                        }
                    }
                    let tail = wet_sample * ec.dry_wet;
                    if tail.abs() > 1e-7 {
                        let track_vol = self.tracks[track_idx].volume;
                        let chain_level = self.chains[track_idx].level;
                        let amplitude = tail * track_vol * chain_level * self.master_volume;
                        let pan = self.tracks[track_idx].pan;
                        let left_gain = ((1.0 - pan) * 0.5 + 0.5).min(1.0);
                        let right_gain = ((1.0 + pan) * 0.5).min(1.0);
                        left += amplitude * left_gain;
                        right += amplitude * right_gain;
                    }
                }
                continue;
            }

            let freq = note.frequency;

            let chain = &self.chains[track_idx];
            if chain.waveforms.is_empty() {
                continue;
            }

            // Use running phase accumulator for continuous waveform across notes.
            // Phase advances by freq * TABLE_SIZE / SAMPLE_RATE each sample,
            // so frequency changes between steps produce no phase discontinuity.
            let phase = self.track_phases[track_idx];

            // Generate sample from chain oscillators (additive mix)
            let mut sample = 0.0_f32;
            let osc_count = chain.waveforms.len() as f32;
            for waveform in &chain.waveforms {
                let osc_sample = match waveform {
                    Waveform::Sine => table_lookup(&self.osc_tables.sine_table, phase),
                    Waveform::Saw => table_lookup(&self.osc_tables.saw_table, phase),
                    Waveform::Square => table_lookup(&self.osc_tables.square_table, phase),
                    Waveform::Triangle => table_lookup(&self.osc_tables.triangle_table, phase),
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

            // Advance phase accumulator
            let delta = freq as f64 * TABLE_SIZE as f64 / SAMPLE_RATE as f64;
            self.track_phases[track_idx] = (self.track_phases[track_idx] + delta) % TABLE_SIZE as f64;

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

        // Apply stop-fade gain if transport is stopping
        if self.stop_fade_remaining > 0 {
            let fade_gain = self.stop_fade_remaining as f32 / STOP_FADE_SAMPLES as f32;
            left *= fade_gain;
            right *= fade_gain;
        }

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
