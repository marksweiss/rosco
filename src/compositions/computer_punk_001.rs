use crate::audio_gen::oscillator;
use crate::audio_gen::oscillator::Waveform;
use crate::composition::comp_utils;
use crate::effect::delay::DelayBuilder;
use crate::effect::flanger::FlangerBuilder;
use crate::effect::lfo::LfoBuilder;
use crate::envelope::EnvelopeBuilder;
use crate::envelope::envelope_pair::EnvelopePair;
use crate::note::note_pool::NotePool;
use crate::note::playback_note::{PlaybackNote, PlaybackNoteBuilder};
use crate::note::sampled_note::{SampledNote, SampledNoteBuilder};
use crate::sequence::time_note_sequence::{TimeNoteSequence, TimeNoteSequenceBuilder};
use crate::track::track_grid::TrackGridBuilder;
use crate::track::track::TrackBuilder;

#[allow(dead_code)]
static COMPUTER_PUNK_VERSION: &str = "003";

#[allow(dead_code)]
pub(crate) fn play() {
    // Init
    println!("playing 'computer punk {}'\n", self::COMPUTER_PUNK_VERSION);

    let sampled_note_volume = 0.0009;
    let sampled_note_rev_volume = 0.0042 * 0.3;

    // Track Effect
    #[allow(unused_variables)]
    let delay = DelayBuilder::default()
        .id(0)
        .decay(0.5)
        .mix(0.95)
        .interval_ms(70.0)
        .duration_ms(100.0)
        .num_repeats(4)
        .num_predelay_samples(2)
        .num_concurrent_sample_managers(4)
        .build().unwrap();
    // Envelopes
    let short_envelope = EnvelopeBuilder::default()
        .attack(EnvelopePair(0.03, 0.92))
        .decay(EnvelopePair(0.1, 0.87))
        .sustain(EnvelopePair(0.96, 0.85))
        .build().unwrap();
    // Flangers
    let flanger = FlangerBuilder::default()
        .window_size(12)
        .mix(0.15)
        .build().unwrap();
    let flanger_2 = FlangerBuilder::default()
        .window_size(6)
        .mix(0.5)
        .build().unwrap();
    // LFOs
    let lfo = LfoBuilder::default()
        .waveforms(vec![Waveform::Sine])
        .frequency(110.0)
        .amplitude(0.5)
        .build().unwrap();
    // /Track Effects
    
    let note_pool_capacity = 100;
    let mut sampled_note_pool: NotePool<SampledNote> =
        NotePool::new::<SampledNoteBuilder>(note_pool_capacity);
    let mut playback_note_pool: NotePool<PlaybackNote> =
        NotePool::new::<PlaybackNoteBuilder>(note_pool_capacity);

    // Load Sample Notes and Tracks
    let start_time = 0.0;
    let mut piano_note_1 = comp_utils::build_sampled_playback_note(
        &mut sampled_note_pool,
        &mut playback_note_pool,
        "/Users/markweiss/Downloads/punk_computer/003/piano_note_1_clipped.wav",
        sampled_note_volume,
        start_time,
        vec![short_envelope],
        vec![flanger.clone()],
        vec![delay.clone()],
        vec![lfo.clone()],
    );

    let mut piano_note_1_rev = piano_note_1.clone();
    piano_note_1_rev.sampled_note.reverse();
    piano_note_1_rev.sampled_note.volume = sampled_note_rev_volume;
    piano_note_1_rev.flangers = vec![flanger.clone(), flanger_2.clone()];
    let reverse_delay = delay.clone();
    piano_note_1_rev.delays = vec![reverse_delay];

    // let mut guitar_note_1 = comp_utils::build_sampled_playback_note(
    //     &mut sampled_note_pool,
    //     &mut playback_note_pool,
    //     // "/Users/markweiss/Downloads/punk_computer/001/punk_computer_003_16bit.wav",
    //     "/Users/markweiss/Downloads/punk_computer/003/guitar_note_1.wav",
    //     sampled_note_volume,
    //     start_time,
    //     vec![short_envelope],
    //     vec![flanger_2.clone()],
    //     vec![delay.clone()],
    //     vec![lfo.clone()],
    // );

    // let reverse_guitar_delay = delay.clone();
    // let mut guitar_note_1_rev = guitar_note_1.clone();
    // guitar_note_1_rev.sampled_note.reverse();
    // guitar_note_1_rev.sampled_note.volume = sampled_note_rev_volume;
    // guitar_note_1_rev.flangers = vec![flanger.clone(),
    //                                                      flanger_2.clone(),
    //                                                      flanger.clone()];
    // guitar_note_1_rev.delays = vec![reverse_guitar_delay.clone()];
    
    let mut piano_rest_note = piano_note_1.clone();
    piano_rest_note.sampled_note.volume = 0.0;
    // let mut guitar_rest_note = guitar_note_1.clone();
    // guitar_rest_note.sampled_note.volume = 0.0;

    // Create Tracks and append initial notes
    let piano_sequence_1: TimeNoteSequence= TimeNoteSequenceBuilder::default().build().unwrap();
    let piano_sequence_2: TimeNoteSequence= TimeNoteSequenceBuilder::default().build().unwrap();
    let mut piano_track_1 = TrackBuilder::default()
        .sequence(piano_sequence_1)
        .build().unwrap();
    let mut piano_track_2 = TrackBuilder::default()
        .sequence(piano_sequence_2)
        .build().unwrap();
    // let guitar_sequence: TimeNoteSequence = TimeNoteSequenceBuilder::default().build().unwrap();
    // let mut guitar_track_1 = TrackBuilder::default()
    //     .sequence(guitar_sequence)
    //     .build().unwrap();

    fn adjust_note_start_end_time(note: &mut PlaybackNote, start_time: f32, note_dur: f32) -> PlaybackNote {
        note.set_note_start_time_ms(start_time);
        note.set_note_end_time_ms(start_time + note_dur);
        note.clone()
    }

    // Add additional notes to the sequence
    let note_dur = piano_note_1.sampled_note.duration_ms();
    piano_track_1.sequence.append_note(
        adjust_note_start_end_time(
            &mut piano_note_1, 0.0, note_dur));
    piano_track_2.sequence.append_note(
        adjust_note_start_end_time(
            &mut piano_note_1, 1.0, note_dur));
    // guitar_track_1.sequence.append_note(
    //     adjust_note_start_end_time(
    //         &mut guitar_note_1, 0.0, note_dur));
    piano_track_1.sequence.append_note(
        adjust_note_start_end_time(
            &mut piano_rest_note, 1.0 * note_dur, note_dur));
    piano_track_2.sequence.append_note(
        adjust_note_start_end_time(
            &mut piano_note_1, (1.0 * note_dur) + 0.5, note_dur));
    // guitar_track_1.sequence.append_note(
    //     adjust_note_start_end_time(
    //         &mut guitar_note_1_rev, 1.0 * note_dur, note_dur));
    piano_track_1.sequence.append_note(
        adjust_note_start_end_time(
            &mut piano_note_1, 2.0 * note_dur, note_dur));
    piano_track_2.sequence.append_note(
        adjust_note_start_end_time(
            &mut piano_rest_note, (2.0 * note_dur) + 0.5, note_dur));
    // guitar_track_1.sequence.append_note(
    //     adjust_note_start_end_time(
    //         &mut guitar_rest_note, 2.0 * note_dur, note_dur));
    piano_track_1.sequence.append_note(
        adjust_note_start_end_time(
            &mut piano_note_1_rev, 3.0 * note_dur, note_dur));
    piano_track_2.sequence.append_note(
        adjust_note_start_end_time(
            &mut piano_note_1, (3.0 * note_dur) + 1.0, note_dur));
    // guitar_track_1.sequence.append_note(
    //     adjust_note_start_end_time(
    //         &mut guitar_note_1_rev, 3.0 * note_dur, note_dur));

    let mut tracks = Vec::new();
    tracks.push(piano_track_1);
    tracks.push(piano_track_2);
    // tracks.push(guitar_track_1);

    // Load and play Track Grid
    let track_grid =
        TrackGridBuilder::<TimeNoteSequence>::default()
        .tracks(tracks)
        .build().unwrap();

    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for playback_notes in track_grid {
            tx.send(playback_notes).unwrap();
        }
    });

    for playback_notes in rx.iter() {
        crate::audio_gen::gen_notes_stream(playback_notes, oscillator::OscillatorTables::new());
    }
}
