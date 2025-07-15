extern crate derive_builder;

mod audio_gen;
mod common;
mod effect;
mod envelope;
mod filter;
mod midi;
mod note;
mod sequence;
mod track;
mod composition;
mod meter;
mod dsl;
mod compositions;

use crate::compositions::dsl_1;
// use crate::compositions::computer_punk_001;
// use crate::compositions::computer_punk_003;

fn main() {
    dsl_1::play();
    // computer_punk_001::play();
    // computer_punk_003::play();
}
