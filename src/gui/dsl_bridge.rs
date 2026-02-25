use crate::dsl::parser::parse_dsl;

use super::effects::*;
use super::envelope::EnvelopesState;
use super::sequencer::{SequencerState, NUM_STEPS, NUM_TRACKS};

/// Result of loading a DSL file into GUI state.
pub struct DslLoadResult {
    pub envelopes: EnvelopesState,
    pub effects: EffectsRackState,
    pub equalizers: EqualizersState,
    pub sequencer: SequencerState,
    pub tempo: f32,
    pub status: String,
}

/// Load a .dsl file and extract GUI state from it.
pub fn load_dsl_file(path: &str) -> Result<DslLoadResult, String> {
    let contents = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    load_dsl_string(&contents)
}

/// Load DSL from a string.
fn load_dsl_string(input: &str) -> Result<DslLoadResult, String> {
    let track_grid = parse_dsl(input)?;

    if track_grid.tracks.is_empty() {
        return Err("DSL file contains no tracks".to_string());
    }

    let mut envelopes = EnvelopesState::default();
    let mut effects = EffectsRackState::default();
    let equalizers = EqualizersState::default();
    let mut sequencer = SequencerState::default();

    // Extract tempo from first track's sequence
    let first_track = &track_grid.tracks[0];
    let tempo = first_track.sequence.tempo as f32;
    let step_duration_ms = 60000.0 / tempo; // quarter note duration

    // Extract per-track envelopes
    for (track_idx, track) in track_grid.tracks.iter().enumerate() {
        if track_idx >= NUM_TRACKS {
            break;
        }
        if let Some(env) = track.effects.envelopes.first() {
            envelopes.envelopes[track_idx].attack = (env.attack.0, env.attack.1);
            envelopes.envelopes[track_idx].decay = (env.decay.0, env.decay.1);
            envelopes.envelopes[track_idx].sustain = (env.sustain.0, env.sustain.1);
        }
    }

    // Extract effects from first track
    let fx = &first_track.effects;

    // Map delay
    if let Some(delay) = fx.delays.first() {
        effects.delay.enabled = true;
        effects.delay.mix = delay.mix;
        effects.delay.decay = delay.decay;
        effects.delay.interval_ms = delay.interval_ms;
        effects.delay.duration_ms = delay.duration_ms;
        effects.delay.num_repeats = delay.num_repeats;
    }

    // Map flanger
    if let Some(flanger) = fx.flangers.first() {
        effects.flanger.enabled = true;
        effects.flanger.delay_ms = flanger.delay_ms;
        effects.flanger.depth_ms = flanger.depth_ms;
        effects.flanger.rate_hz = flanger.rate_hz;
        effects.flanger.mix = flanger.mix;
        effects.flanger.feedback = flanger.feedback;
    }

    // Map LFO
    if let Some(lfo) = fx.lfos.first() {
        effects.lfo.enabled = true;
        effects.lfo.frequency = lfo.frequency;
        effects.lfo.amplitude = lfo.amplitude;
    }

    // Map tremolo
    if let Some(tremolo) = fx.tremolos.first() {
        effects.tremolo.enabled = true;
        effects.tremolo.mod_freq = tremolo.mod_freq;
        effects.tremolo.mod_depth = tremolo.mod_depth;
    }

    // Map vibrato
    if let Some(vibrato) = fx.vibratos.first() {
        effects.vibrato.enabled = true;
        effects.vibrato.avg_delay = vibrato.avg_delay;
        effects.vibrato.mod_width = vibrato.mod_width;
        effects.vibrato.mod_freq = vibrato.mod_freq;
    }

    // Map notes to sequencer grid
    // Compute step index from playback_start_time_ms / step_duration_ms
    for (track_idx, track) in track_grid.tracks.iter().enumerate() {
        if track_idx >= NUM_TRACKS {
            break;
        }
        let all_notes = track.sequence.get_all_notes();
        for note in &all_notes {
            let step = (note.playback_start_time_ms / step_duration_ms).round() as usize;
            if step < NUM_STEPS {
                sequencer.tracks[track_idx].steps[step].enabled = true;
                // Extract volume from the note source
                let vol = match &note.note_source {
                    crate::note::playback_note::NoteSource::Oscillator(n) => n.volume,
                    crate::note::playback_note::NoteSource::Sample(s) => s.volume,
                };
                sequencer.tracks[track_idx].steps[step].velocity = vol.min(1.0);
            }
        }
    }

    let num_tracks = track_grid.tracks.len().min(NUM_TRACKS);
    let status = format!("Loaded {} tracks from DSL", num_tracks);

    Ok(DslLoadResult {
        envelopes,
        effects,
        equalizers,
        sequencer,
        tempo,
        status,
    })
}

/// Export current GUI state as a DSL string.
pub fn export_dsl_string(
    envelopes: &EnvelopesState,
    effects: &EffectsRackState,
    equalizers: &EqualizersState,
    sequencer: &SequencerState,
    tempo: f32,
) -> String {
    let mut out = String::new();

    // For each track that has at least one enabled step, produce an outer block
    for (track_idx, track) in sequencer.tracks.iter().enumerate() {
        let has_steps = track.steps.iter().any(|s| s.enabled);
        if !has_steps {
            continue;
        }

        out.push_str("{\n");

        // Sequence definition
        out.push_str(&format!(
            "  seq quarter {} {}\n",
            tempo as u8,
            super::sequencer::NUM_STEPS
        ));

        // Per-track envelope
        if track_idx < 8 {
            let env = &envelopes.envelopes[track_idx];
            out.push_str(&format!(
                "  env a {:.2},{:.2} d {:.2},{:.2} s {:.2},{:.2} r 1.0,0.0\n",
                env.attack.0, env.attack.1,
                env.decay.0, env.decay.1,
                env.sustain.0, env.sustain.1,
            ));
        }

        // Effects (only on first track)
        if track_idx == 0 {
            if effects.delay.enabled {
                out.push_str(&format!(
                    "  delay mix {:.2} decay {:.2} interval {:.1} duration {:.1} repeats {}\n",
                    effects.delay.mix, effects.delay.decay,
                    effects.delay.interval_ms, effects.delay.duration_ms,
                    effects.delay.num_repeats
                ));
            }
            if effects.flanger.enabled {
                out.push_str(&format!(
                    "  flanger delay {:.1} depth {:.1} rate {:.2} mix {:.2} feedback {:.2}\n",
                    effects.flanger.delay_ms, effects.flanger.depth_ms,
                    effects.flanger.rate_hz, effects.flanger.mix, effects.flanger.feedback
                ));
            }
            if effects.lfo.enabled {
                out.push_str(&format!(
                    "  lfo freq {:.1} amp {:.2}\n",
                    effects.lfo.frequency, effects.lfo.amplitude
                ));
            }
            if effects.tremolo.enabled {
                out.push_str(&format!(
                    "  tremolo mod_freq {:.1} mod_depth {:.2}\n",
                    effects.tremolo.mod_freq, effects.tremolo.mod_depth
                ));
            }
            if effects.vibrato.enabled {
                out.push_str(&format!(
                    "  vibrato avg_delay {:.4} mod_width {:.4} mod_freq {:.1}\n",
                    effects.vibrato.avg_delay, effects.vibrato.mod_width, effects.vibrato.mod_freq
                ));
            }
        }

        // Per-track equalizer
        if track_idx < 8 && equalizers.equalizers[track_idx].enabled {
            let gains: Vec<String> = equalizers.equalizers[track_idx].gains
                .iter().map(|g| format!("{:.1}", g)).collect();
            out.push_str(&format!("  equalizer gains {}\n", gains.join(",")));
        }

        // Note declarations for enabled steps
        for (step_idx, step) in track.steps.iter().enumerate() {
            if step.enabled {
                out.push_str(&format!(
                    "  osc sine 440.0 {:.2} {}\n",
                    step.velocity, step_idx
                ));
            }
        }

        out.push_str("}\n\n");
    }

    out
}
