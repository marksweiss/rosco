use crate::tui::ui::widgets::{SequencerGrid, StepCell};
use crate::tui::pattern_manager::PatternManager;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug)]
pub struct SequencerPanel {
    pub grid: SequencerGrid,
    clipboard: Option<Vec<StepCell>>,
    pattern_manager: PatternManager,
    show_pattern_browser: bool,
}

#[derive(Debug, Clone)]
pub enum SequencerAction {
    StepToggled { track: u8, step: u8 },
    TrackVolumeChanged { track: u8, volume: f32 },
    TrackPanChanged { track: u8, pan: f32 },
    TrackMuteToggled { track: u8 },
    TrackSoloToggled { track: u8 },
    TrackCleared { track: u8 },
    PatternCopied,
    PatternPasted,
    PatternStored { pattern_id: String },
    PatternLoaded { pattern_id: String },
    PatternBrowserToggled,
    SelectionStarted,
    SelectionCleared,
}

impl SequencerPanel {
    pub fn new() -> Self {
        let mut pattern_manager = PatternManager::new();
        pattern_manager.init_with_defaults();
        
        Self {
            grid: SequencerGrid::new(16), // 16 steps per track
            clipboard: None,
            pattern_manager,
            show_pattern_browser: false,
        }
    }
    
    pub fn handle_key_event(&mut self, key: KeyEvent) -> Vec<SequencerAction> {
        let mut actions = Vec::new();
        
        match key.code {
            // Navigation
            KeyCode::Up => {
                self.grid.move_cursor(-1, 0);
            }
            KeyCode::Down => {
                self.grid.move_cursor(1, 0);
            }
            KeyCode::Left => {
                self.grid.move_cursor(0, -1);
            }
            KeyCode::Right => {
                self.grid.move_cursor(0, 1);
            }
            
            // Focus switching
            KeyCode::Tab => {
                self.grid.switch_focus();
            }
            
            // Step editing
            KeyCode::Enter | KeyCode::Char(' ') => {
                match self.grid.cursor.focus_area {
                    crate::tui::ui::widgets::CursorFocus::Steps => {
                        self.grid.toggle_current_step();
                        actions.push(SequencerAction::StepToggled {
                            track: self.grid.cursor.track,
                            step: self.grid.cursor.step,
                        });
                    }
                    crate::tui::ui::widgets::CursorFocus::TrackControls => {
                        self.handle_track_control_action(&mut actions);
                    }
                }
            }
            
            // Parameter adjustment
            KeyCode::Char('+') | KeyCode::Char('=') => {
                if self.grid.cursor.focus_area == crate::tui::ui::widgets::CursorFocus::TrackControls {
                    let delta = if key.modifiers.contains(KeyModifiers::SHIFT) { 0.01 } else { 0.1 };
                    self.adjust_track_parameter(delta, &mut actions);
                }
            }
            KeyCode::Char('-') => {
                if self.grid.cursor.focus_area == crate::tui::ui::widgets::CursorFocus::TrackControls {
                    let delta = if key.modifiers.contains(KeyModifiers::SHIFT) { -0.01 } else { -0.1 };
                    self.adjust_track_parameter(delta, &mut actions);
                }
            }
            
            // Quick track selection (A-H for tracks 1-8)
            KeyCode::Char(c) if c >= 'a' && c <= 'h' => {
                let track_idx = (c as u8 - b'a').min(7);
                self.grid.cursor.track = track_idx;
            }
            KeyCode::Char(c) if c >= 'A' && c <= 'H' => {
                let track_idx = (c as u8 - b'A').min(7);
                self.grid.cursor.track = track_idx;
            }
            
            // Quick step selection (1-9, 0 for step 10)
            KeyCode::Char(c) if c >= '1' && c <= '9' => {
                let step_idx = (c as u8 - b'1') as u8;
                if step_idx < self.grid.steps_per_track as u8 {
                    self.grid.cursor.step = step_idx;
                }
            }
            KeyCode::Char('0') => {
                if self.grid.steps_per_track > 9 {
                    self.grid.cursor.step = 9; // Step 10 (0-indexed)
                }
            }
            
            // Pattern operations
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Some(pattern) = self.grid.copy_pattern() {
                    self.clipboard = Some(pattern);
                    actions.push(SequencerAction::PatternCopied);
                }
            }
            KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Some(pattern) = &self.clipboard {
                    self.grid.paste_pattern(pattern);
                    actions.push(SequencerAction::PatternPasted);
                }
            }
            KeyCode::Char('x') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Some(pattern) = self.grid.copy_pattern() {
                    self.clipboard = Some(pattern);
                    self.grid.clear_current_track();
                    actions.push(SequencerAction::TrackCleared {
                        track: self.grid.cursor.track,
                    });
                }
            }
            KeyCode::Char('C') => {
                self.grid.clear_current_track();
                actions.push(SequencerAction::TrackCleared {
                    track: self.grid.cursor.track,
                });
            }
            
            // Selection
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if self.grid.selection.is_none() {
                    self.grid.start_selection();
                    actions.push(SequencerAction::SelectionStarted);
                } else {
                    self.grid.clear_selection();
                    actions.push(SequencerAction::SelectionCleared);
                }
            }
            
            // Pattern management
            KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.show_pattern_browser = !self.show_pattern_browser;
                actions.push(SequencerAction::PatternBrowserToggled);
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::ALT) => {
                // Store current track as pattern
                let track = &self.grid.tracks[self.grid.cursor.track as usize];
                let pattern_name = format!("Track {} Pattern", track.track_number);
                let pattern_id = self.pattern_manager.store_pattern(
                    pattern_name,
                    track.steps.clone(),
                    Some(format!("Pattern from track {}", track.track_number)),
                );
                actions.push(SequencerAction::PatternStored { pattern_id });
            }
            KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::ALT) => {
                // Load last pattern to current track
                if let Some(pattern) = self.pattern_manager.get_recent_patterns(1).first() {
                    let track = &mut self.grid.tracks[self.grid.cursor.track as usize];
                    track.steps = pattern.steps.clone();
                    actions.push(SequencerAction::PatternLoaded { 
                        pattern_id: pattern.id.clone() 
                    });
                }
            }
            
            KeyCode::Delete => {
                if self.grid.selection.is_some() {
                    self.grid.delete_selected();
                    actions.push(SequencerAction::SelectionCleared);
                } else if self.grid.cursor.focus_area == crate::tui::ui::widgets::CursorFocus::Steps {
                    let track = &mut self.grid.tracks[self.grid.cursor.track as usize];
                    let step = &mut track.steps[self.grid.cursor.step as usize];
                    step.enabled = false;
                    step.note = None;
                    actions.push(SequencerAction::StepToggled {
                        track: self.grid.cursor.track,
                        step: self.grid.cursor.step,
                    });
                }
            }
            
            // Advanced selection operations
            KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.grid.select_all_track();
                actions.push(SequencerAction::SelectionStarted);
            }
            KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::ALT) => {
                self.grid.select_all_step();
                actions.push(SequencerAction::SelectionStarted);
            }
            KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Fill selection with enabled steps
                if self.grid.selection.is_some() {
                    self.grid.fill_selected(true);
                }
            }
            KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Empty selection (disable all steps in selection)
                if self.grid.selection.is_some() {
                    self.grid.fill_selected(false);
                }
            }
            
            _ => {}
        }
        
        // Update selection if active
        if self.grid.selection.is_some() {
            self.grid.update_selection();
        }
        
        actions
    }
    
    fn handle_track_control_action(&mut self, actions: &mut Vec<SequencerAction>) {
        let track = &mut self.grid.tracks[self.grid.cursor.track as usize];
        
        match track.selected_control {
            crate::tui::ui::widgets::TrackControl::Mute => {
                track.toggle_mute();
                actions.push(SequencerAction::TrackMuteToggled {
                    track: self.grid.cursor.track,
                });
            }
            crate::tui::ui::widgets::TrackControl::Solo => {
                track.toggle_solo();
                actions.push(SequencerAction::TrackSoloToggled {
                    track: self.grid.cursor.track,
                });
            }
            _ => {}
        }
    }
    
    fn adjust_track_parameter(&mut self, delta: f32, actions: &mut Vec<SequencerAction>) {
        let track_idx = self.grid.cursor.track;
        let track = &mut self.grid.tracks[track_idx as usize];
        
        match track.selected_control {
            crate::tui::ui::widgets::TrackControl::Volume => {
                track.adjust_volume(delta);
                actions.push(SequencerAction::TrackVolumeChanged {
                    track: track_idx,
                    volume: track.volume,
                });
            }
            crate::tui::ui::widgets::TrackControl::Pan => {
                track.adjust_pan(delta);
                actions.push(SequencerAction::TrackPanChanged {
                    track: track_idx,
                    pan: track.pan,
                });
            }
            _ => {}
        }
    }
    
    pub fn set_focused(&mut self, focused: bool) {
        self.grid.focused = focused;
    }
    
    pub fn set_playing_step(&mut self, step: Option<usize>) {
        self.grid.set_playing_step(step);
    }
    
    pub fn get_pattern_manager(&self) -> &PatternManager {
        &self.pattern_manager
    }
    
    pub fn get_pattern_manager_mut(&mut self) -> &mut PatternManager {
        &mut self.pattern_manager
    }
    
    pub fn is_pattern_browser_visible(&self) -> bool {
        self.show_pattern_browser
    }
    
    pub fn load_pattern_to_track(&mut self, pattern_id: &str, track_idx: usize) -> bool {
        if let Some(pattern_steps) = self.pattern_manager.get_pattern_steps(pattern_id) {
            if track_idx < self.grid.tracks.len() {
                self.grid.tracks[track_idx].steps = pattern_steps;
                return true;
            }
        }
        false
    }
    
    pub fn store_track_as_pattern(&mut self, track_idx: usize, name: String) -> Option<String> {
        if track_idx < self.grid.tracks.len() {
            let track = &self.grid.tracks[track_idx];
            let pattern_id = self.pattern_manager.store_pattern(
                name,
                track.steps.clone(),
                Some(format!("Pattern from track {}", track.track_number)),
            );
            Some(pattern_id)
        } else {
            None
        }
    }
}