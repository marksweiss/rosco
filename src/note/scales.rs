use crate::note::constants::PITCH_TO_FREQ_HZ;
use std::fmt;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum WesternPitch {
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

#[allow(dead_code)]
pub(crate) enum WesternScale {
    Major,
    Minor,
    Pentatonic,
    Blues, // TODO CHATTY BROKEN
    Chromatic,
}

#[allow(dead_code)]
pub(crate) enum ArabicScale {
    Hijaz,
    Bayati,
    Rast,
    Saba,
}

#[allow(dead_code)]
impl WesternPitch {
    pub fn get_pitch_index(&self) -> u8 {
        match self {
            WesternPitch::C => 0,
            WesternPitch::CSharp => 1,
            WesternPitch::DFlat => 1,
            WesternPitch::D => 2,
            WesternPitch::DSharp => 3,
            WesternPitch::EFlat => 3,
            WesternPitch::E => 4,
            WesternPitch::F => 5,
            WesternPitch::FSharp => 6,
            WesternPitch::GFlat => 6,
            WesternPitch::G => 7,
            WesternPitch::GSharp => 8,
            WesternPitch::AFlat => 8,
            WesternPitch::A => 9,
            WesternPitch::ASharp => 10,
            WesternPitch::BFlat => 10,
            WesternPitch::B => 11,
        }
    }
    
    pub fn get_frequency(&self, octave: u8) -> f32 {
        PITCH_TO_FREQ_HZ[(octave * 12 + self.get_pitch_index()) as usize] as f32
    }

    pub fn all_pitches() -> [WesternPitch; 12] {
        [
            WesternPitch::C,
            WesternPitch::CSharp,
            WesternPitch::D,
            WesternPitch::DSharp,
            WesternPitch::E,
            WesternPitch::F,
            WesternPitch::FSharp,
            WesternPitch::G,
            WesternPitch::GSharp,
            WesternPitch::A,
            WesternPitch::ASharp,
            WesternPitch::B,
        ]
    }

    pub fn next(&self) -> WesternPitch {
        let pitches = Self::all_pitches();
        let current_idx = pitches.iter().position(|p| *p == *self).unwrap_or(0);
        pitches[(current_idx + 1) % pitches.len()]
    }

    pub fn previous(&self) -> WesternPitch {
        let pitches = Self::all_pitches();
        let current_idx = pitches.iter().position(|p| *p == *self).unwrap_or(0);
        pitches[(current_idx + pitches.len() - 1) % pitches.len()]
    }
}

impl fmt::Display for WesternPitch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WesternPitch::C => write!(f, "C"),
            WesternPitch::CSharp => write!(f, "C#"),
            WesternPitch::DFlat => write!(f, "Db"),
            WesternPitch::D => write!(f, "D"),
            WesternPitch::DSharp => write!(f, "D#"),
            WesternPitch::EFlat => write!(f, "Eb"),
            WesternPitch::E => write!(f, "E"),
            WesternPitch::F => write!(f, "F"),
            WesternPitch::FSharp => write!(f, "F#"),
            WesternPitch::GFlat => write!(f, "Gb"),
            WesternPitch::G => write!(f, "G"),
            WesternPitch::GSharp => write!(f, "G#"),
            WesternPitch::AFlat => write!(f, "Ab"),
            WesternPitch::A => write!(f, "A"),
            WesternPitch::ASharp => write!(f, "A#"),
            WesternPitch::BFlat => write!(f, "Bb"),
            WesternPitch::B => write!(f, "B"),
        }
    }
}

#[allow(dead_code)]
impl WesternScale {
    pub(crate) fn get_scale(&self, root_pitch: u8) -> Vec<f32> {
        let mut scale = Vec::new();
        let root_freq = PITCH_TO_FREQ_HZ[root_pitch as usize] as f32;
        match self {
            WesternScale::Major => {
                scale.push(root_freq);
                scale.push(root_freq * 9.0 / 8.0);
                scale.push(root_freq * 5.0 / 4.0);
                scale.push(root_freq * 4.0 / 3.0);
                scale.push(root_freq * 3.0 / 2.0);
                scale.push(root_freq * 5.0 / 3.0);
                scale.push(root_freq * 15.0 / 8.0);
            }
            WesternScale::Minor => {
                scale.push(root_freq);
                scale.push(root_freq * 9.0 / 8.0);
                scale.push(root_freq * 6.0 / 5.0);
                scale.push(root_freq * 4.0 / 3.0);
                scale.push(root_freq * 3.0 / 2.0);
                scale.push(root_freq * 8.0 / 5.0);
                scale.push(root_freq * 9.0 / 5.0);
            }
            WesternScale::Pentatonic => {
                scale.push(root_freq);
                scale.push(root_freq * 9.0 / 8.0);
                scale.push(root_freq * 6.0 / 5.0);
                scale.push(root_freq * 4.0 / 3.0);
                scale.push(root_freq * 3.0 / 2.0);
            }
            WesternScale::Blues => {
                scale.push(root_freq);
                scale.push(root_freq * 6.0 / 5.0);
                scale.push(root_freq * 7.0 / 5.0);
                scale.push(root_freq * 7.0 / 6.0);
                scale.push(root_freq * 9.0 / 5.0);
            }
            WesternScale::Chromatic => {
                for i in 0..12 {
                    scale.push(root_freq * 2.0_f32.powf(i as f32 / 12.0));
                }
            }
        }
        
        scale
    }
}

// TODO ABSOLUTELY NO IDEA IF THIS IS CORRECT
#[allow(dead_code)]
impl ArabicScale {
    pub(crate) fn get_scale(&self, root_pitch: u8) -> Vec<f32> {
        let mut scale = Vec::new();
        let root_freq = PITCH_TO_FREQ_HZ[root_pitch as usize] as f32;
        match self {
            ArabicScale::Hijaz => {
                scale.push(root_freq);
                scale.push(root_freq * 16.0 / 15.0);
                scale.push(root_freq * 10.0 / 9.0);
                scale.push(root_freq * 4.0 / 3.0);
                scale.push(root_freq * 3.0 / 2.0);
                scale.push(root_freq * 8.0 / 5.0);
                scale.push(root_freq * 16.0 / 9.0);
            }
            ArabicScale::Bayati => {
                scale.push(root_freq);
                scale.push(root_freq * 16.0 / 15.0);
                scale.push(root_freq * 10.0 / 9.0);
                scale.push(root_freq * 4.0 / 3.0);
                scale.push(root_freq * 3.0 / 2.0);
                scale.push(root_freq * 8.0 / 5.0);
                scale.push(root_freq * 16.0 / 9.0);
            }
            ArabicScale::Rast => {
                scale.push(root_freq);
                scale.push(root_freq * 9.0 / 8.0);
                scale.push(root_freq * 5.0 / 4.0);
                scale.push(root_freq * 4.0 / 3.0);
                scale.push(root_freq * 3.0 / 2.0);
                scale.push(root_freq * 5.0 / 3.0);
                scale.push(root_freq * 15.0 / 8.0);
            }
            ArabicScale::Saba => {
                scale.push(root_freq);
                scale.push(root_freq * 9.0 / 8.0);
                scale.push(root_freq * 6.0 / 5.0);
                scale.push(root_freq * 4.0 / 3.0);
                scale.push(root_freq * 3.0 / 2.0);
                scale.push(root_freq * 8.0 / 5.0);
                scale.push(root_freq * 9.0 / 5.0);
            }
        }
        
        scale
    }
}
