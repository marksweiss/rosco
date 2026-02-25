use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::effect_chains::EffectChainsState;
use super::effects::{EffectsRackState, EqualizersState};
use super::envelope::EnvelopesState;
use super::oscillator::OscillatorChainsState;
use super::sequencer::{TrackStrip, NUM_TRACKS};
use super::theme::GuiTheme;

// --- GUI configuration (persisted across sessions) ---

#[derive(Serialize, Deserialize)]
pub struct GuiConfig {
    pub theme_name: String,
    pub recent_files: Vec<String>,
}

impl Default for GuiConfig {
    fn default() -> Self {
        Self {
            theme_name: "Dark".to_string(),
            recent_files: Vec::new(),
        }
    }
}

impl GuiConfig {
    fn config_dir() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("rosco"))
    }

    fn config_path() -> Option<PathBuf> {
        Self::config_dir().map(|d| d.join("gui_config.toml"))
    }

    pub fn load() -> Self {
        Self::config_path()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .and_then(|contents| toml::from_str(&contents).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        if let Some(dir) = Self::config_dir() {
            let _ = std::fs::create_dir_all(&dir);
            if let Some(path) = Self::config_path() {
                if let Ok(contents) = toml::to_string_pretty(self) {
                    let _ = std::fs::write(path, contents);
                }
            }
        }
    }

    pub fn add_recent_file(&mut self, path: &str) {
        self.recent_files.retain(|p| p != path);
        self.recent_files.insert(0, path.to_string());
        self.recent_files.truncate(10);
    }

    pub fn resolve_theme(&self) -> GuiTheme {
        match self.theme_name.as_str() {
            "Light" => GuiTheme::light(),
            _ => GuiTheme::dark(),
        }
    }
}

// --- Session state (full GUI state snapshot) ---

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct SessionState {
    #[serde(default)]
    pub oscillator_chains: OscillatorChainsState,
    pub tempo: f32,
    #[serde(default)]
    pub envelopes: EnvelopesState,
    pub effects: EffectsRackState,
    #[serde(default)]
    pub equalizers: EqualizersState,
    #[serde(default)]
    pub effect_chains: EffectChainsState,
    pub tracks: Vec<TrackStrip>,
}

impl SessionState {
    fn session_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("rosco").join("gui_session.json"))
    }

    pub fn save(&self) {
        if let Some(path) = Self::session_path() {
            if let Some(dir) = path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            if let Ok(json) = serde_json::to_string_pretty(self) {
                let _ = std::fs::write(path, json);
            }
        }
    }

    pub fn load() -> Option<Self> {
        Self::session_path()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .and_then(|contents| serde_json::from_str(&contents).ok())
    }
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            oscillator_chains: OscillatorChainsState::default(),
            tempo: 120.0,
            envelopes: EnvelopesState::default(),
            effects: EffectsRackState::default(),
            equalizers: EqualizersState::default(),
            effect_chains: EffectChainsState::default(),
            tracks: (0..NUM_TRACKS).map(|_| TrackStrip::default()).collect(),
        }
    }
}

// --- Effect presets ---

#[derive(Serialize, Deserialize)]
pub struct EffectPreset {
    pub name: String,
    pub effects: EffectsRackState,
}

impl EffectPreset {
    fn presets_dir() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("rosco").join("presets"))
    }

    pub fn save(&self) -> Result<(), String> {
        let dir = Self::presets_dir().ok_or("Could not determine presets directory")?;
        std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create dir: {}", e))?;
        let path = dir.join(format!("{}.toml", self.name));
        let contents = toml::to_string_pretty(self).map_err(|e| format!("Serialize error: {}", e))?;
        std::fs::write(path, contents).map_err(|e| format!("Write error: {}", e))
    }

    pub fn load(name: &str) -> Result<Self, String> {
        let dir = Self::presets_dir().ok_or("Could not determine presets directory")?;
        let path = dir.join(format!("{}.toml", name));
        let contents = std::fs::read_to_string(path).map_err(|e| format!("Read error: {}", e))?;
        toml::from_str(&contents).map_err(|e| format!("Parse error: {}", e))
    }

    pub fn list_presets() -> Vec<String> {
        Self::presets_dir()
            .and_then(|dir| std::fs::read_dir(dir).ok())
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter_map(|e| {
                        let path = e.path();
                        if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                            path.file_stem()
                                .and_then(|s| s.to_str())
                                .map(String::from)
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}
