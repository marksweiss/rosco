use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

use crate::note::playback_note::PlaybackNote;

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
                let new_track = (self.cursor.track as i8 + track_delta)
                    .clamp(0, 7) as u8;
                let new_step = (self.cursor.step as i8 + step_delta)
                    .clamp(0, self.steps_per_track as i8 - 1) as u8;
                
                self.cursor.track = new_track;
                self.cursor.step = new_step;
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
            CursorFocus::TrackControls => CursorFocus::Steps,
        };
    }
    
    pub fn toggle_current_step(&mut self) {
        let track = &mut self.tracks[self.cursor.track as usize];
        let step = &mut track.steps[self.cursor.step as usize];
        step.enabled = !step.enabled;
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
        Self {
            track_number,
            volume: 0.8,
            pan: 0.0,
            mute: false,
            solo: false,
            steps: vec![StepCell::default(); steps],
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
        
        // Calculate layout areas
        let control_width = 80; // Increased to accommodate 3x wider display
        let step_area_width = area.width.saturating_sub(control_width);
        
        // Render track rows
        for (track_idx, track) in self.tracks.iter().enumerate() {
            if track_idx >= area.height.saturating_sub(2) as usize {
                break;
            }
            
            let y = area.y + track_idx as u16;
            let mut x = area.x;
            
            // Track number
            let track_style = if self.cursor.track == track_idx as u8 {
                Style::default().fg(Color::Yellow)
            } else {
                style
            };
            buf.set_string(x, y, &format!("{} ", track.track_number), track_style);
            x += 2;
            
            // Step cells
            let max_steps = ((step_area_width.saturating_sub(2)) / 3) as usize;
            let visible_steps = self.steps_per_track.min(max_steps);
            
            for step_idx in 0..visible_steps {
                if step_idx >= track.steps.len() {
                    break;
                }
                
                let step = &track.steps[step_idx];
                let is_cursor = self.cursor.track == track_idx as u8 && 
                               self.cursor.step == step_idx as u8 &&
                               self.cursor.focus_area == CursorFocus::Steps;
                let is_playing = self.playing_step == Some(step_idx);
                let is_selected = self.is_step_selected(track_idx as u8, step_idx as u8);
                
                let cell_style = if is_cursor {
                    Style::default().fg(Color::Yellow).bg(Color::DarkGray)
                } else if is_playing {
                    Style::default().fg(Color::Green).bg(Color::Black)
                } else if is_selected {
                    Style::default().fg(Color::White).bg(Color::Blue)
                } else {
                    style
                };
                
                let symbol = if step.enabled { "●" } else { "·" };
                buf.set_string(x, y, &format!("│{}│", symbol), cell_style);
                x += 3;
            }
            
            // Track controls
            x = area.x + step_area_width;
            if x < area.x + area.width {
                self.render_track_controls(track, track_idx as u8, x, y, buf, style);
            }
        }
        
        // Render step numbers at bottom
        if area.height > 8 {
            let y = area.y + 8;
            let mut x = area.x + 2; // Offset for track numbers
            
            let max_steps = ((step_area_width.saturating_sub(2)) / 3) as usize;
            let visible_steps = self.steps_per_track.min(max_steps);
            
            for step in 1..=visible_steps {
                buf.set_string(x, y, &format!(" {} ", step), style);
                x += 3;
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
        
        // Volume control with improved display
        let vol_style = if is_track_focused && track.selected_control == TrackControl::Volume {
            Style::default().fg(Color::Yellow).bg(Color::DarkGray)
        } else {
            base_style
        };
        
        // Volume display with percentage and visual scale
        let vol_percent = (track.volume * 100.0) as u8;
        let vol_bars = (track.volume * 15.0) as usize; // Use 15 blocks (3x wider)
        let vol_filled = "█".repeat(vol_bars);
        let vol_empty = "░".repeat(15 - vol_bars);
        let vol_display = format!("V:{}{} {}%", vol_filled, vol_empty, vol_percent);
        buf.set_string(x, y, &vol_display, vol_style);
        
        // Pan control with L/R labels and slider
        let pan_style = if is_track_focused && track.selected_control == TrackControl::Pan {
            Style::default().fg(Color::Yellow).bg(Color::DarkGray)
        } else {
            base_style
        };
        
        // Pan display with L/R labels and slider
        let pan_percent = (track.pan * 100.0) as i8;
        let pan_pos = ((track.pan + 1.0) * 7.5) as usize; // Use 15 positions (0-14)
        let mut pan_display: Vec<char> = "░".repeat(15).chars().collect();
        
        // Mark center position with a different character
        pan_display[7] = '│'; // Center marker (position 7 out of 15)
        if pan_pos < 15 {
            pan_display[pan_pos] = '█'; // Current position
        }
        
        let pan_display: String = pan_display.into_iter().collect();
        let pan_text = format!("L{}R {:+}%", pan_display, pan_percent);
        buf.set_string(x + 25, y, &pan_text, pan_style);
        
        // Mute/Solo buttons
        let mute_style = if is_track_focused && track.selected_control == TrackControl::Mute {
            Style::default().fg(Color::Yellow).bg(Color::DarkGray)
        } else if track.mute {
            Style::default().fg(Color::Red)
        } else {
            base_style
        };
        
        let solo_style = if is_track_focused && track.selected_control == TrackControl::Solo {
            Style::default().fg(Color::Yellow).bg(Color::DarkGray)
        } else if track.solo {
            Style::default().fg(Color::Green)
        } else {
            base_style
        };
        
        buf.set_string(x, y.saturating_add(1), "│M││S│", base_style);
        buf.set_string(x + 1, y.saturating_add(1), if track.mute { "M" } else { " " }, mute_style);
        buf.set_string(x + 4, y.saturating_add(1), if track.solo { "S" } else { " " }, solo_style);
    }
}