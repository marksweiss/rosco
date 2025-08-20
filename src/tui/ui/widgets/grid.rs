use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

use crate::note::playback_note::PlaybackNote;
use crate::note::scales::WesternPitch;

#[derive(Debug, Clone)]
pub struct SequencerGrid {
    pub tracks: [TrackStrip; 8],
    pub steps_per_track: usize,
    pub cursor: GridCursor,
    pub playing_step: Option<usize>,
    pub selection: Option<GridSelection>,
    pub focused: bool,
}

#[derive(Debug, Clone)]
pub struct TrackStrip {
    pub track_number: u8,
    pub volume: f32,
    pub pan: f32,  // Single pan control (-1.0 to +1.0)
    pub mute: bool,
    pub solo: bool,
    pub steps: Vec<StepCell>,
    pub selected_control: TrackControl,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TrackControl {
    Volume,
    Pan,
    Mute,
    Solo,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StepCell {
    pub enabled: bool,
    pub velocity: u8,
    pub frequency: WesternPitch,
    #[serde(skip)] // Skip serialization of PlaybackNote for now
    pub note: Option<PlaybackNote>,
    #[serde(skip)] // Skip serialization of highlighted state
    pub highlighted: bool,
}

#[derive(Debug, Clone)]
pub struct GridCursor {
    pub track: u8,
    pub step: u8,
    pub focus_area: CursorFocus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CursorFocus {
    Steps,
    Frequency,
    FrequencyDropdown, // New state for when dropdown is open
    TrackControls,
}

#[derive(Debug, Clone)]
pub struct GridSelection {
    pub start: GridCursor,
    pub end: GridCursor,
}

impl SequencerGrid {
    pub fn new(steps_per_track: usize) -> Self {
        let tracks = std::array::from_fn(|i| TrackStrip::new(i as u8 + 1, steps_per_track));
        
        Self {
            tracks,
            steps_per_track,
            cursor: GridCursor { 
                track: 0, 
                step: 0, 
                focus_area: CursorFocus::Steps 
            },
            playing_step: None,
            selection: None,
            focused: false,
        }
    }
    
    pub fn move_cursor(&mut self, track_delta: i8, step_delta: i8) {
        match self.cursor.focus_area {
            CursorFocus::Steps => {
                if track_delta != 0 {
                    if track_delta > 0 {
                        // Down arrow: move to frequency row of same track
                        self.cursor.focus_area = CursorFocus::Frequency;
                    } else {
                        // Up arrow: move to previous track's frequency row
                        if self.cursor.track > 0 {
                            self.cursor.track -= 1;
                            self.cursor.focus_area = CursorFocus::Frequency;
                        }
                    }
                } else if step_delta != 0 {
                    // Left/Right in Steps mode: navigate steps
                    let new_step = (self.cursor.step as i8 + step_delta)
                        .clamp(0, self.steps_per_track as i8 - 1) as u8;
                    self.cursor.step = new_step;
                }
            }
            CursorFocus::Frequency => {
                if track_delta != 0 {
                    if track_delta < 0 {
                        // Up arrow: move to steps row of same track
                        self.cursor.focus_area = CursorFocus::Steps;
                    } else {
                        // Down arrow: move to next track's steps row
                        if self.cursor.track < 7 {
                            self.cursor.track += 1;
                            self.cursor.focus_area = CursorFocus::Steps;
                        }
                    }
                } else if step_delta != 0 {
                    // Left/Right in Frequency mode: navigate steps
                    let new_step = (self.cursor.step as i8 + step_delta)
                        .clamp(0, self.steps_per_track as i8 - 1) as u8;
                    self.cursor.step = new_step;
                }
            }
            CursorFocus::FrequencyDropdown => {
                // In dropdown mode, only Up/Down changes frequency values
                // Note: frequency changes need to be handled in sequencer for proper action dispatch
                // This just prevents navigation, actual frequency changes happen in sequencer
                // Left/Right do nothing in dropdown mode
            }
            CursorFocus::TrackControls => {
                if track_delta != 0 {
                    let new_track = (self.cursor.track as i8 + track_delta)
                        .clamp(0, 7) as u8;
                    self.cursor.track = new_track;
                }
                
                if step_delta != 0 {
                    let track = &mut self.tracks[self.cursor.track as usize];
                    let controls = [
                        TrackControl::Volume,
                        TrackControl::Pan,
                        TrackControl::Mute,
                        TrackControl::Solo,
                    ];
                    
                    let current_idx = controls.iter()
                        .position(|c| *c == track.selected_control)
                        .unwrap_or(0);
                    
                    let new_idx = ((current_idx as i8 + step_delta)
                        .clamp(0, controls.len() as i8 - 1)) as usize;
                    
                    track.selected_control = controls[new_idx].clone();
                }
            }
        }
    }
    
    pub fn switch_focus(&mut self) {
        self.cursor.focus_area = match self.cursor.focus_area {
            CursorFocus::Steps => CursorFocus::TrackControls,
            CursorFocus::Frequency => CursorFocus::TrackControls, // Shouldn't happen via Tab
            CursorFocus::FrequencyDropdown => CursorFocus::TrackControls, // Exit dropdown
            CursorFocus::TrackControls => CursorFocus::Steps,
        };
    }
    
    pub fn toggle_current_step(&mut self) {
        let track = &mut self.tracks[self.cursor.track as usize];
        let step = &mut track.steps[self.cursor.step as usize];
        step.enabled = !step.enabled;
    }

    pub fn adjust_current_frequency(&mut self, direction: i8) {
        let track = &mut self.tracks[self.cursor.track as usize];
        let step = &mut track.steps[self.cursor.step as usize];
        
        step.frequency = if direction > 0 {
            step.frequency.next()
        } else {
            step.frequency.previous()
        };
    }

    pub fn get_current_frequency(&self) -> WesternPitch {
        self.tracks[self.cursor.track as usize].steps[self.cursor.step as usize].frequency
    }

    pub fn enter_frequency_dropdown(&mut self) {
        if self.cursor.focus_area == CursorFocus::Frequency {
            self.cursor.focus_area = CursorFocus::FrequencyDropdown;
        }
    }

    pub fn exit_frequency_dropdown(&mut self) {
        if self.cursor.focus_area == CursorFocus::FrequencyDropdown {
            self.cursor.focus_area = CursorFocus::Frequency;
        }
    }
    
    pub fn set_playing_step(&mut self, step: Option<usize>) {
        self.playing_step = step;
    }
    
    pub fn adjust_current_track_control(&mut self, delta: f32) {
        let track = &mut self.tracks[self.cursor.track as usize];
        match track.selected_control {
            TrackControl::Volume => track.adjust_volume(delta),
            TrackControl::Pan => track.adjust_pan(delta),
            TrackControl::Mute => track.toggle_mute(),
            TrackControl::Solo => track.toggle_solo(),
        }
    }
    
    pub fn clear_track(&mut self, track_idx: usize) {
        if track_idx < self.tracks.len() {
            for step in &mut self.tracks[track_idx].steps {
                step.enabled = false;
                step.note = None;
            }
        }
    }
    
    pub fn clear_current_track(&mut self) {
        self.clear_track(self.cursor.track as usize);
    }
    
    pub fn copy_pattern(&self) -> Option<Vec<StepCell>> {
        if let Some(selection) = &self.selection {
            let start_step = selection.start.step.min(selection.end.step) as usize;
            let end_step = selection.start.step.max(selection.end.step) as usize;
            let start_track = selection.start.track.min(selection.end.track) as usize;
            let end_track = selection.start.track.max(selection.end.track) as usize;
            
            // For single track selection, return the steps
            if start_track == end_track && start_track < self.tracks.len() {
                return Some(
                    self.tracks[start_track].steps[start_step..=end_step].to_vec()
                );
            }
            
            // For multi-track selection, flatten the selection
            // This could be extended to support more complex multi-track patterns
            let mut pattern = Vec::new();
            for track_idx in start_track..=end_track {
                if track_idx < self.tracks.len() {
                    pattern.extend_from_slice(
                        &self.tracks[track_idx].steps[start_step..=end_step]
                    );
                }
            }
            
            if !pattern.is_empty() {
                return Some(pattern);
            }
        }
        None
    }
    
    pub fn paste_pattern(&mut self, pattern: &[StepCell]) {
        let start_step = self.cursor.step as usize;
        let track_idx = self.cursor.track as usize;
        
        if track_idx < self.tracks.len() {
            let track = &mut self.tracks[track_idx];
            for (i, step_data) in pattern.iter().enumerate() {
                if start_step + i < track.steps.len() {
                    track.steps[start_step + i] = step_data.clone();
                }
            }
        }
    }
    
    pub fn start_selection(&mut self) {
        self.selection = Some(GridSelection {
            start: self.cursor.clone(),
            end: self.cursor.clone(),
        });
    }
    
    pub fn update_selection(&mut self) {
        if let Some(selection) = &mut self.selection {
            selection.end = self.cursor.clone();
        }
    }
    
    pub fn clear_selection(&mut self) {
        self.selection = None;
    }
    
    pub fn select_all_track(&mut self) {
        self.selection = Some(GridSelection {
            start: GridCursor {
                track: self.cursor.track,
                step: 0,
                focus_area: self.cursor.focus_area.clone(),
            },
            end: GridCursor {
                track: self.cursor.track,
                step: (self.steps_per_track - 1) as u8,
                focus_area: self.cursor.focus_area.clone(),
            },
        });
    }
    
    pub fn select_all_step(&mut self) {
        self.selection = Some(GridSelection {
            start: GridCursor {
                track: 0,
                step: self.cursor.step,
                focus_area: self.cursor.focus_area.clone(),
            },
            end: GridCursor {
                track: 7,
                step: self.cursor.step,
                focus_area: self.cursor.focus_area.clone(),
            },
        });
    }
    
    pub fn delete_selected(&mut self) {
        if let Some(selection) = &self.selection {
            let start_step = selection.start.step.min(selection.end.step) as usize;
            let end_step = selection.start.step.max(selection.end.step) as usize;
            let start_track = selection.start.track.min(selection.end.track) as usize;
            let end_track = selection.start.track.max(selection.end.track) as usize;
            
            for track_idx in start_track..=end_track {
                if track_idx < self.tracks.len() {
                    for step_idx in start_step..=end_step {
                        if step_idx < self.tracks[track_idx].steps.len() {
                            self.tracks[track_idx].steps[step_idx].enabled = false;
                            self.tracks[track_idx].steps[step_idx].note = None;
                        }
                    }
                }
            }
        }
    }
    
    pub fn fill_selected(&mut self, enabled: bool) {
        if let Some(selection) = &self.selection {
            let start_step = selection.start.step.min(selection.end.step) as usize;
            let end_step = selection.start.step.max(selection.end.step) as usize;
            let start_track = selection.start.track.min(selection.end.track) as usize;
            let end_track = selection.start.track.max(selection.end.track) as usize;
            
            for track_idx in start_track..=end_track {
                if track_idx < self.tracks.len() {
                    for step_idx in start_step..=end_step {
                        if step_idx < self.tracks[track_idx].steps.len() {
                            self.tracks[track_idx].steps[step_idx].enabled = enabled;
                        }
                    }
                }
            }
        }
    }
    
    pub fn get_selection_bounds(&self) -> Option<(usize, usize, usize, usize)> {
        if let Some(selection) = &self.selection {
            let start_step = selection.start.step.min(selection.end.step) as usize;
            let end_step = selection.start.step.max(selection.end.step) as usize;
            let start_track = selection.start.track.min(selection.end.track) as usize;
            let end_track = selection.start.track.max(selection.end.track) as usize;
            
            Some((start_track, end_track, start_step, end_step))
        } else {
            None
        }
    }
}

impl TrackStrip {
    pub fn new(track_number: u8, steps: usize) -> Self {
        let track_steps = vec![StepCell::default(); steps];
        
        Self {
            track_number,
            volume: 0.8,
            pan: 0.0,
            mute: false,
            solo: false,
            steps: track_steps,
            selected_control: TrackControl::Volume,
        }
    }
    
    pub fn adjust_volume(&mut self, delta: f32) {
        self.volume = (self.volume + delta).clamp(0.0, 1.0);
    }
    
    pub fn adjust_pan(&mut self, delta: f32) {
        self.pan = (self.pan + delta).clamp(-1.0, 1.0);
    }
    
    pub fn toggle_mute(&mut self) {
        self.mute = !self.mute;
    }
    
    pub fn toggle_solo(&mut self) {
        self.solo = !self.solo;
    }
}

impl Default for StepCell {
    fn default() -> Self {
        Self {
            enabled: false,
            velocity: 127,
            frequency: WesternPitch::C,
            note: None,
            highlighted: false,
        }
    }
}

impl Widget for SequencerGrid {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = if self.focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::White)
        };
        
        // Calculate layout areas - move controls to far right, expand grid
        let control_width = 80; // Increased back to original size for proper layout
        let step_area_width = area.width.saturating_sub(control_width);
        
        // Render track rows (each track takes 2 rows: steps + frequency)
        // Allow for all 8 tracks - be less restrictive with height calculation
        for (track_idx, track) in self.tracks.iter().enumerate() {
            let y_steps = area.y + (track_idx * 2) as u16;
            let y_freq = y_steps + 1;
            
            // Stop if we don't have room for both rows of this track
            // Leave minimal space for step numbers at bottom
            if y_freq >= area.y + area.height.saturating_sub(1) {
                break;
            }
            
            let x = area.x;
            
            // Track number (spans both rows)
            let track_style = if self.cursor.track == track_idx as u8 {
                Style::default().fg(Color::Yellow)
            } else {
                style
            };
            buf.set_string(x, y_steps, &format!("{}", track.track_number), track_style);
            let mut step_x = x + 2;
            
            // Step cells - show as many steps as will fit, up to 16
            let max_steps = ((step_area_width.saturating_sub(2)) / 4) as usize; // 4 chars per step
            let visible_steps = self.steps_per_track.min(max_steps);
            
            for step_idx in 0..visible_steps {
                if step_idx >= track.steps.len() {
                    break;
                }
                
                let step = &track.steps[step_idx];
                let is_step_cursor = self.cursor.track == track_idx as u8 && 
                                   self.cursor.step == step_idx as u8 &&
                                   self.cursor.focus_area == CursorFocus::Steps;
                let is_freq_cursor = self.cursor.track == track_idx as u8 && 
                                   self.cursor.step == step_idx as u8 &&
                                   self.cursor.focus_area == CursorFocus::Frequency;
                let is_freq_dropdown = self.cursor.track == track_idx as u8 && 
                                      self.cursor.step == step_idx as u8 &&
                                      self.cursor.focus_area == CursorFocus::FrequencyDropdown;
                let is_playing = self.playing_step == Some(step_idx);
                let is_selected = self.is_step_selected(track_idx as u8, step_idx as u8);
                
                // Step cell style
                let step_style = if is_step_cursor {
                    Style::default().fg(Color::Yellow).bg(Color::DarkGray)
                } else if is_playing {
                    Style::default().fg(Color::Green).bg(Color::Black)
                } else if is_selected {
                    Style::default().fg(Color::White).bg(Color::Blue)
                } else {
                    style
                };
                
                // Frequency cell style  
                let freq_style = if is_freq_dropdown {
                    Style::default().fg(Color::Rgb(255, 255, 0)).bg(Color::Rgb(0, 0, 255)) // Bright yellow on blue for dropdown
                } else if is_freq_cursor {
                    Style::default().fg(Color::Rgb(0, 255, 0)).bg(Color::Black) // Pure bright green on black for maximum contrast
                } else if is_playing {
                    Style::default().fg(Color::Green).bg(Color::Black)
                } else if is_selected {
                    Style::default().fg(Color::LightGreen).bg(Color::Blue) // Bright light green text for selected frequency cells
                } else {
                    // Use bright green text for better visibility instead of default style
                    Style::default().fg(Color::LightGreen)
                };
                
                // Render step cell
                let symbol = if step.enabled { "●" } else { "·" };
                buf.set_string(step_x, y_steps, &format!(" {} ", symbol), step_style);
                
                // Render frequency cell - match the step cell format for alignment
                let freq_text = if step.enabled {
                    if is_freq_dropdown {
                        // Show active dropdown with special indicators
                        format!("▼{}▲", step.frequency)
                    } else if is_freq_cursor {
                        // Show selectable frequency with brackets
                        format!("[{}]", step.frequency)
                    } else {
                        format!(" {} ", step.frequency)
                    }
                } else {
                    " · ".to_string()
                };
                buf.set_string(step_x, y_freq, &freq_text, freq_style);
                
                step_x += 4;
            }
            
            // Track controls (positioned to the right, spans both step and frequency rows)
            let controls_x = area.x + step_area_width;
            if controls_x < area.x + area.width {
                self.render_track_controls(track, track_idx as u8, controls_x, y_steps, buf, style);
            }
        }
        
        // Render step numbers at bottom
        // Calculate how many tracks we actually rendered using the same logic as the track loop
        let mut rendered_tracks = 0;
        for track_idx in 0..self.tracks.len() {
            let y_steps = area.y + (track_idx * 2) as u16;
            let y_freq = y_steps + 1;
            
            if y_freq >= area.y + area.height.saturating_sub(1) {
                break;
            }
            rendered_tracks += 1;
        }
        
        // Only render step numbers if there's space
        let step_numbers_y = area.y + (rendered_tracks * 2) as u16;
        if step_numbers_y < area.y + area.height {
            let mut x = area.x + 2; // Offset for track numbers
            
            let max_steps = ((step_area_width.saturating_sub(2)) / 4) as usize;
            let visible_steps = self.steps_per_track.min(max_steps);
            
            for step in 1..=visible_steps {
                buf.set_string(x, step_numbers_y, &format!("{:^4}", step), style);
                x += 4;
            }
        }
    }
}

impl SequencerGrid {
    fn is_step_selected(&self, track: u8, step: u8) -> bool {
        if let Some(selection) = &self.selection {
            let min_track = selection.start.track.min(selection.end.track);
            let max_track = selection.start.track.max(selection.end.track);
            let min_step = selection.start.step.min(selection.end.step);
            let max_step = selection.start.step.max(selection.end.step);
            
            track >= min_track && track <= max_track &&
            step >= min_step && step <= max_step
        } else {
            false
        }
    }
    
    fn render_track_controls(&self, track: &TrackStrip, track_idx: u8, x: u16, y: u16, buf: &mut Buffer, base_style: Style) {
        let is_track_focused = self.cursor.track == track_idx && 
                              self.cursor.focus_area == CursorFocus::TrackControls;
        
        // Volume control - match original format: "V1 [████████░░] 80%"
        let vol_style = if is_track_focused && track.selected_control == TrackControl::Volume {
            Style::default().fg(Color::Yellow).bg(Color::DarkGray)
        } else {
            base_style
        };
        
        let vol_percent = (track.volume * 100.0) as u8;
        let vol_bars = (track.volume * 15.0) as usize; // 15 blocks like original
        let vol_filled = "█".repeat(vol_bars);
        let vol_empty = "░".repeat(15 - vol_bars);
        let vol_display = format!("V{} {}{} {}%", track_idx + 1, vol_filled, vol_empty, vol_percent);
        buf.set_string(x, y, &vol_display, vol_style);
        
        // Pan control - match original format: "L [░░░█░░░] R +0%"
        let pan_style = if is_track_focused && track.selected_control == TrackControl::Pan {
            Style::default().fg(Color::Yellow).bg(Color::DarkGray)
        } else {
            base_style
        };
        
        let pan_percent = (track.pan * 100.0) as i8;
        let pan_pos = ((track.pan + 1.0) * 7.5) as usize; // 15 positions (0-14) like original
        let mut pan_display: Vec<char> = "░".repeat(15).chars().collect();
        
        // Mark center position with a different character
        pan_display[7] = '│'; // Center marker (position 7 out of 15)
        if pan_pos < 15 {
            pan_display[pan_pos] = '█'; // Current position
        }
        
        let pan_display: String = pan_display.into_iter().collect();
        let pan_text = format!("L {} R {:+}%", pan_display, pan_percent);
        buf.set_string(x + 25, y, &pan_text, pan_style);
    }
}