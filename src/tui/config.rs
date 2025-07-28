use crate::tui::{TuiError, app::SynthParameters};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuiConfig {
    // Display preferences
    pub theme: ColorTheme,
    pub layout: LayoutPreferences,
    
    // Audio settings
    pub audio_device: Option<String>,
    pub sample_rate: u32,
    pub buffer_size: u32,
    
    // Keyboard mappings
    pub key_bindings: HashMap<String, String>,
    
    // Synthesizer defaults
    pub default_synth_params: SynthParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorTheme {
    pub name: String,
    pub focused_border: String,
    pub unfocused_border: String,
    pub highlight: String,
    pub background: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutPreferences {
    pub minimum_width: u16,
    pub minimum_height: u16,
    pub synthesizer_height_percent: u16,
    pub sequencer_height_percent: u16,
}

impl Default for TuiConfig {
    fn default() -> Self {
        let mut key_bindings = HashMap::new();
        key_bindings.insert("quit".to_string(), "q".to_string());
        key_bindings.insert("help".to_string(), "F1".to_string());
        key_bindings.insert("focus_next".to_string(), "Tab".to_string());
        key_bindings.insert("play_stop".to_string(), "Space".to_string());
        
        Self {
            theme: ColorTheme::default(),
            layout: LayoutPreferences::default(),
            audio_device: None,
            sample_rate: 44100,
            buffer_size: 512,
            key_bindings,
            default_synth_params: SynthParameters::default(),
        }
    }
}

impl Default for ColorTheme {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            focused_border: "cyan".to_string(),
            unfocused_border: "white".to_string(),
            highlight: "yellow".to_string(),
            background: "black".to_string(),
        }
    }
}

impl Default for LayoutPreferences {
    fn default() -> Self {
        Self {
            minimum_width: 80,
            minimum_height: 24,
            synthesizer_height_percent: 40,
            sequencer_height_percent: 55,
        }
    }
}

impl TuiConfig {
    pub fn load_or_default() -> Result<Self, TuiError> {
        match Self::load() {
            Ok(config) => Ok(config),
            Err(_) => {
                let config = Self::default();
                config.save()?;
                Ok(config)
            }
        }
    }
    
    pub fn load() -> Result<Self, TuiError> {
        let config_path = Self::config_file_path()?;
        let content = std::fs::read_to_string(config_path)
            .map_err(|e| TuiError::Config(format!("Failed to read config file: {}", e)))?;
        
        toml::from_str(&content)
            .map_err(|e| TuiError::Config(format!("Failed to parse config file: {}", e)))
    }
    
    pub fn save(&self) -> Result<(), TuiError> {
        let config_path = Self::config_file_path()?;
        
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| TuiError::Config(format!("Failed to create config directory: {}", e)))?;
        }
        
        let content = toml::to_string_pretty(self)
            .map_err(|e| TuiError::Config(format!("Failed to serialize config: {}", e)))?;
        
        std::fs::write(config_path, content)
            .map_err(|e| TuiError::Config(format!("Failed to write config file: {}", e)))?;
        
        Ok(())
    }
    
    fn config_file_path() -> Result<PathBuf, TuiError> {
        let mut path = dirs::config_dir()
            .ok_or_else(|| TuiError::Config("Could not determine config directory".to_string()))?;
        path.push("rosco");
        path.push("tui_config.toml");
        Ok(path)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub synth_params: SynthParameters,
    pub tempo: f32,
    pub transport_playing: bool,
}

impl SessionState {
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<(), TuiError> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| TuiError::Config(format!("Failed to serialize session: {}", e)))?;
        
        std::fs::write(path, content)
            .map_err(|e| TuiError::Config(format!("Failed to write session file: {}", e)))?;
        
        Ok(())
    }
    
    pub fn load_from_file(path: &std::path::Path) -> Result<Self, TuiError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| TuiError::Config(format!("Failed to read session file: {}", e)))?;
        
        serde_json::from_str(&content)
            .map_err(|e| TuiError::Config(format!("Failed to parse session file: {}", e)))
    }
}