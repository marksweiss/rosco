extern crate derive_builder;

pub mod audio_gen;
pub mod common;
pub mod effect;
pub mod envelope;
pub mod filter;
pub mod midi;
pub mod note;
pub mod sequence;
pub mod track;
pub mod composition;
pub mod meter;
pub mod dsl;
pub mod compositions;
pub mod tui;

#[cfg(test)]
pub mod test_frequency_fix;