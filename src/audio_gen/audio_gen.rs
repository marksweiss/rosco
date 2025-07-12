use std::time;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::audio_gen::get_sample;
use crate::audio_gen::oscillator::OscillatorTables;
use crate::common::constants::SAMPLE_RATE;
use crate::note::playback_note::PlaybackNote;

// TODO SUPPORT LOFI AND 32-BIT
static WAV_SPEC: hound::WavSpec  = hound::WavSpec {
    channels: 1,
    sample_rate: SAMPLE_RATE as u32,
    bits_per_sample: 16,
    sample_format: hound::SampleFormat::Int,
};

#[allow(dead_code)]
pub(crate) fn gen_note_stream(playback_note: PlaybackNote, oscillator_tables: OscillatorTables) {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("No output device available");
    let config = device.default_output_config().unwrap();

    gen_note_stream_impl::<f32>(&device, &config.into(), oscillator_tables, playback_note);
}

#[allow(dead_code)]
pub(crate) fn gen_notes_stream(playback_notes: Vec<PlaybackNote>,
                               oscillator_tables: OscillatorTables)
{
    let host = cpal::default_host();
    let device = host.default_output_device().expect("No output device available");
    let config = device.default_output_config().unwrap();

    let window_start_time_ms = playback_notes.iter()
        .map(|playback_note| playback_note.playback_start_time_ms)
        .reduce(|a, b| a.min(b))
        .unwrap();
    let window_end_time_ms = playback_notes.iter()
        .map(|playback_note| playback_note.playback_end_time_ms)
        .reduce(|a, b| a.max(b))
        .unwrap();
    let window_duration_ms = (window_end_time_ms - window_start_time_ms).floor() as u64;
    
    gen_notes_stream_impl::<f32>(&device, &config.into(), oscillator_tables, playback_notes,
                                 window_duration_ms);
}

// TODO PARAMETERIZE SAMPLE TYPE TO SUPPORT LOFI AND 32-BIT
#[allow(dead_code)]
pub(crate) fn read_audio_file(file_path: &str) -> Vec<i16> {
    let mut reader = hound::WavReader::open(file_path).unwrap();
    let samples: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap()).collect();
    samples
}

// TODO PARAMETERIZE SAMPLE TYPE TO SUPPORT LOFI AND 32-BIT
#[allow(dead_code)]
pub(crate) fn write_audio_file(file_path: &str, samples: Vec<f32>) {
    let mut writer = hound::WavWriter::create(file_path, WAV_SPEC).unwrap();
    for sample in samples {
        writer.write_sample(sample.round() as i16).unwrap();
    }
    writer.finalize().unwrap();
}

//noinspection Duplicates
#[allow(dead_code)]
fn gen_note_stream_impl<T>(device: &cpal::Device, config: &cpal::StreamConfig,
                           oscillator_tables: OscillatorTables,  mut playback_note: PlaybackNote)
where
    T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>,
{
    let mut sample_count = 0;
    let mut sample_clock = -1.0 / SAMPLE_RATE;
    let duration_ms = playback_note.playback_duration_ms();
    let mut next_samples = move || {
        sample_clock = (sample_clock + 1.0) % SAMPLE_RATE;
        sample_count += 1;
        get_sample::get_note_sample(&mut playback_note, &oscillator_tables,
                                        sample_clock / SAMPLE_RATE,
                                        sample_count - 1)
    };

    let channels = config.channels as usize;
    let err_fn =
        |err| eprintln!("an error occurred on the output audio stream: {}", err);
    let stream = device.build_output_stream(
        config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            write_stream::<f32>(data, channels, &mut next_samples)
        },
        err_fn,
        None
    ).unwrap();
    stream.play().unwrap();

    std::thread::sleep(time::Duration::from_millis(duration_ms.ceil() as u64));
}

//noinspection Duplicates
#[allow(dead_code)]
fn gen_notes_stream_impl<T>(device: &cpal::Device, config: &cpal::StreamConfig,
                            oscillator_tables: OscillatorTables, mut playback_notes: Vec<PlaybackNote>,
                            note_duration_ms: u64)
{
    let mut sample_count = 0;
    let mut sample_clock = -1.0;
    let mut next_samples = move || {
        sample_clock = (sample_clock + 1.0) % SAMPLE_RATE;
        sample_count += 1;
        get_sample::get_notes_sample(&mut playback_notes, &oscillator_tables,
                                     sample_clock / SAMPLE_RATE,
                                     sample_count - 1)
    };

    let channels = config.channels as usize;
    let err_fn =
        |err| eprintln!("an error occurred on the output audio stream: {}", err);
    let stream = device.build_output_stream(
        config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            write_stream::<f32>(data, channels, &mut next_samples)
        },
        err_fn,
        None
    ).unwrap();
    stream.play().unwrap();
    
    std::thread::sleep(time::Duration::from_millis(note_duration_ms));
}

// Based on this https://github.com/RustAudio/cpal/issues/735  stereo output is interleaved samples
// in Left, right order.
// It's undocumented in cpal, and they ignored the request to document it
fn write_stream<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> (f32, f32))
where
    T: cpal::Sample + cpal::FromSample<f32>,
{
    for output_frame in output.chunks_mut(channels) {
        let (next_sample_l, next_sample_r) = next_sample();
        output_frame[0] = T::from_sample::<f32>(next_sample_l);
        output_frame[1] = T::from_sample::<f32>(next_sample_r);
    }
}