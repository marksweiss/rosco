use std::fmt;

#[derive(Debug)]
pub enum RoscoError {
    Parse(String),
    Io(String),
    Midi(String),
    Audio(String),
}

impl fmt::Display for RoscoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RoscoError::Parse(msg) => write!(f, "Parse error: {}", msg),
            RoscoError::Io(msg) => write!(f, "IO error: {}", msg),
            RoscoError::Midi(msg) => write!(f, "MIDI error: {}", msg),
            RoscoError::Audio(msg) => write!(f, "Audio error: {}", msg),
        }
    }
}

impl std::error::Error for RoscoError {}

impl From<std::io::Error> for RoscoError {
    fn from(err: std::io::Error) -> Self {
        RoscoError::Io(err.to_string())
    }
}

impl From<String> for RoscoError {
    fn from(msg: String) -> Self {
        RoscoError::Parse(msg)
    }
}
