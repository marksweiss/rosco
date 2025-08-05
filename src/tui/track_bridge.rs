use crate::tui::ui::widgets::{TrackStrip, StepCell};

/// Bridge between TUI sequencer tracks and Rosco Track system
/// Simplified version for Week 3 implementation
#[derive(Debug)]
pub struct TrackBridge {
    track_data: Vec<TrackData>,
    tempo: u8,
}

#[derive(Debug, Clone)]
pub struct TrackData {
    pub track_number: u8,
    pub volume: f32,
    pub pan: f32,  // Single pan control (-1.0 to +1.0)
    pub mute: bool,
    pub solo: bool,
    pub steps: Vec<StepCell>,
}

#[derive(Debug)]
pub enum TrackUpdate {
    Volume(f32),
    Pan(f32),
    Mute(bool),
    Solo(bool),
    StepToggled { step: usize, enabled: bool },
    SequenceChanged,
}

impl TrackBridge {
    pub fn new(num_tracks: usize, steps_per_track: usize, tempo: u8) -> Self {
        let mut track_data = Vec::new();
        
        for i in 0..num_tracks {
            let track = TrackData {
                track_number: i as u8 + 1,
                volume: 0.8,
                pan: 0.0,
                mute: false,
                solo: false,
                steps: vec![StepCell::default(); steps_per_track],
            };
            track_data.push(track);
        }
        
        Self { track_data, tempo }
    }
    
    /// Convert TUI TrackStrip to internal track data
    pub fn sync_from_tui(&mut self, track_strips: &[TrackStrip; 8]) {
        for (i, strip) in track_strips.iter().enumerate() {
            if i < self.track_data.len() {
                self.track_data[i].volume = strip.volume;
                self.track_data[i].pan = strip.pan;
                self.track_data[i].mute = strip.mute;
                self.track_data[i].solo = strip.solo;
                self.track_data[i].steps = strip.steps.clone();
            }
        }
    }
    
    /// Convert internal track data to TUI TrackStrip
    pub fn sync_to_tui(&self, track_strips: &mut [TrackStrip; 8]) {
        for (i, track_data) in self.track_data.iter().enumerate() {
            if i < track_strips.len() {
                track_strips[i].volume = track_data.volume;
                track_strips[i].pan = track_data.pan;
                track_strips[i].mute = track_data.mute;
                track_strips[i].solo = track_data.solo;
                track_strips[i].steps = track_data.steps.clone();
            }
        }
    }
    
    /// Get all track data for processing
    pub fn get_track_data(&self) -> &[TrackData] {
        &self.track_data
    }
    
    /// Get mutable reference to track data
    pub fn get_track_data_mut(&mut self) -> &mut [TrackData] {
        &mut self.track_data
    }
    
    /// Update track parameter
    pub fn update_track(&mut self, track_idx: usize, update: TrackUpdate) {
        if let Some(track_data) = self.track_data.get_mut(track_idx) {
            match update {
                TrackUpdate::Volume(volume) => {
                    track_data.volume = volume;
                }
                TrackUpdate::Pan(pan) => {
                    track_data.pan = pan;
                }
                TrackUpdate::Mute(mute) => {
                    track_data.mute = mute;
                }
                TrackUpdate::Solo(solo) => {
                    track_data.solo = solo;
                }
                TrackUpdate::StepToggled { step, enabled } => {
                    if step < track_data.steps.len() {
                        track_data.steps[step].enabled = enabled;
                    }
                }
                TrackUpdate::SequenceChanged => {
                    // Handle sequence-level changes
                }
            }
        }
    }
    
    /// Set tempo for all tracks
    pub fn set_tempo(&mut self, tempo: u8) {
        self.tempo = tempo;
    }
    
    /// Get current tempo
    pub fn get_tempo(&self) -> u8 {
        self.tempo
    }
}

/// Helper functions for common operations
impl TrackBridge {
    /// Initialize tracks with default patterns
    pub fn init_with_default_patterns(&mut self) {
        // Add some default patterns for demonstration
        for (track_idx, track_data) in self.track_data.iter_mut().enumerate() {
            match track_idx {
                0 => {
                    // Kick pattern: steps 1, 5, 9, 13
                    for &step in &[0, 4, 8, 12] {
                        if step < track_data.steps.len() {
                            track_data.steps[step].enabled = true;
                            track_data.steps[step].velocity = 127;
                        }
                    }
                }
                1 => {
                    // Snare pattern: steps 5, 13
                    for &step in &[4, 12] {
                        if step < track_data.steps.len() {
                            track_data.steps[step].enabled = true;
                            track_data.steps[step].velocity = 120;
                        }
                    }
                }
                2 => {
                    // Hi-hat pattern: every other step
                    for step in (1..16).step_by(2) {
                        if step < track_data.steps.len() {
                            track_data.steps[step].enabled = true;
                            track_data.steps[step].velocity = 80;
                        }
                    }
                }
                _ => {
                    // Other tracks start empty
                }
            }
        }
    }
}