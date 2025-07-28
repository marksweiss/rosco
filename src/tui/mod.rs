pub mod app;
pub mod audio_bridge;
pub mod config;
pub mod events;
pub mod ui;
pub mod track_bridge;
pub mod pattern_manager;

pub use app::RoscoTuiApp;
pub use config::TuiConfig;
pub use track_bridge::TrackBridge;
pub use pattern_manager::{PatternManager, Pattern};

use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum TuiError {
    Io(std::io::Error),
    Audio(String),
    Config(String),
    Terminal(String),
}

impl fmt::Display for TuiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TuiError::Io(err) => write!(f, "IO error: {}", err),
            TuiError::Audio(msg) => write!(f, "Audio error: {}", msg),
            TuiError::Config(msg) => write!(f, "Config error: {}", msg),
            TuiError::Terminal(msg) => write!(f, "Terminal error: {}", msg),
        }
    }
}

impl Error for TuiError {}

impl From<std::io::Error> for TuiError {
    fn from(err: std::io::Error) -> Self {
        TuiError::Io(err)
    }
}