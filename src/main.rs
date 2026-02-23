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

use crate::compositions::dsl_1;
// use crate::compositions::computer_punk_001;
// use crate::compositions::computer_punk_003;

fn main() {
    dsl_1::play();
    // computer_punk_001::play();
    // computer_punk_003::play();
}
