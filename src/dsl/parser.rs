use std::str::FromStr;
use std::collections::HashMap;
use regex;

use crate::audio_gen::oscillator::Waveform;
use crate::effect::delay::DelayBuilder;
use crate::effect::flanger::{FlangerBuilder};
use crate::effect::lfo::{LFOBuilder};
use crate::effect::tremolo::TremoloBuilder;
use crate::effect::vibrato::VibratoBuilder;
use crate::effect::chorus::ChorusBuilder;
use crate::effect::equalizer::EqualizerBuilder;
use crate::envelope::envelope::{EnvelopeBuilder, EnvelopeCurve};
use crate::envelope::envelope_pair::EnvelopePair;
use crate::filter::low_pass_filter::{LowPassFilterBuilder};
use crate::meter::durations::{DurationType};
use crate::note::note::{NoteBuilder};
use crate::note::playback_note::{NoteSource, PlaybackNote, PlaybackNoteBuilder};
use crate::note::sampled_note::{SampledNoteBuilder};
use crate::note::scales::WesternPitch;
use crate::sequence::fixed_time_note_sequence::{FixedTimeNoteSequence, FixedTimeNoteSequenceBuilder};
use crate::sequence::note_sequence_trait::AppendNote;
use crate::track::track::{Track, TrackBuilder};
use crate::track::track_effects::{TrackEffects, TrackEffectsBuilder};
use crate::track::track_grid::{TrackGrid, TrackGridBuilder};

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum WaveformType {
    Sine,
    Sin,
    Square,
    Sqr,
    Triangle,
    Tri,
    Sawtooth,
    Saw,
    GaussianNoise,
    Noise,
}

impl FromStr for WaveformType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "sine" | "sin" => Ok(WaveformType::Sine),
            "square" | "sqr" => Ok(WaveformType::Square),
            "triangle" | "tri" => Ok(WaveformType::Triangle),
            "sawtooth" | "saw" => Ok(WaveformType::Sawtooth),
            "gaussiannoise" | "noise" => Ok(WaveformType::GaussianNoise),
            _ => Err(format!("Unknown waveform: {}", s)),
        }
    }
}

impl WaveformType {
    fn to_waveform(&self) -> Waveform {
        match self {
            WaveformType::Sine | WaveformType::Sin => Waveform::Sine,
            WaveformType::Square | WaveformType::Sqr => Waveform::Square,
            WaveformType::Triangle | WaveformType::Tri => Waveform::Triangle,
            WaveformType::Sawtooth | WaveformType::Saw => Waveform::Saw,
            WaveformType::GaussianNoise | WaveformType::Noise => Waveform::GaussianNoise,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum WesternPitchType {
    C,
    CSharp,
    DFlat,
    D,
    DSharp,
    EFlat,
    E,
    F,
    FSharp,
    GFlat,
    G,
    GSharp,
    AFlat,
    A,
    ASharp,
    BFlat,
    B,
}

impl FromStr for WesternPitchType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "C" => Ok(WesternPitchType::C),
            "CSharp" | "C#" => Ok(WesternPitchType::CSharp),
            "DFlat" | "Db" => Ok(WesternPitchType::DFlat),
            "D" => Ok(WesternPitchType::D),
            "DSharp" | "D#" => Ok(WesternPitchType::DSharp),
            "EFlat" | "Eb" => Ok(WesternPitchType::EFlat),
            "E" => Ok(WesternPitchType::E),
            "F" => Ok(WesternPitchType::F),
            "FSharp" | "F#" => Ok(WesternPitchType::FSharp),
            "GFlat" | "Gb" => Ok(WesternPitchType::GFlat),
            "G" => Ok(WesternPitchType::G),
            "GSharp" | "G#" => Ok(WesternPitchType::GSharp),
            "AFlat" | "Ab" => Ok(WesternPitchType::AFlat),
            "A" => Ok(WesternPitchType::A),
            "ASharp" | "A#" => Ok(WesternPitchType::ASharp),
            "BFlat" | "Bb" => Ok(WesternPitchType::BFlat),
            "B" => Ok(WesternPitchType::B),
            _ => Err(format!("Unknown western pitch: {}", s)),
        }
    }
}

impl WesternPitchType {
    fn to_western_pitch(&self) -> WesternPitch {
        match self {
            WesternPitchType::C => WesternPitch::C,
            WesternPitchType::CSharp => WesternPitch::CSharp,
            WesternPitchType::DFlat => WesternPitch::DFlat,
            WesternPitchType::D => WesternPitch::D,
            WesternPitchType::DSharp => WesternPitch::DSharp,
            WesternPitchType::EFlat => WesternPitch::EFlat,
            WesternPitchType::E => WesternPitch::E,
            WesternPitchType::F => WesternPitch::F,
            WesternPitchType::FSharp => WesternPitch::FSharp,
            WesternPitchType::GFlat => WesternPitch::GFlat,
            WesternPitchType::G => WesternPitch::G,
            WesternPitchType::GSharp => WesternPitch::GSharp,
            WesternPitchType::AFlat => WesternPitch::AFlat,
            WesternPitchType::A => WesternPitch::A,
            WesternPitchType::ASharp => WesternPitch::ASharp,
            WesternPitchType::BFlat => WesternPitch::BFlat,
            WesternPitchType::B => WesternPitch::B,
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct DelayDef {
    pub(crate) mix: f32,
    pub(crate) decay: f32,
    pub(crate) interval_ms: f32,
    pub(crate) duration_ms: f32,
    pub(crate) num_repeats: usize,
    pub(crate) num_predelay_samples: usize,
    pub(crate) num_concurrent_delays: usize,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct FlangerDef {
    pub(crate) delay_ms: f32,
    pub(crate) mix: f32,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct LFODef {
    pub(crate) freq: f32,
    pub(crate) amp: f32,
    pub(crate) waveforms: Vec<WaveformType>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct FilterDef {
    pub(crate) cutoff_frequency: f32,
    pub(crate) resonance: f32,
    pub(crate) mix: f32,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct TremoloDef {
    pub(crate) mod_freq: f32,
    pub(crate) mod_depth: f32,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct VibratoDef {
    pub(crate) avg_delay: f32,
    pub(crate) mod_width: f32,
    pub(crate) mod_freq: f32,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct ChorusDef {
    pub(crate) chorus_count: usize,
    pub(crate) chorus_gains: Vec<f32>,
    pub(crate) dry_gain: f32,
    pub(crate) chorus_delays: Vec<f32>,
    pub(crate) mod_freqs: Vec<f32>,
    pub(crate) mod_widths: Vec<f32>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct EqualizerDef {
    pub(crate) gains: Vec<f32>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) enum EffectDef {
    Delay(DelayDef),
    Flanger(FlangerDef),
    LFO(LFODef),
    Filter(FilterDef),
    Tremolo(TremoloDef),
    Vibrato(VibratoDef),
    Chorus(ChorusDef),
    Equalizer(EqualizerDef),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct EnvelopeDef {
    pub(crate) attack: (f32, f32),
    pub(crate) attack_curve: EnvelopeCurve,
    pub(crate) decay: (f32, f32),
    pub(crate) decay_curve: EnvelopeCurve,
    pub(crate) sustain: (f32, f32),
    pub(crate) sustain_curve: EnvelopeCurve,
    pub(crate) release: (f32, f32),
    pub(crate) release_curve: EnvelopeCurve,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct SequenceDef {
    pub(crate) dur: DurationType,
    pub(crate) tempo: u8,
    pub(crate) num_steps: usize,
    pub(crate) panning: Option<f32>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) enum NoteDeclaration {
    Oscillator {
        waveforms: Vec<WaveformType>,
        note_freq: f32,
        volume: f32,
        step_index: usize,
    },
    Sample {
        file_path: String,
        volume: f32,
        step_index: usize,
    },
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct OuterBlock {
    pub(crate) sequence_def: SequenceDef,
    pub(crate) envelope_defs: Vec<EnvelopeDef>,
    pub(crate) effect_defs: Vec<EffectDef>,
    pub(crate) note_declarations: Vec<NoteDeclaration>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct MacroDef {
    pub(crate) name: String,
    pub(crate) expression: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct Script {
    pub(crate) macro_defs: HashMap<String, String>,
    pub(crate) outer_blocks: Vec<OuterBlock>,
}

#[allow(dead_code)]
pub(crate) struct Parser {
    tokens: Vec<String>,
    current: usize,
}

impl Parser {
    pub(crate) fn new(input: &str) -> Result<Self, String> {
        let input_tokens: Vec<String> = input.lines().map(|s| s.to_string()).collect();

        let input_after_macro = Self::expand_macros(input_tokens.join("\n").as_str())?;

        let input_after_generators = Self::expand_generators(input_after_macro.as_str())
            .unwrap_or_else(|_| input_after_macro.to_string());

        let input_after_apply = Self::expand_apply_defs(input_after_generators.as_str())
            .unwrap_or_else(|_| Vec::new());

        let tokens = Self::tokenize(&input_after_apply.join("\n"));

        Ok(Self {
            tokens,
            current: 0,
        })
    }

    fn expand_macros(input: &str) -> Result<String, String> {
        let mut expanded = input.to_string();
        let mut macro_defs = HashMap::new();

        let lines: Vec<String> = input.lines().map(|s| s.to_string().trim().to_string()).collect();

        // First pass: collect all macro definitions
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim();
            if line.starts_with("let ") {
                // Parse macro definition
                if let Some((name, value)) = Self::parse_macro_definition_line(line)? {
                    macro_defs.insert(name, value);
                }
            } else if line.starts_with("FixedTimeNoteSequence") {
                break;
            }
            i += 1;
        }

        // Multi-pass expansion: repeat until no changes
        let mut changed = true;
        while changed {
            changed = false;
            let mut new_expanded = expanded.clone();
            for (name, value) in &macro_defs {
                let pattern = format!("${}", name);
                if new_expanded.contains(&pattern) {
                    new_expanded = new_expanded.replace(&pattern, value);
                    changed = true;
                }
            }
            expanded = new_expanded;
        }
        // Check for any remaining $name that is not in macro_defs and return error
        let re = regex::Regex::new(r"\$([a-zA-Z][a-zA-Z0-9\-_]*)").unwrap();
        for (line_idx, line) in expanded.lines().enumerate() {
            for cap in re.captures_iter(line) {
                let macro_name = &cap[1];
                if !macro_defs.contains_key(macro_name) {
                    return Err(format!(
                        "Undefined macro '${}' encountered on line {}: \n  {}",
                        macro_name,
                        line_idx + 1,
                        line.trim()
                    ));
                }
            }
        }
        Ok(expanded)
    }

    fn parse_macro_definition_line(line: &str) -> Result<Option<(String, String)>, String> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 || parts[0] != "let" || parts[2] != "=" {
            return Ok(None);
        }

        let name = parts[1].to_string();
        let value = parts[3..].join(" ");

        Ok(Some((name, value)))
    }

    // TODO FIX INNER LOOP BORROW ISSUE SO THAT WE CAN HAVE MORE THAN ONE SUBST PER LINE
    #[allow(unused_assignments)]
    fn expand_generators(input: &str) -> Result<String, String> {

        let mut lines: Vec<String> = input.lines().map(|s| s.to_string()).collect();

        let mut i = 0;
        let lines_len = lines.len();
        while i < lines_len {
            let line_content = lines[i].trim();
            let mut chars = line_content.chars().peekable();
            let mut in_generator = false;
            let mut j: usize= 0;
            let mut lbound: usize= 0;
            let mut rbound: usize = 0;
            while let Some(ch) = chars.next() {
                if ch == '\n' {
                    break;
                }
                if in_generator && ch != ')' {
                    j += 1;
                    continue;
                }
                if ch == '(' {
                    in_generator = true;
                    lbound = j;
                    j += 1;
                    continue;

                } else if ch == ')' {
                    rbound = j;
                    let generated =
                        Self::call_generator_with_args(&line_content[lbound..rbound + 1])
                            .unwrap_or("parse of generator failed".to_string());
                    lines[i] = line_content.replace(&line_content[lbound..rbound + 1], &generated);
                    in_generator = false;
                    break;
                }
                j += 1;
            }
            i += 1;
        }
        return Ok(lines.join("\n"));
    }

    fn call_generator_with_args(generator_substring: &str) -> Result<String, String> {
        let generator_and_args = generator_substring[1..generator_substring.len() - 1]
            .split(" ").collect::<Vec<&str>>();
        let generator_name = generator_and_args[0];
        let args = generator_and_args[1].split(",").collect::<Vec<&str>>();
        match generator_name {
            "range" => Self::expand_range_generator(args),
            _ => Err(format!("Unknown generator: {}", generator_name)),
        }
    }

    fn expand_range_generator(args: Vec<&str>) -> Result<String, String> {
        if args.len() != 3 {
            return Err("range generator requires 3 arguments".to_string());
        }
        let start = args[0].parse::<i32>().map_err(|_| "range generator start must be an integer".to_string())?;
        let end = args[1].parse::<i32>().map_err(|_| "range generator end must be an integer".to_string())?;
        let step = args[2].parse::<i32>().map_err(|_| "range generator step must be an integer".to_string())?;
        let mut result = String::new();
        for i in (start..=end).step_by(step as usize) {
            result.push_str(&i.to_string());
            result.push(',');
        }
        result.pop();
        Ok(result)
    }

    fn expand_apply_defs(input: &str) -> Result<Vec<String>, String> {
    
        let mut lines: Vec<String> = input.lines().map(|s| s.to_string()).collect();
        let mut i = 0;
        
        while i < lines.len() {
            let line_content = lines[i].trim();
            if line_content.starts_with("apply") {
                if let Some((apply_defs, _identifier)) =
                        Self::parse_apply_def(line_content)? {
                    // Build lines from the apply list of values and template
                    let mut new_lines = Vec::new();
                    for (key, values) in apply_defs {
                        for value in values {
                            let line_content_tokens = line_content.split(" ").collect::<Vec<&str>>();
                            let apply_expression = line_content_tokens[2..].join(" ");
                            let new_line = apply_expression.replace(&format!("{{{}}}", key), &value);
                            new_lines.push(new_line);
                        }
                    }

                    // Insert expanded lines in place at the point of the apply line after
                    // commenting out the apply line to not process in later passes
                    // Comment out the original apply line
                    lines[i] = format!("# {}", lines[i]);
                    let num_new_lines = new_lines.len();
                    // Insert new lines
                    for (j, new_line) in new_lines.into_iter().enumerate() {
                        lines.insert(i + j + 1, new_line);
                    }
                    // Skip index past inserted lines
                    i += num_new_lines;
                }
            }
            i += 1;
        }

        Ok(lines)
        
    }

    fn parse_apply_def(line: &str) -> Result<Option<(HashMap<String, Vec<String>>, String)>, String> {
        let mut parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 || parts[0] != "apply" {
            return Ok(None);
        }

        let mut apply_defs = HashMap::new();
        for token in parts.iter_mut() {
            if token.contains(":") {
                let key = token.split(":").next().unwrap().to_string();
                if key == "osc" || key == "samp" {
                    continue;
                }  
                let value =
                    token.split(":").nth(1).unwrap().split(",").map(|s| s.to_string()).collect();
                apply_defs.insert(key, value);
            }
        }
        
        // NOTE: identifiers can't have ':' in their name or this code breaks
        let identifier = parts[parts.len() - 1].to_string();

        Ok(Some((apply_defs, identifier)))
    }

    fn tokenize(input: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current_token = String::new();
        let mut in_comment = false;
        let mut in_file_path = false;
        let mut chars = input.chars().peekable();
        let mut at_line_start = true;
        let mut line_buffer = String::new();

        while let Some(ch) = chars.next() {
            if at_line_start && ch == '#' {
                in_comment = true;
                continue;
            }

            // Buffer the line for blank line detection
            if ch == '\n' {
                if !in_comment {
                    // If the line is blank (only whitespace), skip it
                    if line_buffer.trim().is_empty() {
                        at_line_start = true;
                        line_buffer.clear();
                        continue;
                    }
                }
                at_line_start = true;
                line_buffer.clear();
            } else {
                line_buffer.push(ch);
                if !ch.is_whitespace() && ch != '#' {
                    at_line_start = false;
                }
            }

            if in_comment {
                if ch == '\n' {
                    in_comment = false;
                }
                continue;
            }

            if in_file_path {
                if ch == ':' {
                    in_file_path = false;
                    if !current_token.is_empty() {
                        tokens.push(current_token.clone());
                        current_token.clear();
                    }
                    tokens.push(":".to_string());
                } else {
                    current_token.push(ch);
                }
                continue;
            }

            // Detect start of file path after 'samp' and ':'
            if current_token == "samp" && chars.peek() == Some(&':') {
                tokens.push(current_token.clone());
                current_token.clear();
                chars.next(); // consume the ':'
                tokens.push(":".to_string());
                in_file_path = true;
                continue;
            }

            match ch {
                ':' | ',' | ' ' | '\n' | '\r' | '\t' => {
                    if !current_token.is_empty() {
                        tokens.push(current_token.clone());
                        current_token.clear();
                    }
                    if ch != ' ' && ch != '\n' && ch != '\r' && ch != '\t' {
                        tokens.push(ch.to_string());
                    }
                }
                _ => {
                    current_token.push(ch);
                }
            }
        }

        if !current_token.is_empty() {
            tokens.push(current_token);
        }

        tokens.retain(|token| !token.is_empty());
        tokens
    }

    pub(crate) fn parse(&mut self) -> Result<TrackGrid<FixedTimeNoteSequence>, String> {
        let script = self.parse_script()?;
        self.build_track_grid(script)
    }

    fn parse_script(&mut self) -> Result<Script, String> {
        let mut macro_defs = HashMap::new();
        let mut outer_blocks = Vec::new();
        
        // Parse macro definitions first
        while self.current < self.tokens.len() && self.peek() == "let" && !self.is_comment_start() {
            let (name, expression) = self.parse_assignment()?;
            macro_defs.insert(name, expression);
        }
        
        // Parse outer blocks
        while self.current < self.tokens.len() && !self.is_comment_start() {
            let block = self.parse_outer_block()?;
            outer_blocks.push(block);
        }

        Ok(Script { 
            macro_defs,
            outer_blocks 
        })
    }

    fn parse_outer_block(&mut self) -> Result<OuterBlock, String> {
        let sequence_def = self.parse_sequence_def()?;
        let mut envelope_defs = Vec::new();
        let mut effect_defs = Vec::new();
        let mut note_declarations = Vec::new();

        // Parse optional envelope definitions
        while self.current < self.tokens.len() && self.peek() == "a" {
            let envelope_def = self.parse_envelope_def()?;
            envelope_defs.push(envelope_def);
        }

        // Parse optional effect definitions
        while self.current < self.tokens.len() && self.is_effect_start() {
            let effect_def = self.parse_effect_def()?;
            effect_defs.push(effect_def);
        }

        // Parse note declarations
        while self.current < self.tokens.len() && self.is_note_declaration_start() {
            let note_declaration = self.parse_note_declaration()?;
            note_declarations.push(note_declaration);
        }

        Ok(OuterBlock {
            sequence_def,
            envelope_defs,
            effect_defs,
            note_declarations,
        })
    }

    fn parse_sequence_def(&mut self) -> Result<SequenceDef, String> {
        self.skip_comment_lines();

        self.expect("FixedTimeNoteSequence")?;
        self.expect("dur")?;
        let dur = self.parse_duration_type()?;
        self.expect("tempo")?;
        let tempo = self.parse_u8()?;
        self.expect("num_steps")?;
        let num_steps = self.parse_usize()?;

        // Parse optional panning parameter
        let panning = if self.current < self.tokens.len() &&
                         !self.is_comment_start() &&
                         self.peek() == "panning" {
            self.expect("panning")?;
            Some(self.parse_f32()?)
        } else {
            None
        };

        Ok(SequenceDef {
            dur,
            tempo,
            num_steps,
            panning,
        })
    }

    fn parse_duration_type(&mut self) -> Result<DurationType, String> {
        let token = self.advance();
        DurationType::from_str(&token)
    }

    fn parse_envelope_def(&mut self) -> Result<EnvelopeDef, String> {
        self.skip_comment_lines();

        self.expect("a")?;
        let (attack, attack_curve) = self.parse_envelope_pair()?;
        self.expect("d")?;
        let (decay, decay_curve) = self.parse_envelope_pair()?;
        self.expect("s")?;
        let (sustain, sustain_curve) = self.parse_envelope_pair()?;
        self.expect("r")?;
        let (release, release_curve) = self.parse_envelope_pair()?;

        Ok(EnvelopeDef {
            attack,
            attack_curve,
            decay,
            decay_curve,
            sustain,
            sustain_curve,
            release,
            release_curve,
        })
    }

    fn parse_envelope_pair(&mut self) -> Result<((f32, f32), EnvelopeCurve), String> {
        self.skip_comment_lines();

        let first = self.parse_f32()?;
        self.expect(",")?;
        let second = self.parse_f32()?;
        let curve = if self.current < self.tokens.len() && self.peek() == "," {
            self.advance(); // consume comma
            let curve_token = self.advance();
            match curve_token.as_str() {
                "exp" => EnvelopeCurve::Exponential,
                "lin" => EnvelopeCurve::Linear,
                _ => return Err(format!("Unknown curve type: {}", curve_token)),
            }
        } else {
            EnvelopeCurve::Linear
        };
        Ok(((first, second), curve))
    }

    fn parse_effect_def(&mut self) -> Result<EffectDef, String> {
        if self.peek() == "delay" {
            self.parse_delay_def()
        } else if self.peek() == "flanger" {
            self.parse_flanger_def()
        } else if self.peek() == "lfo" {
            self.parse_lfo_def()
        } else if self.peek() == "filter" {
            self.parse_filter_def()
        } else if self.peek() == "tremolo" {
            self.parse_tremolo_def()
        } else if self.peek() == "vibrato" {
            self.parse_vibrato_def()
        } else if self.peek() == "chorus" {
            self.parse_chorus_def()
        } else if self.peek() == "equalizer" {
            self.parse_equalizer_def()
        } else {
            Err(format!("Unknown effect type: {}", self.peek()))
        }
    }

    fn parse_delay_def(&mut self) -> Result<EffectDef, String> {
        self.skip_comment_lines();

        self.expect("delay")?;
        self.expect("mix")?;
        let mix = self.parse_f32()?;
        self.expect("decay")?;
        let decay = self.parse_f32()?;
        self.expect("interval_ms")?;
        let interval_ms = self.parse_f32()?;
        self.expect("duration_ms")?;
        let duration_ms = self.parse_f32()?;
        self.expect("num_repeats")?;
        let num_repeats = self.parse_usize()?;
        self.expect("num_predelay_samples")?;
        let num_predelay_samples = self.parse_usize()?;
        self.expect("num_concurrent_delays")?;
        let num_concurrent_delays = self.parse_usize()?;

        Ok(EffectDef::Delay(DelayDef {
            mix,
            decay,
            interval_ms,
            duration_ms,
            num_repeats,
            num_predelay_samples,
            num_concurrent_delays,
        }))
    }

    fn parse_flanger_def(&mut self) -> Result<EffectDef, String> {
        self.skip_comment_lines();

        self.expect("flanger")?;
        self.expect("window_size")?;
        // window_size (in samples) is converted to delay_ms for the new flanger implementation
        let window_size = self.parse_usize()?;
        let delay_ms = window_size as f32 / (crate::common::constants::SAMPLE_RATE / 1000.0);
        self.expect("mix")?;
        let mix = self.parse_f32()?;

        Ok(EffectDef::Flanger(FlangerDef {
            delay_ms,
            mix,
        }))
    }

    fn parse_lfo_def(&mut self) -> Result<EffectDef, String> {
        self.skip_comment_lines();

        self.expect("lfo")?;
        self.expect("freq")?;
        let freq = self.parse_f32()?;
        self.expect("amp")?;
        let amp = self.parse_f32()?;
        self.expect("waveforms")?;
        let waveforms = self.parse_waveforms()?;

        Ok(EffectDef::LFO(LFODef {
            freq,
            amp,
            waveforms,
        }))
    }

    fn parse_filter_def(&mut self) -> Result<EffectDef, String> {
        self.skip_comment_lines();

        self.expect("filter")?;
        self.expect("cutoff_frequency")?;
        let cutoff_frequency = self.parse_f32()?;
        self.expect("resonance")?;
        let resonance = self.parse_f32()?;
        self.expect("mix")?;
        let mix = self.parse_f32()?;

        Ok(EffectDef::Filter(FilterDef {
            cutoff_frequency,
            resonance,
            mix,
        }))
    }

    fn parse_tremolo_def(&mut self) -> Result<EffectDef, String> {
        self.skip_comment_lines();

        self.expect("tremolo")?;
        self.expect("mod_freq")?;
        let mod_freq = self.parse_f32()?;
        self.expect("mod_depth")?;
        let mod_depth = self.parse_f32()?;

        Ok(EffectDef::Tremolo(TremoloDef {
            mod_freq,
            mod_depth,
        }))
    }

    fn parse_vibrato_def(&mut self) -> Result<EffectDef, String> {
        self.skip_comment_lines();

        self.expect("vibrato")?;
        self.expect("avg_delay")?;
        let avg_delay = self.parse_f32()?;
        self.expect("mod_width")?;
        let mod_width = self.parse_f32()?;
        self.expect("mod_freq")?;
        let mod_freq = self.parse_f32()?;

        Ok(EffectDef::Vibrato(VibratoDef {
            avg_delay,
            mod_width,
            mod_freq,
        }))
    }

    fn parse_chorus_def(&mut self) -> Result<EffectDef, String> {
        self.skip_comment_lines();

        self.expect("chorus")?;
        self.expect("chorus_count")?;
        let chorus_count = self.parse_usize()?;
        self.expect("chorus_gains")?;
        let chorus_gains = self.parse_f32_list()?;
        self.expect("dry_gain")?;
        let dry_gain = self.parse_f32()?;
        self.expect("chorus_delays")?;
        let chorus_delays = self.parse_f32_list()?;
        self.expect("mod_freqs")?;
        let mod_freqs = self.parse_f32_list()?;
        self.expect("mod_widths")?;
        let mod_widths = self.parse_f32_list()?;

        Ok(EffectDef::Chorus(ChorusDef {
            chorus_count,
            chorus_gains,
            dry_gain,
            chorus_delays,
            mod_freqs,
            mod_widths,
        }))
    }

    fn parse_equalizer_def(&mut self) -> Result<EffectDef, String> {
        self.skip_comment_lines();

        self.expect("equalizer")?;
        self.expect("gains")?;
        let gains = self.parse_f32_list()?;

        Ok(EffectDef::Equalizer(EqualizerDef {
            gains,
        }))
    }

    fn parse_f32_list(&mut self) -> Result<Vec<f32>, String> {
        let mut values = Vec::new();

        loop {
            let value = self.parse_f32()?;
            values.push(value);

            if self.peek() == "," {
                self.advance(); // consume comma
            } else {
                break;
            }
        }

        Ok(values)
    }

    fn parse_waveforms(&mut self) -> Result<Vec<WaveformType>, String> {
        let mut waveforms = Vec::new();
        
        loop {
            let waveform = self.parse_waveform()?;
            waveforms.push(waveform);
            
            if self.peek() == "," {
                self.advance(); // consume comma
            } else {
                break;
            }
        }

        Ok(waveforms)
    }

    fn parse_waveform(&mut self) -> Result<WaveformType, String> {
        let token = self.advance();
        WaveformType::from_str(&token)
    }

    fn parse_note_declaration(&mut self) -> Result<NoteDeclaration, String> {
        if self.peek() == "osc" {
            self.parse_osc_note()
        } else if self.peek() == "samp" {
            self.parse_samp_note()
        } else {
            Err(format!("Unknown note type: {}", self.peek()))
        }
    }

    fn parse_osc_note(&mut self) -> Result<NoteDeclaration, String> {
        self.skip_comment_lines();

        self.expect("osc")?;
        self.expect(":")?;
        let waveforms = self.parse_waveforms()?;
        self.expect(":")?;
        let note_freq = self.parse_note_freq()?;
        self.expect(":")?;
        let volume = self.parse_f32()?;
        self.expect(":")?;
        let step_index = self.parse_usize()?;

        Ok(NoteDeclaration::Oscillator {
            waveforms,
            note_freq,
            volume,
            step_index,
        })
    }

    fn parse_samp_note(&mut self) -> Result<NoteDeclaration, String> {
        self.skip_comment_lines();
        
        self.expect("samp")?;
        self.expect(":")?;
        let file_path = self.parse_file_path()?;
        self.expect(":")?;
        let volume = self.parse_f32()?;
        self.expect(":")?;
        let step_index = self.parse_usize()?;

        Ok(NoteDeclaration::Sample {
            file_path,
            volume,
            step_index,
        })
    }

    fn parse_note_freq(&mut self) -> Result<f32, String> {
        let token = self.advance();
        
        // Try to parse as octave,western_pitch format first
        if let Ok(octave) = token.parse::<u8>() {
            if self.peek() == "," {
                self.advance(); // consume comma
                let pitch_token = self.advance();
                if let Ok(pitch) = WesternPitchType::from_str(&pitch_token) {
                    let western_pitch = pitch.to_western_pitch();
                    return Ok(western_pitch.get_frequency(octave));
                } else {
                    return Err(format!("Invalid western pitch: {}", pitch_token));
                }
            }
        }
        
        // Try to parse as western pitch (default octave 4)
        if let Ok(pitch) = WesternPitchType::from_str(&token) {
            let western_pitch = pitch.to_western_pitch();
            // Default to octave 4 (middle C)
            return Ok(western_pitch.get_frequency(4));
        }
        
        // Try to parse as float
        token.parse::<f32>().map_err(|_| format!("Invalid note frequency: {}", token))
    }

    fn parse_file_path(&mut self) -> Result<String, String> {
        let mut file_path = String::new();
        
        while self.current < self.tokens.len() && self.peek() != ":" {
            if !file_path.is_empty() {
                file_path.push(':');
            }
            file_path.push_str(&self.advance());
        }
        
        if file_path.is_empty() {
            Err("Empty file path".to_string())
        } else {
            Ok(file_path)
        }
    }
    
    fn skip_comment_lines(&mut self) {
        while self.current < self.tokens.len() && self.peek() == "#" {
            while self.current < self.tokens.len() && self.peek() != "\n" {
                self.advance();
            }
            self.advance(); // consume newline
        }
    }

    fn is_effect_start(&self) -> bool {
        self.peek() == "delay" || self.peek() == "flanger" || self.peek() == "lfo" || self.peek() == "filter"
            || self.peek() == "tremolo" || self.peek() == "vibrato" || self.peek() == "chorus" || self.peek() == "equalizer"
    }

    fn is_note_declaration_start(&self) -> bool {
        self.peek() == "osc" || self.peek() == "samp"
    }

    fn is_comment_start(&self) -> bool {
        self.peek() == "#"
    }

    fn expect(&mut self, expected: &str) -> Result<(), String> {
        let token = self.advance();
        if token == expected {
            Ok(())
        } else {
            Err(format!("Expected '{}', got '{}'", expected, token))
        }
    }

    fn advance(&mut self) -> String {
        if self.current < self.tokens.len() {
            let token = self.tokens[self.current].clone();
            self.current += 1;
            token
        } else {
            String::new()
        }
    }

    fn peek(&self) -> &str {
        if self.current < self.tokens.len() {
            &self.tokens[self.current]
        } else {
            ""
        }
    }

    fn parse_f32(&mut self) -> Result<f32, String> {
        let token = self.advance();
        token.parse::<f32>().map_err(|_| format!("Invalid float: {}", token))
    }

    fn parse_u8(&mut self) -> Result<u8, String> {
        let token = self.advance();
        token.parse::<u8>().map_err(|_| format!("Invalid u8: {}", token))
    }

    fn parse_usize(&mut self) -> Result<usize, String> {
        let token = self.advance();
        token.parse::<usize>().map_err(|_| format!("Invalid usize: {}", token))
    }

    fn build_track_grid(&self, script: Script) -> Result<TrackGrid<FixedTimeNoteSequence>, String> {
        let mut tracks = Vec::new();

        for block in script.outer_blocks {
            let track = self.build_track_from_block(block)?;
            tracks.push(track);
        }

        TrackGridBuilder::default()
            .tracks(tracks)
            .build()
            .map_err(|e| format!("Failed to build TrackGrid: {:?}", e))
    }

    fn build_track_from_block(&self, block: OuterBlock) -> Result<Track<FixedTimeNoteSequence>, String> {
        // Build FixedTimeNoteSequence
        let sequence = self.build_fixed_time_note_sequence(&block.sequence_def)?;

        // Build TrackEffects
        let track_effects = self.build_track_effects(&block.envelope_defs, &block.effect_defs, &block.sequence_def)?;

        // Add notes to sequence
        let mut sequence_with_notes = sequence;
        for note_decl in &block.note_declarations {
            let playback_note = self.build_playback_note(note_decl, &block.sequence_def, &block.effect_defs)?;
            sequence_with_notes.append_note(playback_note);
        }

        // Build Track
        TrackBuilder::default()
            .sequence(sequence_with_notes)
            .effects(track_effects)
            .build()
            .map_err(|e| format!("Failed to build Track: {:?}", e))
    }

    fn build_fixed_time_note_sequence(&self, sequence_def: &SequenceDef) -> Result<FixedTimeNoteSequence, String> {
        FixedTimeNoteSequenceBuilder::default()
            .duration_type(sequence_def.dur)
            .tempo(sequence_def.tempo)
            .num_steps(sequence_def.num_steps)
            .build()
            .map_err(|e| format!("Failed to build FixedTimeNoteSequence: {:?}", e))
    }

    fn build_track_effects(&self, envelope_defs: &[EnvelopeDef], effect_defs: &[EffectDef], sequence_def: &SequenceDef) -> Result<TrackEffects, String> {
        let mut envelopes = Vec::new();
        let mut delays = Vec::new();
        let mut flangers = Vec::new();
        let mut lfos = Vec::new();
        let mut tremolos = Vec::new();
        let mut vibratos = Vec::new();
        let mut choruses = Vec::new();
        let mut equalizers = Vec::new();

        // Build envelopes
        for env_def in envelope_defs {
            let envelope = EnvelopeBuilder::default()
                .attack(EnvelopePair(env_def.attack.0, env_def.attack.1))
                .decay(EnvelopePair(env_def.decay.0, env_def.decay.1))
                .sustain(EnvelopePair(env_def.sustain.0, env_def.sustain.1))
                .release(EnvelopePair(env_def.release.0, env_def.release.1))
                .attack_curve(env_def.attack_curve)
                .decay_curve(env_def.decay_curve)
                .sustain_curve(env_def.sustain_curve)
                .release_curve(env_def.release_curve)
                .build()
                .map_err(|e| format!("Failed to build Envelope: {:?}", e))?;
            envelopes.push(envelope);
        }

        // Build effects
        for effect_def in effect_defs {
            match effect_def {
                EffectDef::Delay(delay_def) => {
                    let delay = DelayBuilder::default()
                        .id(0) // Default ID
                        .mix(delay_def.mix)
                        .decay(delay_def.decay)
                        .interval_ms(delay_def.interval_ms)
                        .duration_ms(delay_def.duration_ms)
                        .num_repeats(delay_def.num_repeats)
                        .num_predelay_samples(delay_def.num_predelay_samples)
                        .num_concurrent_sample_managers(delay_def.num_concurrent_delays)
                        .build()
                        .map_err(|e| format!("Failed to build Delay: {:?}", e))?;
                    delays.push(delay);
                }
                EffectDef::Flanger(flanger_def) => {
                    let flanger = FlangerBuilder::default()
                        .delay_ms(flanger_def.delay_ms)
                        .mix(flanger_def.mix)
                        .build()
                        .map_err(|e| format!("Failed to build Flanger: {:?}", e))?;
                    flangers.push(flanger);
                }
                EffectDef::LFO(lfo_def) => {
                    let waveforms: Vec<Waveform> = lfo_def.waveforms.iter()
                        .map(|w| w.to_waveform())
                        .collect();
                    let lfo = LFOBuilder::default()
                        .frequency(lfo_def.freq)
                        .amplitude(lfo_def.amp)
                        .waveforms(waveforms)
                        .build()
                        .map_err(|e| format!("Failed to build LFO: {:?}", e))?;
                    lfos.push(lfo);
                }
                EffectDef::Filter(_filter_def) => {
                    // Filters are added to individual notes, not track effects
                    // This is handled in build_playback_note
                }
                EffectDef::Tremolo(tremolo_def) => {
                    let tremolo = TremoloBuilder::default()
                        .mod_freq(tremolo_def.mod_freq)
                        .mod_depth(tremolo_def.mod_depth)
                        .build()
                        .map_err(|e| format!("Failed to build Tremolo: {:?}", e))?;
                    tremolos.push(tremolo);
                }
                EffectDef::Vibrato(vibrato_def) => {
                    let vibrato = VibratoBuilder::default()
                        .avg_delay(vibrato_def.avg_delay)
                        .mod_width(vibrato_def.mod_width)
                        .mod_freq(vibrato_def.mod_freq)
                        .build()
                        .map_err(|e| format!("Failed to build Vibrato: {:?}", e))?;
                    vibratos.push(vibrato);
                }
                EffectDef::Chorus(chorus_def) => {
                    let chorus = ChorusBuilder::default()
                        .chorus_count(chorus_def.chorus_count)
                        .chorus_gains(chorus_def.chorus_gains.clone())
                        .dry_gain(chorus_def.dry_gain)
                        .chorus_delays(chorus_def.chorus_delays.clone())
                        .mod_freqs(chorus_def.mod_freqs.clone())
                        .mod_widths(chorus_def.mod_widths.clone())
                        .build()
                        .map_err(|e| format!("Failed to build Chorus: {}", e))?;
                    choruses.push(chorus);
                }
                EffectDef::Equalizer(equalizer_def) => {
                    let equalizer = EqualizerBuilder::default()
                        .gains(equalizer_def.gains.clone())
                        .build()
                        .map_err(|e| format!("Failed to build Equalizer: {}", e))?;
                    equalizers.push(equalizer);
                }
            }
        }

        // Set panning and num_channels if panning is specified
        if let Some(panning_value) = sequence_def.panning {
            TrackEffectsBuilder::default()
                .envelopes(envelopes)
                .delays(delays)
                .flangers(flangers)
                .lfos(lfos)
                .tremolos(tremolos)
                .vibratos(vibratos)
                .choruses(choruses)
                .equalizers(equalizers)
                .panning(panning_value)
                .num_channels(2)
                .build()
                .map_err(|e| format!("Failed to build TrackEffects: {:?}", e))
        } else {
            TrackEffectsBuilder::default()
                .envelopes(envelopes)
                .delays(delays)
                .flangers(flangers)
                .lfos(lfos)
                .tremolos(tremolos)
                .vibratos(vibratos)
                .choruses(choruses)
                .equalizers(equalizers)
                .build()
                .map_err(|e| format!("Failed to build TrackEffects: {:?}", e))
        }
    }

    fn build_playback_note(&self, note_decl: &NoteDeclaration, sequence_def: &SequenceDef, effect_defs: &[EffectDef]) -> Result<PlaybackNote, String> {
        let step_duration_ms = (60000.0 / sequence_def.tempo as f32) * sequence_def.dur.to_factor();
        let start_time_ms = note_decl.get_step_index() as f32 * step_duration_ms;
        let end_time_ms = start_time_ms + step_duration_ms;

        // Build filters from effect definitions
        let mut filters = Vec::new();
        for effect_def in effect_defs {
            if let EffectDef::Filter(filter_def) = effect_def {
                let filter = LowPassFilterBuilder::default()
                    .cutoff_frequency(filter_def.cutoff_frequency)
                    .resonance(filter_def.resonance)
                    .mix(filter_def.mix)
                    .build()
                    .map_err(|e| format!("Failed to build Filter: {:?}", e))?;
                filters.push(filter);
            }
        }

        match note_decl {
            NoteDeclaration::Oscillator { waveforms, note_freq, volume, .. } => {
                let waveforms: Vec<Waveform> = waveforms.iter()
                    .map(|w| w.to_waveform())
                    .collect();

                let note = NoteBuilder::default()
                    .frequency(*note_freq)
                    .volume(*volume)
                    .start_time_ms(start_time_ms)
                    .end_time_ms(end_time_ms)
                    .waveforms(waveforms)
                    .build()
                    .map_err(|e| format!("Failed to build Note: {:?}", e))?;

                PlaybackNoteBuilder::default()
                    .note_source(NoteSource::Oscillator(note))
                    .playback_start_time_ms(start_time_ms)
                    .playback_end_time_ms(end_time_ms)
                    .filters(filters.clone())
                    .build()
                    .map_err(|e| format!("Failed to build PlaybackNote: {:?}", e))
            }
            NoteDeclaration::Sample { file_path, volume, .. } => {
                let sampled_note = SampledNoteBuilder::default()
                    .file_path(file_path.clone())
                    .volume(*volume)
                    .start_time_ms(start_time_ms)
                    .end_time_ms(end_time_ms)
                    .build()
                    .map_err(|e| format!("Failed to build SampledNote: {:?}", e))?;

                PlaybackNoteBuilder::default()
                    .note_source(NoteSource::Sample(sampled_note))
                    .playback_start_time_ms(start_time_ms)
                    .playback_end_time_ms(end_time_ms)
                    .filters(filters.clone())
                    .build()
                    .map_err(|e| format!("Failed to build PlaybackNote: {:?}", e))
            }
        }
    }

    fn parse_assignment(&mut self) -> Result<(String, String), String> {
        self.expect("let")?;
        let name = self.parse_identifier()?;
        self.expect("=")?;
        let expression = self.parse_expression()?;
        Ok((name, expression))
    }

    fn parse_identifier(&mut self) -> Result<String, String> {
        let token = self.advance();
        if token.chars().next().map_or(false, |c| c.is_alphabetic()) &&
           token.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            Ok(token)
        } else {
            Err(format!("Invalid identifier: {}", token))
        }
    }

    fn parse_expression(&mut self) -> Result<String, String> {
        let mut expression_tokens = Vec::new();
        
        // Parse until we reach the end of the line or encounter another 'let'
        while self.current < self.tokens.len() {
            let token = self.peek();
            
            // Stop if we encounter another 'let' (start of next macro definition)
            if token == "let" {
                break;
            }
            
            // Stop if we encounter 'FixedTimeNoteSequence' (start of outer block)
            if token == "FixedTimeNoteSequence" {
                break;
            }
            
            expression_tokens.push(self.advance());
        }
        
        if expression_tokens.is_empty() {
            return Err("Empty expression".to_string());
        }
        
        // Reconstruct the original text by joining tokens intelligently
        let mut expression = String::new();
        for (i, token) in expression_tokens.iter().enumerate() {
            if i > 0 {
                // Add space before token, except for certain punctuation
                let prev = &expression_tokens[i - 1];
                if token != "," && token != ":" && prev != "," && prev != ":" {
                    expression.push(' ');
                }
            }
            expression.push_str(token);
        }
        
        Ok(expression.trim().to_string())
    }
}

impl NoteDeclaration {
    fn get_step_index(&self) -> usize {
        match self {
            NoteDeclaration::Oscillator { step_index, .. } => *step_index,
            NoteDeclaration::Sample { step_index, .. } => *step_index,
        }
    }
}

pub(crate) fn parse_dsl(input: &str) -> Result<TrackGrid<FixedTimeNoteSequence>, String> {
    let mut parser = Parser::new(input)?;
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_script() {
        let input = r#"
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            a 0.1,0.8 d 0.3,0.6 s 0.8,0.4 r 1.0,0.0
            delay mix 0.5 decay 0.7 interval_ms 100.0 duration_ms 50.0 num_repeats 3 num_predelay_samples 10 num_concurrent_delays 2
            osc:sine:440.0:0.5:0
            osc:square:880.0:0.3:4
            samp:/Users/markweiss/RustroverProjects/osc_bak/src/dsl/test_data/test_sample.wav:0.005:2
        "#;

        let result = parse_dsl(input);
        if let Err(e) = &result {
            println!("Parse error: {}", e);
        }
        assert!(result.is_ok());
        
        let track_grid = result.unwrap();
        assert_eq!(track_grid.tracks.len(), 1);
        
        let track = &track_grid.tracks[0];
        assert_eq!(track.effects.envelopes.len(), 1);
        assert_eq!(track.effects.delays.len(), 1);
    }

    #[test]
    fn test_parse_multiple_blocks() {
        let input = r#"
            FixedTimeNoteSequence dur Eighth tempo 140 num_steps 8
            a 0.05,0.9 d 0.2,0.7 s 0.9,0.5 r 1.0,0.0
            osc:sine:220.0:0.4:0
            
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            flanger window_size 8 mix 0.3
            samp:/Users/markweiss/RustroverProjects/osc_bak/src/dsl/test_data/test_sample.wav:0.005:2
        "#;

        let result = parse_dsl(input);
        assert!(result.is_ok());
        
        let track_grid = result.unwrap();
        assert_eq!(track_grid.tracks.len(), 2);
    }

    #[test]
    fn test_parse_western_pitch() {
        let input = r#"
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            osc:sine:C:0.5:0
            osc:triangle:F#:0.3:4
        "#;

        let result = parse_dsl(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_octave_western_pitch() {
        let input = r#"
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            osc:sine:4,C:0.5:0
            osc:triangle:5,F#:0.3:4
            osc:square:3,A:0.7:8
        "#;

        let result = parse_dsl(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_complex_effects() {
        let input = r#"
            FixedTimeNoteSequence dur Half tempo 100 num_steps 32
            a 0.1,0.9 d 0.4,0.6 s 0.8,0.3 r 1.0,0.0
            delay mix 0.8 decay 0.6 interval_ms 80.0 duration_ms 40.0 num_repeats 5 num_predelay_samples 15 num_concurrent_delays 3
            flanger window_size 12 mix 0.4
            lfo freq 2.5 amp 0.3 waveforms sine,triangle
            osc:sine,square:440.0:0.7:0
        "#;

        let result = parse_dsl(input);
        assert!(result.is_ok());

        let track_grid = result.unwrap();
        let track = &track_grid.tracks[0];
        assert_eq!(track.effects.envelopes.len(), 1);
        assert_eq!(track.effects.delays.len(), 1);
        assert_eq!(track.effects.flangers.len(), 1);
        assert_eq!(track.effects.lfos.len(), 1);
    }

    #[test]
    fn test_parse_filter_effects() {
        let input = r#"
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            filter cutoff_frequency 1000.0 resonance 0.3 mix 0.8
            osc:sine:440.0:0.5:0
        "#;

        let result = parse_dsl(input);
        assert!(result.is_ok());

        let track_grid = result.unwrap();
        let track = &track_grid.tracks[0];
        let sequence = &track.sequence;
        
        // Check that the note has a filter
        let all_notes = sequence.get_all_notes();
        assert_eq!(all_notes.len(), 1);
        let note = &all_notes[0];
        assert_eq!(note.filters.len(), 1);
        
        // Check filter parameters
        let filter = &note.filters[0];
        assert_eq!(filter.cutoff_frequency, 1000.0);
        assert_eq!(filter.resonance, 0.3);
        assert_eq!(filter.mix, 0.8);
    }

    #[test]
    fn test_parse_multiple_filters() {
        let input = r#"
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            filter cutoff_frequency 500.0 resonance 0.2 mix 0.6
            filter cutoff_frequency 2000.0 resonance 0.5 mix 0.4
            osc:sine:440.0:0.5:0
        "#;

        let result = parse_dsl(input);
        assert!(result.is_ok());

        let track_grid = result.unwrap();
        let track = &track_grid.tracks[0];
        let sequence = &track.sequence;
        
        // Check that the note has multiple filters
        let all_notes = sequence.get_all_notes();
        assert_eq!(all_notes.len(), 1);
        let note = &all_notes[0];
        assert_eq!(note.filters.len(), 2);
        
        // Check first filter parameters
        let filter1 = &note.filters[0];
        assert_eq!(filter1.cutoff_frequency, 500.0);
        assert_eq!(filter1.resonance, 0.2);
        assert_eq!(filter1.mix, 0.6);
        
        // Check second filter parameters
        let filter2 = &note.filters[1];
        assert_eq!(filter2.cutoff_frequency, 2000.0);
        assert_eq!(filter2.resonance, 0.5);
        assert_eq!(filter2.mix, 0.4);
    }

    #[test]
    fn test_parse_track_panning() {
        let input = r#"
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16 panning -0.5
            osc:sine:440.0:0.5:0

            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16 panning 0.7
            osc:square:880.0:0.3:4

            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            osc:triangle:220.0:0.4:8
        "#;

        let result = parse_dsl(input);
        assert!(result.is_ok());

        let track_grid = result.unwrap();
        assert_eq!(track_grid.tracks.len(), 3);

        // First track with left panning
        let track1 = &track_grid.tracks[0];
        assert_eq!(track1.effects.panning, -0.5);
        assert_eq!(track1.effects.num_channels, 2);

        // Second track with right panning
        let track2 = &track_grid.tracks[1];
        assert_eq!(track2.effects.panning, 0.7);
        assert_eq!(track2.effects.num_channels, 2);

        // Third track with no panning (default)
        let track3 = &track_grid.tracks[2];
        assert_eq!(track3.effects.panning, 0.0);
        assert_eq!(track3.effects.num_channels, 1);
    }

    #[test]
    fn test_parse_track_panning_keyword_required() {
        // Test that a float after num_steps without "panning" keyword is not parsed as panning
        let input = r#"
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            a 0.1,0.9 d 0.4,0.6 s 0.8,0.3 r 1.0,0.0
            osc:sine:440.0:0.5:0
        "#;

        let result = parse_dsl(input);
        assert!(result.is_ok());

        let track_grid = result.unwrap();
        assert_eq!(track_grid.tracks.len(), 1);

        // Track should have default panning (no panning specified)
        let track = &track_grid.tracks[0];
        assert_eq!(track.effects.panning, 0.0);
        assert_eq!(track.effects.num_channels, 1);
    }

    #[test]
    fn test_parse_macro_definitions() {
        let input = r#"
            let env1 = a 0.2,0.8 d 0.3,0.6 s 0.8,0.5 r 1.0,0.0
            let delay1 = delay mix 0.5 decay 0.7 interval_ms 100.0 duration_ms 50.0 num_repeats 3 num_predelay_samples 10 num_concurrent_delays 2
            let flanger1 = flanger window_size 8 mix 0.3
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            osc:sine:440.0:0.5:0
        "#;

        let mut parser = Parser::new(input).unwrap();
        let script = parser.parse_script().unwrap();
        
        // Verify macro definitions are parsed correctly
        assert_eq!(script.macro_defs.len(), 3);
        
        // Check that env1 contains the envelope definition
        assert_eq!(script.macro_defs.get("env1").unwrap(), "a 0.2,0.8 d 0.3,0.6 s 0.8,0.5 r 1.0,0.0");
        
        // Check that delay1 contains the delay definition
        assert_eq!(script.macro_defs.get("delay1").unwrap(), "delay mix 0.5 decay 0.7 interval_ms 100.0 duration_ms 50.0 num_repeats 3 num_predelay_samples 10 num_concurrent_delays 2");
        
        // Check that flanger1 contains the flanger definition
        assert_eq!(script.macro_defs.get("flanger1").unwrap(), "flanger window_size 8 mix 0.3");
        
        // Verify that outer blocks are still parsed correctly
        assert_eq!(script.outer_blocks.len(), 1);
    }

    #[test]
    fn test_parse_macro_definitions_with_whitespace() {
        let input = r#"
            let env1 =   a 0.2,0.8 d 0.3,0.6 s 0.8,0.5 r 1.0,0.0   
            let delay1 = delay mix 0.5 decay 0.7 interval_ms 100.0 duration_ms 50.0 num_repeats 3 num_predelay_samples 10 num_concurrent_delays 2
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            osc:sine:440.0:0.5:0
        "#;

        let mut parser = Parser::new(input).unwrap();
        let script = parser.parse_script().unwrap();
        
        // Verify that whitespace is trimmed from expressions
        assert_eq!(script.macro_defs.get("env1").unwrap(), "a 0.2,0.8 d 0.3,0.6 s 0.8,0.5 r 1.0,0.0");
        assert_eq!(script.macro_defs.get("delay1").unwrap(), "delay mix 0.5 decay 0.7 interval_ms 100.0 duration_ms 50.0 num_repeats 3 num_predelay_samples 10 num_concurrent_delays 2");
    }

    #[test]
    fn test_parse_multiple_macro_definitions() {
        let input = r#"
            let env1 = a 0.2,0.8 d 0.3,0.6 s 0.8,0.5 r 1.0,0.0
            let env2 = a 0.1,0.9 d 0.2,0.7 s 0.9,0.4 r 0.8,0.1
            let delay1 = delay mix 0.5 decay 0.7 interval_ms 100.0 duration_ms 50.0 num_repeats 3 num_predelay_samples 10 num_concurrent_delays 2
            let lfo1 = lfo freq 2.0 amp 0.3 waveforms sine,triangle
            let note1 = osc:sine:440.0:0.5:0
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            osc:sine:880.0:0.3:4
        "#;

        let mut parser = Parser::new(input).unwrap();
        let script = parser.parse_script().unwrap();
        
        // Verify all macro definitions are parsed
        assert_eq!(script.macro_defs.len(), 5);
        
        // Check each macro definition
        assert_eq!(script.macro_defs.get("env1").unwrap(), "a 0.2,0.8 d 0.3,0.6 s 0.8,0.5 r 1.0,0.0");
        assert_eq!(script.macro_defs.get("env2").unwrap(), "a 0.1,0.9 d 0.2,0.7 s 0.9,0.4 r 0.8,0.1");
        assert_eq!(script.macro_defs.get("delay1").unwrap(), "delay mix 0.5 decay 0.7 interval_ms 100.0 duration_ms 50.0 num_repeats 3 num_predelay_samples 10 num_concurrent_delays 2");
        assert_eq!(script.macro_defs.get("lfo1").unwrap(), "lfo freq 2.0 amp 0.3 waveforms sine,triangle");
        assert_eq!(script.macro_defs.get("note1").unwrap(), "osc:sine:440.0:0.5:0");
        
        // Verify that outer blocks are still parsed correctly
        assert_eq!(script.outer_blocks.len(), 1);
    }

    #[test]
    fn test_parse_identifier_validation() {
        let input = r#"
            let valid-name = a 0.2,0.8 d 0.3,0.6 s 0.8,0.5 r 1.0,0.0
            let valid_name = delay mix 0.5 decay 0.7 interval_ms 100.0 duration_ms 50.0 num_repeats 3 num_predelay_samples 10 num_concurrent_delays 2
            let validName123 = flanger window_size 8 mix 0.3
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            osc:sine:440.0:0.5:0
        "#;

        let mut parser = Parser::new(input).unwrap();
        let script = parser.parse_script().unwrap();
        
        // Verify that valid identifiers with hyphens, underscores, and numbers are accepted
        assert_eq!(script.macro_defs.len(), 3);
        assert!(script.macro_defs.contains_key("valid-name"));
        assert!(script.macro_defs.contains_key("valid_name"));
        assert!(script.macro_defs.contains_key("validName123"));
    }

    #[test]
    fn test_macro_expansion_basic() {
        let input = r#"
            let env1 = a 0.2,0.8 d 0.3,0.6 s 0.8,0.5 r 1.0,0.0
            let delay1 = delay mix 0.5 decay 0.7 interval_ms 100.0 duration_ms 50.0 num_repeats 3 num_predelay_samples 10 num_concurrent_delays 2
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            $env1
            $delay1
            osc:sine:440.0:0.5:0
        "#;

        let result = parse_dsl(input);
        assert!(result.is_ok());
        
        let track_grid = result.unwrap();
        assert_eq!(track_grid.tracks.len(), 1);
        
        let track = &track_grid.tracks[0];
        // Should have one envelope and one delay from the expanded macros
        assert_eq!(track.effects.envelopes.len(), 1);
        assert_eq!(track.effects.delays.len(), 1);
    }

    #[test]
    fn test_macro_expansion_multiple_uses() {
        let input = r#"
            let env1 = a 0.2,0.8 d 0.3,0.6 s 0.8,0.5 r 1.0,0.0
            let flanger1 = flanger window_size 8 mix 0.3
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            $env1
            $flanger1
            osc:sine:440.0:0.5:0
            
            FixedTimeNoteSequence dur Eighth tempo 140 num_steps 8
            $env1
            $flanger1
            osc:square:880.0:0.3:4
        "#;

        let result = parse_dsl(input);
        assert!(result.is_ok());
        
        let track_grid = result.unwrap();
        assert_eq!(track_grid.tracks.len(), 2);
        
        // Both tracks should have the same envelope and flanger from expanded macros
        for track in &track_grid.tracks {
            assert_eq!(track.effects.envelopes.len(), 1);
            assert_eq!(track.effects.flangers.len(), 1);
        }
    }

    #[test]
    fn test_macro_expansion_nested() {
        let input = r#"
            let base_env = a 0.1,0.9 d 0.2,0.7 s 0.9,0.4 r 0.8,0.1
            let env1 = $base_env
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            $env1
            osc:sine:440.0:0.5:0
        "#;

        let result = parse_dsl(input);
        assert!(result.is_ok());
        
        let track_grid = result.unwrap();
        assert_eq!(track_grid.tracks.len(), 1);
        
        let track = &track_grid.tracks[0];
        // Should have one envelope from the expanded macro
        assert_eq!(track.effects.envelopes.len(), 1);
    }

    #[test]
    fn test_macro_expansion_in_sequence() {
        let input = r#"
            let my_tempo = 140
            let my_steps = 8
            FixedTimeNoteSequence dur Quarter tempo $my_tempo num_steps $my_steps
            osc:sine:440.0:0.5:0
        "#;

        let result = parse_dsl(input);
        assert!(result.is_ok());
        
        let track_grid = result.unwrap();
        assert_eq!(track_grid.tracks.len(), 1);
        
        let track = &track_grid.tracks[0];
        // The sequence should have been expanded with the macro values
        assert_eq!(track.sequence.tempo, 140);
    }

    #[test]
    fn test_macro_expansion_undefined() {
        let input = r#"
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            $undefined_macro
            osc:sine:440.0:0.5:0
        "#;

        let result = parse_dsl(input);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Undefined macro '$undefined_macro'"));
    }

    #[test]
    fn test_macro_expansion_complex() {
        let input = r#"
            let attack_params = 0.2,0.8
            let decay_params = 0.3,0.6
            let sustain_params = 0.8,0.5
            let release_params = 1.0,0.0
            let env1 = a $attack_params d $decay_params s $sustain_params r $release_params
            let delay1 = delay mix 0.5 decay 0.7 interval_ms 100.0 duration_ms 50.0 num_repeats 3 num_predelay_samples 10 num_concurrent_delays 2
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            $env1
            $delay1
            osc:sine:440.0:0.5:0
        "#;

        let result = parse_dsl(input);
        assert!(result.is_ok());
        
        let track_grid = result.unwrap();
        assert_eq!(track_grid.tracks.len(), 1);
        
        let track = &track_grid.tracks[0];
        // Should have one envelope and one delay from the expanded macros
        assert_eq!(track.effects.envelopes.len(), 1);
        assert_eq!(track.effects.delays.len(), 1);
    }

    #[test]
    fn test_macro_expansion_with_filters() {
        let input = r#"
            let filter1 = filter cutoff_frequency 1000.0 resonance 0.3 mix 0.8
            let filter2 = filter cutoff_frequency 500.0 resonance 0.2 mix 0.6
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            $filter1
            $filter2
            osc:sine:440.0:0.5:0
        "#;

        let result = parse_dsl(input);
        assert!(result.is_ok());
        
        let track_grid = result.unwrap();
        assert_eq!(track_grid.tracks.len(), 1);
        
        let track = &track_grid.tracks[0];
        let sequence = &track.sequence;
        
        // Check that the note has both filters from the expanded macros
        let all_notes = sequence.get_all_notes();
        assert_eq!(all_notes.len(), 1);
        let note = &all_notes[0];
        assert_eq!(note.filters.len(), 2);
        
        // Check first filter parameters
        let filter1 = &note.filters[0];
        assert_eq!(filter1.cutoff_frequency, 1000.0);
        assert_eq!(filter1.resonance, 0.3);
        assert_eq!(filter1.mix, 0.8);
        
        // Check second filter parameters
        let filter2 = &note.filters[1];
        assert_eq!(filter2.cutoff_frequency, 500.0);
        assert_eq!(filter2.resonance, 0.2);
        assert_eq!(filter2.mix, 0.6);
    }

    #[test]
    fn test_parse_envelope_with_curve_types() {
        let input = r#"
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            a 0.1,0.8,exp d 0.3,0.6,lin s 0.8,0.4,lin r 1.0,0.0,exp
            osc:sine:440.0:0.5:0
        "#;

        let result = parse_dsl(input);
        if let Err(e) = &result {
            println!("Parse error: {}", e);
        }
        assert!(result.is_ok());

        let track_grid = result.unwrap();
        assert_eq!(track_grid.tracks.len(), 1);

        let track = &track_grid.tracks[0];
        assert_eq!(track.effects.envelopes.len(), 1);

        let envelope = &track.effects.envelopes[0];
        assert_eq!(envelope.attack_curve, EnvelopeCurve::Exponential);
        assert_eq!(envelope.decay_curve, EnvelopeCurve::Linear);
        assert_eq!(envelope.sustain_curve, EnvelopeCurve::Linear);
        assert_eq!(envelope.release_curve, EnvelopeCurve::Exponential);
    }

    #[test]
    fn test_parse_envelope_without_curve_types_defaults_linear() {
        let input = r#"
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            a 0.1,0.8 d 0.3,0.6 s 0.8,0.4 r 1.0,0.0
            osc:sine:440.0:0.5:0
        "#;

        let result = parse_dsl(input);
        assert!(result.is_ok());

        let track_grid = result.unwrap();
        let track = &track_grid.tracks[0];
        let envelope = &track.effects.envelopes[0];

        assert_eq!(envelope.attack_curve, EnvelopeCurve::Linear);
        assert_eq!(envelope.decay_curve, EnvelopeCurve::Linear);
        assert_eq!(envelope.sustain_curve, EnvelopeCurve::Linear);
        assert_eq!(envelope.release_curve, EnvelopeCurve::Linear);
    }

    #[test]
    fn test_parse_envelope_mixed_curve_some_omitted() {
        let input = r#"
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            a 0.1,0.8,exp d 0.3,0.6 s 0.8,0.4,exp r 1.0,0.0
            osc:sine:440.0:0.5:0
        "#;

        let result = parse_dsl(input);
        assert!(result.is_ok());

        let track_grid = result.unwrap();
        let track = &track_grid.tracks[0];
        let envelope = &track.effects.envelopes[0];

        assert_eq!(envelope.attack_curve, EnvelopeCurve::Exponential);
        assert_eq!(envelope.decay_curve, EnvelopeCurve::Linear);
        assert_eq!(envelope.sustain_curve, EnvelopeCurve::Exponential);
        assert_eq!(envelope.release_curve, EnvelopeCurve::Linear);
    }

    #[test]
    fn test_parse_tremolo() {
        let input = r#"
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            tremolo mod_freq 5.0 mod_depth 0.5
            osc:sine:440.0:0.5:0
        "#;

        let result = parse_dsl(input);
        if let Err(e) = &result {
            println!("Parse error: {}", e);
        }
        assert!(result.is_ok());

        let track_grid = result.unwrap();
        let track = &track_grid.tracks[0];
        assert_eq!(track.effects.tremolos.len(), 1);
        assert_eq!(track.effects.tremolos[0].mod_freq, 5.0);
        assert_eq!(track.effects.tremolos[0].mod_depth, 0.5);
    }

    #[test]
    fn test_parse_vibrato() {
        let input = r#"
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            vibrato avg_delay 0.007 mod_width 0.003 mod_freq 5.0
            osc:sine:440.0:0.5:0
        "#;

        let result = parse_dsl(input);
        if let Err(e) = &result {
            println!("Parse error: {}", e);
        }
        assert!(result.is_ok());

        let track_grid = result.unwrap();
        let track = &track_grid.tracks[0];
        assert_eq!(track.effects.vibratos.len(), 1);
        assert_eq!(track.effects.vibratos[0].avg_delay, 0.007);
        assert_eq!(track.effects.vibratos[0].mod_width, 0.003);
        assert_eq!(track.effects.vibratos[0].mod_freq, 5.0);
    }

    #[test]
    fn test_parse_chorus() {
        let input = r#"
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            chorus chorus_count 3 chorus_gains 0.4,0.4,0.4 dry_gain 0.7 chorus_delays 0.015,0.020,0.030 mod_freqs 0.25,0.33,0.40 mod_widths 0.003,0.004,0.005
            osc:sine:440.0:0.5:0
        "#;

        let result = parse_dsl(input);
        if let Err(e) = &result {
            println!("Parse error: {}", e);
        }
        assert!(result.is_ok());

        let track_grid = result.unwrap();
        let track = &track_grid.tracks[0];
        assert_eq!(track.effects.choruses.len(), 1);
        assert_eq!(track.effects.choruses[0].chorus_count, 3);
        assert_eq!(track.effects.choruses[0].chorus_gains, vec![0.4, 0.4, 0.4]);
        assert_eq!(track.effects.choruses[0].dry_gain, 0.7);
        assert_eq!(track.effects.choruses[0].chorus_delays, vec![0.015, 0.020, 0.030]);
        assert_eq!(track.effects.choruses[0].mod_freqs, vec![0.25, 0.33, 0.40]);
        assert_eq!(track.effects.choruses[0].mod_widths, vec![0.003, 0.004, 0.005]);
    }

    #[test]
    fn test_parse_equalizer() {
        let input = r#"
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            equalizer gains 0.0,3.0,-2.0,0.0,0.0,0.0,0.0,0.0
            osc:sine:440.0:0.5:0
        "#;

        let result = parse_dsl(input);
        if let Err(e) = &result {
            println!("Parse error: {}", e);
        }
        assert!(result.is_ok());

        let track_grid = result.unwrap();
        let track = &track_grid.tracks[0];
        assert_eq!(track.effects.equalizers.len(), 1);
        assert_eq!(track.effects.equalizers[0].gains, vec![0.0, 3.0, -2.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_parse_all_new_effects() {
        let input = r#"
            FixedTimeNoteSequence dur Half tempo 100 num_steps 32
            a 0.1,0.9 d 0.4,0.6 s 0.8,0.3 r 1.0,0.0
            delay mix 0.8 decay 0.6 interval_ms 80.0 duration_ms 40.0 num_repeats 5 num_predelay_samples 15 num_concurrent_delays 3
            tremolo mod_freq 6.0 mod_depth 0.4
            vibrato avg_delay 0.005 mod_width 0.002 mod_freq 4.0
            chorus chorus_count 2 chorus_gains 0.3,0.3 dry_gain 0.8 chorus_delays 0.010,0.020 mod_freqs 0.3,0.4 mod_widths 0.002,0.003
            equalizer gains 0.0,0.0,3.0,0.0,0.0,0.0,0.0,0.0
            osc:sine:440.0:0.7:0
        "#;

        let result = parse_dsl(input);
        if let Err(e) = &result {
            println!("Parse error: {}", e);
        }
        assert!(result.is_ok());

        let track_grid = result.unwrap();
        let track = &track_grid.tracks[0];
        assert_eq!(track.effects.envelopes.len(), 1);
        assert_eq!(track.effects.delays.len(), 1);
        assert_eq!(track.effects.tremolos.len(), 1);
        assert_eq!(track.effects.vibratos.len(), 1);
        assert_eq!(track.effects.choruses.len(), 1);
        assert_eq!(track.effects.equalizers.len(), 1);
    }
}