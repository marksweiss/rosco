use crate::audio_gen::oscillator;
use crate::audio_gen::oscillator::{get_gaussian_noise_sample, OscillatorTables};
use crate::audio_gen::oscillator::Waveform;
use crate::common::constants::NYQUIST_FREQUENCY;
// khz samples per second
use crate::note::playback_note::{NoteType, PlaybackNote};

pub(crate) fn get_note_sample(playback_note: &mut PlaybackNote, osc_tables: &OscillatorTables,
                              sample_position: f32, sample_count: u64) -> (f32, f32) {
    // Set to stereo output if either the note or the track is set to stereo
    let mut num_channels = playback_note.num_channels;
    if num_channels == 1 {
        num_channels = playback_note.track_effects.num_channels;
    }
    
    match playback_note.note_type {
        NoteType::Oscillator => {
            let mut sample = 0.0;
            for waveform in playback_note.note.waveforms.clone() {
                sample += match waveform {
                    Waveform::GaussianNoise => get_gaussian_noise_sample(),
                    Waveform::Saw => oscillator::get_sample(
                        &osc_tables.saw_table, playback_note.note.frequency, sample_count),
                    Waveform::Sine => oscillator::get_sample(
                        &osc_tables.sine_table, playback_note.note.frequency, sample_count),
                    Waveform::Square => oscillator::get_sample(
                        &osc_tables.square_table, playback_note.note.frequency, sample_count),
                    Waveform::Triangle => oscillator::get_sample(
                        &osc_tables.triangle_table, playback_note.note.frequency, sample_count),
                }
            }

            match num_channels {
                1 => {
                    let sample = playback_note.apply_effects(
                        playback_note.note.volume * sample, sample_position, sample_count);
                    (sample, sample)
                }
                2 => {
                    playback_note.apply_effects_stereo(
                        playback_note.note.volume * sample, sample_position, sample_count)
                }
                _ => (0.0, 0.0)
            }
        }
        NoteType::Sample => {
            match num_channels {
                1 => {
                    let mut sample = playback_note.sampled_note.next_sample();
                    sample = playback_note.apply_effects(
                        playback_note.note_volume() * sample, sample_position, sample_count);
                    (sample, sample)
                }
                2 => {
                    let sample = playback_note.sampled_note.next_sample();
                    playback_note.apply_effects_stereo(
                        playback_note.note_volume() * sample, sample_position, sample_count)
                }
                _ => (0.0, 0.0)
            }
        }
    }
}

pub(crate) fn get_notes_sample(playback_notes: &mut Vec<PlaybackNote>,
                               oscillator_tables: &OscillatorTables,
                               sample_position: f32, sample_count: u64) -> (f32, f32) {
    let mut out_sample_l = 0.0;
    let mut out_sample_r = 0.0;
    for playback_note in playback_notes.iter_mut() {
        if sample_count > playback_note.playback_sample_end_time {
            continue;
        }
        let next_samples = get_note_sample(playback_note, oscillator_tables,
                                           sample_position, sample_count);
        out_sample_l += next_samples.0;
        out_sample_r += next_samples.1;
    }

    if out_sample_l >= NYQUIST_FREQUENCY {
        out_sample_l = NYQUIST_FREQUENCY - 1.0;
    } else if out_sample_l <= -NYQUIST_FREQUENCY {
        out_sample_l = -NYQUIST_FREQUENCY + 1.0;
    }
    if out_sample_r >= NYQUIST_FREQUENCY {
        out_sample_r = NYQUIST_FREQUENCY - 1.0;
    } else if out_sample_r <= -NYQUIST_FREQUENCY {
        out_sample_r = -NYQUIST_FREQUENCY + 1.0;
    }

    (out_sample_l, out_sample_r)
}

