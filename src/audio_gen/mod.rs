#[allow(clippy::module_inception)]
mod audio_gen;
pub mod get_sample;
pub mod oscillator;

pub(crate) use audio_gen::{gen_notes_stream, read_audio_file};
pub use oscillator::Waveform;
