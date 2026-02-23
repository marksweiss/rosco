use std::collections::HashMap;

use nodi::midly;
use nodi::midly::num::{u28, u4, u7, u15};

use crate::note::constants;
use crate::note::note::NoteBuilder;
use crate::note::playback_note::{NoteType, PlaybackNote, PlaybackNoteBuilder};
use crate::note::sampled_note::SampledNoteBuilder;
use crate::sequence::note_sequence_trait::{AppendNote, BuilderWrapper};
use crate::track::track::{Track, TrackBuilder};

#[allow(dead_code)]
pub(crate) static DEFAULT_BPM: u8 = 120;
#[allow(dead_code)]
static MSECS_PER_MIN: f32 = 60000.0;

// The MIDI standard doesn't support connecting NoteOn and NoteOff events, nor NoteOn events with
// > 0 velocity and NoteOn events on the same pitch with 0 velocity, which are treated as NoteOff.
// We are processing raw Midi events in a stream, so we can't do any better and can only validly
// process input which doesn't have overlapping notes on the same channel with the same pitch.
// So the natural key for a current note in NoteOn state waiting for the NoteOff is (channel, pitch)
#[allow(dead_code)]
#[derive(Debug, Eq, Hash, PartialEq)]
struct NoteKey {
    channel: u4,
    pitch: u7
}

pub(crate) fn midi_file_to_tracks<
    SequenceType: AppendNote + Clone,
    SequenceBuilderType: BuilderWrapper<SequenceType>
>
(file_name: &str, note_type: NoteType) -> Result<Vec<Track<SequenceType>>, crate::common::error::RoscoError> {

    let mut tracks: Vec<Track<SequenceType>> = Vec::new();
    let data = std::fs::read(file_name)?;
    let midi = midly::Smf::parse(&data)
        .map_err(|e| crate::common::error::RoscoError::Midi(format!("Failed to parse MIDI: {}", e)))?;

    // Map key is channel and pitch, so there can be more than one note in process on a channel
    //  but only one per pitch. This is a bug / limitation.
    let mut track_notes_map: HashMap<NoteKey, PlaybackNote>= HashMap::new();
    let mut track_sequence_map: HashMap<u4, SequenceType> = HashMap::new();

    let bpm = get_beats_per_minute(&midi);
    let ticks_per_beat = get_ticks_per_beat(&midi)?;
    let ticks_per_ms: f32 = get_ticks_per_ms(ticks_per_beat, bpm);
    let mut ticks_since_start: u28 = u28::from(0);
    for track in midi.tracks.iter() {
        for event in track.iter() {
            match event {
                // delta is the number of ticks since the last Midi event
                midly::TrackEvent { delta, kind} => {
                    ticks_since_start += *delta;

                    match kind {
                        midly::TrackEventKind::Midi { channel, message } => {
                            match message {
                                // 'key' is midi pitch 1..127
                                midly::MidiMessage::NoteOn { key, vel } => {
                                    let note_key = NoteKey {channel: *channel, pitch: *key};

                                    if *vel > u7::from(0) {
                                        // If we have never seen the channel before, init the state
                                        // of the map being used to collect events into sequences
                                        if !track_sequence_map.contains_key(channel) {
                                            track_sequence_map.insert(*channel,
                                                                      SequenceBuilderType::new());
                                        }
                                        // Update the current note for this channel.
                                        // - Capture the velocity from the NoteOn event, should
                                        //  be a value > 0 if it's a note meant to be heard
                                        // - Set the duration to 0 to start
                                        // - Capture the start time in ticks converted to msecs

                                        // TODO THIS SHOULD CLOSE THE OPEN NOTE AND OPEN A NEW ONE
                                        // Handle case of existing open note with same key by
                                        //  skipping this note if it is a duplicate
                                        if !track_notes_map.contains_key(&note_key) {
                                            let note_start_time_ms =
                                                ticks_since_start.as_int() as f32 / ticks_per_ms;
                                            match note_type {
                                                NoteType::Oscillator => { 
                                                    let note =
                                                        NoteBuilder::default().
                                                            frequency(
                                                                constants::PITCH_TO_FREQ_HZ[key.as_int() as usize] as f32)
                                                            .volume(vel.as_int() as f32 / 127.0f32)
                                                            .start_time_ms(note_start_time_ms)
                                                            .end_time_ms(note_start_time_ms)
                                                            .build().unwrap();

                                                    track_notes_map.insert(
                                                       note_key,
                                                       PlaybackNoteBuilder::default()
                                                           .note_type(note_type)
                                                           .note(note)
                                                           .playback_start_time_ms(note_start_time_ms)
                                                           .playback_end_time_ms(note_start_time_ms)
                                                           .build().unwrap());
                                                }
                                                NoteType::Sample => {
                                                    let sampled_note =
                                                        SampledNoteBuilder::default()
                                                            .volume(vel.as_int() as f32 / 127.0f32)
                                                            .start_time_ms(note_start_time_ms)
                                                            .end_time_ms(note_start_time_ms)
                                                            .build().unwrap();
                                                    track_notes_map.insert(
                                                        note_key,
                                                        PlaybackNoteBuilder::default()
                                                            .note_type(note_type)
                                                            .sampled_note(sampled_note)
                                                            .playback_start_time_ms(note_start_time_ms)
                                                            .playback_end_time_ms(note_start_time_ms)
                                                            .build().unwrap());
                                                    
                                                }
                                            }
                                        }
                                        // 0 volume for a note we got the start of previously
                                    } else if track_sequence_map.contains_key(channel) &&
                                        track_notes_map.contains_key(&note_key) {
                                        let ms_since_start =
                                            ticks_since_start.as_int() as f32 / ticks_per_ms;
                                        handle_note_off(note_key,
                                                        ms_since_start,
                                                        &mut track_notes_map,
                                                        &mut track_sequence_map);
                                    }
                                }

                                #[allow(unused_variables)]
                                midly::MidiMessage::NoteOff { key, vel } => {
                                    let note_key = NoteKey {channel: *channel, pitch: *key};
                                    let ms_since_start =
                                        ticks_since_start.as_int() as f32 / ticks_per_ms;
                                    handle_note_off(note_key,
                                                    ms_since_start,
                                                    &mut track_notes_map,
                                                    &mut track_sequence_map);
                                }

                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    for (midi_channel, sequence) in track_sequence_map.iter() {
        let track = TrackBuilder::default()
            .num(midi_channel.as_int() as i16)
            .sequence(sequence.clone())
            .volume(1.0 / track_sequence_map.len() as f32)
            .build()
            .map_err(|e| crate::common::error::RoscoError::Midi(format!("Failed to build Track: {:?}", e)))?;
        tracks.push(track);
    }

    Ok(tracks)
}

#[allow(dead_code)]
pub(crate) fn get_beats_per_minute(midi: &midly::Smf) -> u8 {
    for track in midi.tracks.iter() {
        for event in track.iter() {
            match event {
                #[allow(unused_variables)]
                midly::TrackEvent { delta, kind } => {
                    match kind {
                        midly::TrackEventKind::Meta(meta) => {
                            match meta {
                                midly::MetaMessage::Tempo(tempo) => {
                                    let normalized_tempo=
                                        if tempo.as_int() > 0 {tempo.as_int()} else {1};
                                    return (60000000 / normalized_tempo) as u8;
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    DEFAULT_BPM
}

#[allow(dead_code)]
pub(crate) fn get_ticks_per_beat(midi: &midly::Smf) -> Result<u15, crate::common::error::RoscoError> {
    let header = midi.header;

    match header.timing {
        midly::Timing::Metrical(ticks_per_beat) => Ok(ticks_per_beat),
        _ => Err(crate::common::error::RoscoError::Midi(
            "Only Metrical timing is supported".to_string()
        )),
    }
}

#[allow(dead_code)]
pub(crate) fn get_ticks_per_ms(ticks_per_beat: u15, beats_per_minute: u8) -> f32 {
    (ticks_per_beat.as_int() as f32 * beats_per_minute as f32) / MSECS_PER_MIN
}

#[allow(dead_code)]
fn handle_note_off<SequenceType: AppendNote>(note_key: NoteKey,
                                             ms_since_start: f32,
                                             track_notes_map: &mut HashMap<NoteKey, PlaybackNote>,
                                             track_sequence_map: &mut HashMap<u4, SequenceType>) {
    // Add the last tick delta to the note duration, copy the note to the output track sequence
    // and remove it from the current notes map
    let mut playback_note = track_notes_map.get_mut(&note_key).unwrap().clone();
    playback_note.set_note_end_time_ms(ms_since_start);
    playback_note.playback_end_time_ms = ms_since_start;
    track_sequence_map.get_mut(&note_key.channel).unwrap().append_note(playback_note);
    track_notes_map.remove(&note_key);
}
