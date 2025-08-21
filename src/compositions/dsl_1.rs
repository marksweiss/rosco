use crate::dsl::parser::parse_dsl;
use crate::composition::comp_utils::play_track_grid;

#[allow(dead_code)]
pub(crate) fn play() {
    println!("playing dsl 1");

    let input = r#"
let env1 = a 0.3,0.4 d 0.5,0.6 s 0.6,0.5 r 1.0,0.0
let env2 = a 0.1,0.9 d 0.2,0.6 s 0.8,0.6 r 1.0,0.0
let delay1 = delay mix 0.5 decay 1.0 interval_ms 30.0 duration_ms 60.0 num_repeats 9 num_predelay_samples 30 num_concurrent_delays 2
let flanger1 = flanger window_size 20 mix 0.2
let samp1 = samp:/Users/markweiss/Downloads/punk_computer/003/piano_note_1_clipped.wav:0.003:{step}
let C5 = osc:sine,sine,sawtooth,sawtooth,sine,sine:5,C:0.3:{step}
let G5 = osc:sine,sine,sawtooth,sawtooth,sine,sine:5,G:0.3:{step} 

FixedTimeNoteSequence dur Quarter tempo 12 num_steps 16 panning -0.9
$env1
$delay1

apply step:(range 0,12,4) $samp1

FixedTimeNoteSequence dur Quarter tempo 12 num_steps 16 panning 0.9
$env1
$flanger1

apply step:(range 0,12,2) $C5

FixedTimeNoteSequence dur Quarter tempo 12 num_steps 16 panning 0.9
$env1
$flanger1
$delay1

apply step:(range 1,13,3) $G5
"#;

    play_track_grid(parse_dsl(input).unwrap());
}