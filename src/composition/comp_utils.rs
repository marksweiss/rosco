use crate::{audio_gen, common, midi, note};
use crate::audio_gen::audio_gen::gen_notes_stream;
use crate::audio_gen::oscillator::{OscillatorTables, Waveform};
use crate::effect::delay::Delay;
use crate::effect::flanger::Flanger;
use crate::effect::lfo::LFO;
use crate::envelope::envelope::Envelope;
use crate::note::playback_note::{NoteType, PlaybackNote};
use crate::sequence::note_sequence_trait::{AppendNote, AppendNotes, BuilderWrapper, IterMutWrapper,
    NextNotes, SetCurPosition};
use crate::track::track::{Track, TrackBuilder};
use crate::track::track_grid::TrackGrid;
use crate::note::note_pool::NotePool;
use crate::note::sampled_note::SampledNote;

const ARGS_DELIMITER: &str = ",";

#[allow(dead_code)]
pub(crate) struct SampleBuf {
    buf: Vec<f32>,
    len: usize,
}

pub(crate) fn build_sampled_playback_note(sampled_note_pool: &mut NotePool<SampledNote>,
                                          playback_note_pool: &mut NotePool<PlaybackNote>,
                                          file_path: &str, volume: f32, start_time: f32,
                                          envelopes: Vec<Envelope>, flangers: Vec<Flanger>,
                                          delays: Vec<Delay>, lfos: Vec<LFO>) -> PlaybackNote {
    let sample_buf: SampleBuf = load_sample_data(file_path);
    let mut sampled_note = sampled_note_pool.acquire().unwrap();
    sampled_note.volume = volume;
    sampled_note.start_time_ms = start_time;
    sampled_note.end_time_ms = (sample_buf.len as f32 / common::constants::SAMPLE_RATE) * 1000.0;
    sampled_note.set_sample_buf(&sample_buf.buf);

    let mut playback_note = playback_note_pool.acquire().unwrap();
    playback_note.note_type = NoteType::Sample;
    playback_note.sampled_note = sampled_note;
    playback_note.playback_start_time_ms = start_time;
    playback_note.playback_end_time_ms = start_time + ((sample_buf.len as f32 / common::constants::SAMPLE_RATE) * 1000.0);
    playback_note.playback_sample_start_time = start_time as u64;
    playback_note.playback_sample_end_time = sample_buf.len as u64;
    playback_note.envelopes = envelopes;
    playback_note.flangers = flangers;
    playback_note.delays = delays;
    playback_note.lfos = lfos;
    
    playback_note
}

pub(crate) fn load_sample_data(file_path: &str) -> SampleBuf {
    let sample_data= audio_gen::audio_gen::read_audio_file(file_path).into_boxed_slice();
    let mut sample_buf: Vec<f32> = Vec::with_capacity(note::sampled_note::BUF_STORAGE_SIZE);
    for sample in  sample_data[..].iter() {
        sample_buf.push(*sample as f32);
    }
    SampleBuf {
        buf: sample_buf,
        len: sample_data.len(),
    }
}

#[allow(dead_code)]
pub(crate) fn load_midi_file_to_tracks<
    SequenceType: AppendNote + Clone + IterMutWrapper,
    SequenceBuilderType: BuilderWrapper<SequenceType>
>
(file_path: &str, waveforms: Vec<Waveform>, envelopes: Vec<Envelope>, flangers: Vec<Flanger>,
 delays: Vec<Delay>, lfo: LFO, volume: f32) -> Vec<Track<SequenceType>> {
    let mut midi_time_tracks =
        midi::midi::midi_file_to_tracks::<SequenceType, SequenceBuilderType>(
            file_path, NoteType::Oscillator);

    for track in midi_time_tracks.iter_mut() {
        for playback_notes in track.sequence.iter_mut() {
            for playback_note in playback_notes {
                playback_note.note.waveforms = waveforms.clone();
                playback_note.note.volume = volume;
                playback_note.envelopes = envelopes.clone();
                playback_note.flangers = flangers.clone();
                playback_note.delays = delays.clone();
                playback_note.lfos = vec![lfo.clone()]
            }
        }
    }

    midi_time_tracks
}

#[allow(dead_code)]
pub(crate) fn load_note_to_new_track<
    SequenceType: AppendNote + Clone + IterMutWrapper,
    SequenceBuilderType: BuilderWrapper<SequenceType>
>
(mut playback_note: PlaybackNote, volume: f32) -> Track<SequenceType> {
    let mut sequence = SequenceBuilderType::new();
    // NOTE: generically modifies volume of BOTH underlying notes
    playback_note.sampled_note.volume = volume;
    playback_note.note.volume = volume;
    sequence.append_note(playback_note.clone());
    TrackBuilder::default()
        .sequence(sequence)
        .build().unwrap()
}

#[allow(dead_code)]
pub(crate) fn load_notes_to_new_track<
    SequenceType: AppendNotes + Clone + IterMutWrapper,
    SequenceBuilderType: BuilderWrapper<SequenceType>
>
(playback_notes: &mut Vec<PlaybackNote>, volume: f32) -> Track<SequenceType> {
    let mut sequence = SequenceBuilderType::new();
    for playback_note in playback_notes.iter_mut() {
        playback_note.sampled_note.volume = volume;
    }
    sequence.append_notes(&playback_notes.clone());
    TrackBuilder::default()
        .sequence(sequence)
        .build().unwrap()
}

#[allow(dead_code)]
pub(crate) fn set_notes_offset(playback_notes: &mut Vec<PlaybackNote>, offset: f32) {
    for playback_note in playback_notes.iter_mut() {
        playback_note.playback_start_time_ms += offset;
        playback_note.playback_end_time_ms += offset;
        playback_note.sampled_note.start_time_ms += offset;
        playback_note.sampled_note.end_time_ms += offset;

        if playback_note.note_type == NoteType::Oscillator{
            playback_note.note.start_time_ms += offset;
            playback_note.note.end_time_ms += offset;
        }
    }
}

pub(crate) fn collect_args () -> String {
    let mut waveforms_arg = String::from("sine");
    for (i, arg) in std::env::args().enumerate() {
        match i {
            // skip program name in 0th args position
            0 => continue,
            1 => waveforms_arg = arg,
            _ => break,
        }
    }

    waveforms_arg
}

#[allow(dead_code)]
pub(crate) fn get_waveforms_from_arg() -> Vec<Waveform> {
    collect_args().split(ARGS_DELIMITER)
        .map( |waveform| {
            let matched = match waveform {
                "gaussian_noise" => Waveform::GaussianNoise,
                "saw" => Waveform::Saw,
                "sine" => Waveform::Sine,
                "square" => Waveform::Square,
                "triangle" => Waveform::Triangle,
                _ => Waveform::Sine,
            };
            matched
        })
        .collect()
}

#[allow(dead_code)]
pub(crate) fn play_track_grid<SequenceType>(track_grid: TrackGrid<SequenceType>)
where
    // Add Send + 'static bounds to ensure thread safety
    SequenceType: NextNotes + Iterator + SetCurPosition + Send + 'static,
    // The Item of the iterator must also be Send to be sent across the channel
    <SequenceType as Iterator>::Item: Send,
{
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for playback_notes in track_grid {
            if tx.send(playback_notes).is_err() {
                // The receiver has hung up, so we can stop the thread.
                break;
            }
        }
    });

    for playback_notes in rx.iter() {
        gen_notes_stream(playback_notes, OscillatorTables::new());
    }
}