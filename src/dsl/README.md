# Overview of Script Structure and Processing

When the script is parsed, the parser first creates a @track.rs `Vec<Track>`.

The parser then processes macro substitution declarations at the top of the script, before the first `Outer Block`. These declarations use the `let` keyword to bind expressions to identifiers for later reuse. Macro names can then be referenced throughout the script using the `$` prefix syntax (e.g., `$env1`).

It then reads each `Outer Block`. For each one, the parser creates a new `FixedTimeNoteSequence` and a new `TrackEffects`. The envelope and effects declared in the script are converted to their corresponding structs, `Envelope`, `Flanger`, `Delay` and `LFO`. These are passed to the builder call to create the `TrackEffects`. If a panning value is specified in the sequence definition, the `TrackEffects` panning is set to that value and the number of channels is set to 2 for stereo output. Then a Track is built, setting its sequence to the new `FixedTimeNoteSequence` and its track_effects to the new `TrackEffects`.

After this the parser processes each line defining a new note declaration, constructing a `PlaybackNote` of either type `osc` for a `Note` based on its waveforms, or of type `samp` for `SampledNote`. Each note is added to the current sequence.

After the last outer block, the parser constructs a `TrackGrid`, setting its tracks to the `Vec<Track>` and returns it.

## Track-Level Panning

Track-level panning can be specified as an optional parameter after the `num_steps` value in the `FixedTimeNoteSequence` declaration using the `panning` keyword followed by a float value. The panning value should be a float between -1.0 and 1.0, where:
- `-1.0` = full left panning
- `0.0` = center (no panning)
- `1.0` = full right panning

When panning is specified, the track automatically switches to stereo output (2 channels). If no panning is specified, the track uses mono output (1 channel) with a default panning of 0.0.

Examples:
- `FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16 panning -0.5` (panned left)
- `FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16 panning 0.3` (panned slightly right)
- `FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16` (center, mono)

# DSL Syntax Specification

- Expressions are ALL_CAPS
- Terminals are a sequence upper- and/or lower-case characters and possibly other ASCII characters
- `+` means "one or more"
- `*` means "zero or more"
- `{1}` means "exactly one"
- `|` indicates alternation
- `.` represents any chracter

---

```
COMMENT -> #.*

DELAY -> delay mix f32 decay f32 interval_ms f32 duration_ms f32 num_repeats usize num_predelay_samples usize num_concurrent_delays uszie 
FLANGER -> flanger window_size usize mix f32
LFO -> lfo freq f32 amp f32 waveforms WAVEFORMS
EFFECT_DEF -> DELAY | FLANGER | LFO

WESTERN_PITCH -> C | CSharp | C#| DFlat | Db | D | DSharp | D#| EFlat | Eb| E | F | FSharp | F#| GFlat | Gb | G | GSharp | G# | AFlat | Ab | A | ASharp | A#| BFlat | Bb | B
OCTAVE -> 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8
NOTE_FREQ -> f32 | OCTAVE,WESTERN_PITCH
WAVEFORM -> sine | sin | square | sqr | triangle | tri | sawtooth | saw | guassiannoise | noise
WAVEFORMS -> WAVEFORM, | WAVEFORM
VOLUME -> f32
FILE_PATH -> .+
STEP_INDEX -> usize
OSC_NOTE -> osc:WAVEFORMS:NOTE_FREQ:VOLUME:STEP_INDEX
SAMP_NOTE -> samp:FILE_PATH:VOLUME:STEP_INDEX
NOTE_DECLARATION -> OSC_NOTE | SAMP_NOTE

DURATION_TYPE -> Whole | Half | Quarter | Eighth | Sixteenth | ThirtySecond | SixtyFourth | 1 | 1/2 | 1/4 | 1/8 | 1/16 | 1/32 | 1/64
TEMPO -> u8
NUM_STEPS -> usize
PANNING_VALUE -> f32
PANNING -> panning PANNING_VALUE
SEQUENCE_DEF -> FixedTimeNoteSequence dur DURATION_TYPE tempo TEMPO num_steps NUM_STEPS [PANNING]

ENVELOPE_PAIR -> f32,f32
ENVELOPE_DEF -> a ENVELOPE_PAIR d ENVELOPE_PAIR s ENVELOPE_PAIR r ENVELOPE_PAIR

IDENTIFIER -> `[a-zA-Z][a-zA-Z0-9\-_]*`
MACRO_REFERENCE -> $IDENTIFIER
EXPR -> ENVELOPE_DEF | EFFECT_DEF | SEQUENCE_DEF | NOTE_DECLARATION | MACRO_REFERENCE
ASSIGNMENT -> let IDENTIFIER = EXPR

OUTER_BLOCK -> SEQUENCE_DEF{1} ENVELOPE_DEF* EFFECT_DEF* NOTE_DECLARATION*

SCRIPT -> ASSIGNMENT* OUTER_BLOCK+
```
