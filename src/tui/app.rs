use crate::tui::{TuiError, audio_bridge::AudioBridge, config::TuiConfig, events::EventHandler};
use crate::tui::ui::{SynthesizerPanel, SequencerPanel};
use crate::tui::ui::synthesizer::SynthesizerSubSection;
use crate::audio_gen;
use crate::filter::filter_manager::FilterManager;

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
    Frame, Terminal, buffer::Buffer,
};
use std::io;
use crate::sequence::FixedTimeNoteSequence;
use crate::track::Track;

// Custom widget to render only the grid part without controls
struct GridOnlyWidget {
    grid: crate::tui::ui::widgets::SequencerGrid,
}

impl Widget for GridOnlyWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = if self.grid.focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::White)
        };
        
        // Render track rows (each track takes 2 rows: steps + frequency)
        for (track_idx, track) in self.grid.tracks.iter().enumerate() {
            let y_steps = area.y + (track_idx * 2) as u16;
            let y_freq = y_steps + 1;
            
            // Stop if we don't have room for both rows of this track
            if y_freq >= area.y + area.height.saturating_sub(1) {
                break;
            }
            
            let x = area.x;
            
            // Track number (spans both rows)
            let track_style = if self.grid.cursor.track == track_idx as u8 {
                Style::default().fg(Color::Yellow)
            } else {
                style
            };
            buf.set_string(x, y_steps, &format!("{}", track.track_number), track_style);
            let mut step_x = x + 2;
            
            // Step cells - show as many steps as will fit, up to 16
            let max_steps = ((area.width.saturating_sub(2)) / 4) as usize; // 4 chars per step
            let visible_steps = self.grid.steps_per_track.min(max_steps);
            
            for step_idx in 0..visible_steps {
                if step_idx >= track.steps.len() {
                    break;
                }
                
                let step = &track.steps[step_idx];
                let is_step_cursor = self.grid.cursor.track == track_idx as u8 && 
                                   self.grid.cursor.step == step_idx as u8 &&
                                   self.grid.cursor.focus_area == crate::tui::ui::widgets::CursorFocus::Steps;
                let is_freq_cursor = self.grid.cursor.track == track_idx as u8 && 
                                   self.grid.cursor.step == step_idx as u8 &&
                                   self.grid.cursor.focus_area == crate::tui::ui::widgets::CursorFocus::Frequency;
                let is_freq_dropdown = self.grid.cursor.track == track_idx as u8 && 
                                      self.grid.cursor.step == step_idx as u8 &&
                                      self.grid.cursor.focus_area == crate::tui::ui::widgets::CursorFocus::FrequencyDropdown;
                let is_playing = self.grid.playing_step == Some(step_idx);
                
                // Step cell style
                let step_style = if is_step_cursor {
                    Style::default().fg(Color::Yellow).bg(Color::DarkGray)
                } else if is_playing {
                    Style::default().fg(Color::Green).bg(Color::Black)
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
                } else {
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
        }
        
        // Render step numbers at bottom
        let mut rendered_tracks = 0;
        for track_idx in 0..self.grid.tracks.len() {
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
            
            let max_steps = ((area.width.saturating_sub(2)) / 4) as usize;
            let visible_steps = self.grid.steps_per_track.min(max_steps);
            
            for step in 1..=visible_steps {
                buf.set_string(x, step_numbers_y, &format!("{:^4}", step), style);
                x += 4;
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FocusArea {
    Synthesizer(SynthSection),
    Sequencer,
    TrackVolume,
    TrackPanning,
    Transport,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SynthSection {
    Oscillator,
    Filter,
    Envelope,
    Effects,
}

#[derive(Debug)]
pub struct UiState {
    pub show_help: bool,
    pub show_pattern_browser: bool,
    pub pattern_browser_selection: usize,
    pub status_message: Option<String>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            show_help: false,
            show_pattern_browser: false,
            pattern_browser_selection: 0,
            status_message: None,
        }
    }
}

pub struct RoscoTuiApp {
    // UI State
    ui_state: UiState,
    current_focus: FocusArea,
    
    // UI Components
    synthesizer_panel: SynthesizerPanel,
    sequencer_panel: SequencerPanel,
    
    // Audio Engine Integration
    audio_bridge: Option<AudioBridge>,
    
    // Synthesizer State
    synth_params: SynthParameters,
    
    // Filter Management
    filter_manager: FilterManager,
    
    // Sequencer State
    #[allow(dead_code)]
    tracks: Vec<Track<FixedTimeNoteSequence>>,
    
    // Transport State
    #[allow(dead_code)]
    transport: TransportState,
    
    // Configuration
    #[allow(dead_code)]
    config: TuiConfig,

    // Event handling
    #[allow(dead_code)]
    event_handler: EventHandler,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SynthParameters {
    pub oscillator_waveform: audio_gen::Waveform,
    pub oscillator_volume: f32,
    pub filter_type: crate::tui::ui::widgets::selector::FilterType,
    pub filter_cutoff: f32,
    pub filter_center_frequency: f32,
    pub filter_bandwidth: f32,
    pub filter_resonance: f32,
    pub filter_mix: f32,
}

impl Default for SynthParameters {
    fn default() -> Self {
        Self {
            oscillator_waveform: audio_gen::Waveform::Sine,
            oscillator_volume: 0.75,
            filter_type: crate::tui::ui::widgets::selector::FilterType::LowPass,
            filter_cutoff: 1000.0,
            filter_center_frequency: 1000.0,
            filter_bandwidth: 200.0,
            filter_resonance: 0.3,
            filter_mix: 0.8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TransportState {
    pub is_playing: bool,
    pub is_recording: bool,
    pub tempo: f32,
    pub position: PlaybackPosition,
    pub focused_button: TransportButton,
    pub current_step: usize, // 0-15 for 16 steps
    pub last_step_time: std::time::Instant,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransportButton {
    Play,
    Stop,
}

impl Default for TransportState {
    fn default() -> Self {
        Self {
            is_playing: false,
            is_recording: false,
            tempo: 120.0,
            position: PlaybackPosition::default(),
            focused_button: TransportButton::Play,
            current_step: 0,
            last_step_time: std::time::Instant::now(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PlaybackPosition {
    pub measure: u32,
    pub beat: u8,
    pub tick: u16,
}

impl RoscoTuiApp {
    pub fn new() -> Result<Self, TuiError> {
        println!("Loading TUI config...");
        let config = TuiConfig::load_or_default()?;
        println!("Config loaded successfully");
        
        println!("Creating event handler...");
        let event_handler = EventHandler::new();
        println!("Event handler created");
        
        println!("Creating synthesizer panel...");
        let synthesizer_panel = SynthesizerPanel::new();
        println!("Synthesizer panel created");
        
        println!("Creating sequencer panel...");
        let sequencer_panel = SequencerPanel::new();
        println!("Sequencer panel created");
        
        println!("Creating filter manager...");
        let mut filter_manager = FilterManager::new();
        // Initialize with default filter settings
        filter_manager.update_filter_type(crate::tui::ui::widgets::selector::FilterType::LowPass);
        filter_manager.update_frequency(1000.0);
        filter_manager.update_bandwidth(200.0);
        filter_manager.update_resonance(0.3);
        filter_manager.update_mix(0.8);
        println!("Filter manager created");
        
        println!("Creating synth parameters...");
        let synth_params = SynthParameters::default();
        println!("Synth parameters created");
        
        println!("Creating transport state...");
        let transport = TransportState::default();
        println!("Transport state created");
        
        println!("Constructing final app struct...");
        Ok(Self {
            ui_state: UiState::default(),
            current_focus: FocusArea::Synthesizer(SynthSection::Oscillator),
            synthesizer_panel,
            sequencer_panel,
            audio_bridge: None,
            synth_params,
            filter_manager,
            tracks: Vec::new(),
            transport,
            config,
            event_handler,
        })
    }
    
    pub async fn run(&mut self) -> Result<(), TuiError> {
        // Initialize audio bridge with real audio engine
        println!("Initializing audio bridge with real audio engine...");
        match AudioBridge::new() {
            Ok(bridge) => {
                self.audio_bridge = Some(bridge);
                println!("Audio bridge initialized successfully");
                
                // Initialize sequencer data in audio state
                self.sync_sequencer_to_audio();
            }
            Err(e) => {
                println!("Warning: Could not initialize audio bridge: {:?}", e);
                println!("Running in silent mode...");
                self.audio_bridge = None;
            }
        }
        
        // Setup terminal
        if let Err(e) = enable_raw_mode() {
            eprintln!("Warning: Cannot enable raw mode ({}). TUI may not work properly.", e);
            eprintln!("Please run from a proper terminal application for full functionality.");
            return Err(TuiError::Terminal(format!("Terminal access required. Error: {}", e)));
        }
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        
        // Main application loop
        let result = self.run_app(&mut terminal).await;
        
        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen
        )?;
        terminal.show_cursor()?;
        
        result
    }
    
    async fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<(), TuiError> {
        loop {
            // Process audio feedback (step position updates from audio engine)
            self.process_audio_feedback()?;
            
            terminal.draw(|f| self.update_ui(f))?;
            
            if self.handle_events().await? {
                break;
            }
        }
        Ok(())
    }
    
    fn process_audio_feedback(&mut self) -> Result<(), TuiError> {
        if let Some(bridge) = &mut self.audio_bridge {
            let feedback = bridge.receive_audio_feedback();
            for fb in feedback {
                match fb {
                    crate::tui::audio_bridge::AudioFeedback::PlaybackPosition(step) => {
                        // Audio engine is master clock - it tells us the current step
                        let step_int = step as usize;
                        if step_int < 16 {
                            self.transport.current_step = step_int;
                            self.sequencer_panel.grid.set_playing_step(Some(step_int));
                        }
                    }
                    _ => {
                        // Handle other feedback types as needed
                    }
                }
            }
        }
        Ok(())
    }
    
    async fn handle_events(&mut self) -> Result<bool, TuiError> {
        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                return Ok(self.handle_key_event(key)?);
            }
        }
        Ok(false)
    }
    
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<bool, TuiError> {
        // Clear status message on any input
        self.ui_state.status_message = None;
        
        match key.code {
            KeyCode::Esc => {
                // Close overlays first, then quit
                if self.ui_state.show_pattern_browser {
                    self.ui_state.show_pattern_browser = false;
                    self.ui_state.status_message = Some("Pattern browser closed".to_string());
                    return Ok(false);
                } else if self.ui_state.show_help {
                    self.ui_state.show_help = false;
                    return Ok(false);
                } else {
                    return Ok(true);
                }
            }
            KeyCode::Char('q') => return Ok(true),
            KeyCode::F(1) => self.ui_state.show_help = !self.ui_state.show_help,
            KeyCode::F(2) => {
                // F2: Save current track as a new pattern
                self.save_current_track_as_pattern();
            }
            KeyCode::F(3) => {
                // F3: Toggle pattern browser
                self.ui_state.show_pattern_browser = !self.ui_state.show_pattern_browser;
                if self.ui_state.show_pattern_browser {
                    self.ui_state.pattern_browser_selection = 0;
                    self.ui_state.status_message = Some("Pattern browser opened. Use arrows to navigate, Enter to load, Esc to close.".to_string());
                } else {
                    self.ui_state.status_message = Some("Pattern browser closed".to_string());
                }
            }
            KeyCode::Tab => {
                if !self.ui_state.show_pattern_browser {
                    self.cycle_focus();
                }
            }
            KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => {
                if self.ui_state.show_pattern_browser {
                    self.handle_pattern_browser_navigation(key)?;
                } else {
                    self.handle_navigation(key)?;
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                if self.ui_state.show_pattern_browser {
                    self.handle_pattern_browser_selection()?;
                } else {
                    self.handle_activation()?;
                }
            }
            // Quick section switching with number keys
            KeyCode::Char('1') => {
                self.current_focus = FocusArea::Synthesizer(SynthSection::Oscillator);
                self.synthesizer_panel.current_section = crate::tui::ui::synthesizer::SynthesizerSubSection::Oscillator(crate::tui::ui::synthesizer::OscillatorSubSection::Waveform);
                // Sync the individual control's sub_focus
                self.synthesizer_panel.oscillator.sub_focus = crate::tui::ui::synthesizer::OscillatorSubSection::Waveform;
                self.ui_state.status_message = Some("Oscillator section".to_string());
            }
            KeyCode::Char('2') => {
                self.current_focus = FocusArea::Synthesizer(SynthSection::Filter);
                self.synthesizer_panel.current_section = crate::tui::ui::synthesizer::SynthesizerSubSection::Filter(crate::tui::ui::synthesizer::FilterSubSection::Type);
                // Sync the individual control's sub_focus
                self.synthesizer_panel.filter.sub_focus = crate::tui::ui::synthesizer::FilterSubSection::Type;
                self.ui_state.status_message = Some("Filter section focused".to_string());
            }
            KeyCode::Char('3') => {
                self.current_focus = FocusArea::Synthesizer(SynthSection::Envelope);
                self.synthesizer_panel.current_section = crate::tui::ui::synthesizer::SynthesizerSubSection::Envelope(crate::tui::ui::synthesizer::EnvelopeSubSection::AttackTime);
                // Sync the individual control's sub_focus
                self.synthesizer_panel.envelope.sub_focus = crate::tui::ui::synthesizer::EnvelopeSubSection::AttackTime;
                self.ui_state.status_message = Some("Envelope section focused".to_string());
            }
            KeyCode::Char('4') => {
                self.current_focus = FocusArea::Synthesizer(SynthSection::Effects);
                self.synthesizer_panel.current_section = crate::tui::ui::synthesizer::SynthesizerSubSection::Effects(crate::tui::ui::synthesizer::EffectsSubSection::DelayTime);
                self.synthesizer_panel.effects.sub_focus = crate::tui::ui::synthesizer::EffectsSubSection::DelayTime;
                self.ui_state.status_message = Some("Effects section focused".to_string());
            }
            KeyCode::Char('5') => {
                self.current_focus = FocusArea::Sequencer;
                self.ui_state.status_message = Some("Track Sequencer section".to_string());
            }
            KeyCode::Char('6') => {
                self.current_focus = FocusArea::TrackVolume;
                self.ui_state.status_message = Some("Track Volume section".to_string());
            }
            KeyCode::Char('7') => {
                self.current_focus = FocusArea::TrackPanning;
                self.ui_state.status_message = Some("Track Panning section".to_string());
            }
            KeyCode::Char('8') => {
                self.current_focus = FocusArea::Transport;
                self.ui_state.status_message = Some("Transport section".to_string());
            }
            // Fine adjustment with +/- keys
            KeyCode::Char('+') | KeyCode::Char('=') => {
                if let FocusArea::Synthesizer(_section) = &self.current_focus {
                    if let Some(update) = self.synthesizer_panel.handle_fine_adjustment(true) {
                        // Update local state for display
                        match &update {
                            crate::tui::audio_bridge::ParameterUpdate::OscillatorVolume(vol) => {
                                self.synth_params.oscillator_volume = *vol;
                                self.ui_state.status_message = Some(format!("Volume increased to {:.0}%", vol * 100.0));
                            }
                            crate::tui::audio_bridge::ParameterUpdate::FilterCutoff(cutoff) => {
                                self.synth_params.filter_cutoff = *cutoff;
                                self.ui_state.status_message = Some(format!("Cutoff increased to {:.0} Hz", cutoff));
                            }
                            crate::tui::audio_bridge::ParameterUpdate::FilterCenterFrequency(center_freq) => {
                                self.synth_params.filter_center_frequency = *center_freq;
                                self.ui_state.status_message = Some(format!("Center frequency increased to {:.0} Hz", center_freq));
                            }
                            crate::tui::audio_bridge::ParameterUpdate::FilterBandwidth(bandwidth) => {
                                self.synth_params.filter_bandwidth = *bandwidth;
                                self.ui_state.status_message = Some(format!("Bandwidth increased to {:.0} Hz", bandwidth));
                            }
                            crate::tui::audio_bridge::ParameterUpdate::FilterResonance(resonance) => {
                                self.synth_params.filter_resonance = *resonance;
                                self.ui_state.status_message = Some(format!("Resonance increased to {:.0}%", resonance * 100.0));
                            }
                            crate::tui::audio_bridge::ParameterUpdate::FilterMix(mix) => {
                                self.synth_params.filter_mix = *mix;
                                self.ui_state.status_message = Some(format!("Mix increased to {:.0}%", mix * 100.0));
                            }
                            crate::tui::audio_bridge::ParameterUpdate::EnvelopeAttackTime(val) => {
                                self.ui_state.status_message = Some(format!("Attack Time increased to {:.3}s", val));
                            }
                            crate::tui::audio_bridge::ParameterUpdate::EnvelopeAttackLevel(val) => {
                                self.ui_state.status_message = Some(format!("Attack Level increased to {:.0}%", val * 100.0));
                            }
                            crate::tui::audio_bridge::ParameterUpdate::EnvelopeDecayTime(val) => {
                                self.ui_state.status_message = Some(format!("Decay Time increased to {:.3}s", val));
                            }
                            crate::tui::audio_bridge::ParameterUpdate::EnvelopeDecayLevel(val) => {
                                self.ui_state.status_message = Some(format!("Decay Level increased to {:.0}%", val * 100.0));
                            }
                            crate::tui::audio_bridge::ParameterUpdate::EnvelopeSustainTime(val) => {
                                self.ui_state.status_message = Some(format!("Sustain Time increased to {:.3}s", val));
                            }
                            crate::tui::audio_bridge::ParameterUpdate::EnvelopeSustainLevel(val) => {
                                self.ui_state.status_message = Some(format!("Sustain Level increased to {:.0}%", val * 100.0));
                            }
                            crate::tui::audio_bridge::ParameterUpdate::EnvelopeReleaseTime(val) => {
                                self.ui_state.status_message = Some(format!("Release Time increased to {:.3}s", val));
                            }
                            crate::tui::audio_bridge::ParameterUpdate::EnvelopeReleaseLevel(val) => {
                                self.ui_state.status_message = Some(format!("Release Level increased to {:.0}%", val * 100.0));
                            }
                            _ => {}
                        }
                        self.send_parameter_update_real_time(update)?;
                    }
                }
            }
            KeyCode::Char('-') => {
                if let FocusArea::Synthesizer(_section) = &self.current_focus {
                    if let Some(update) = self.synthesizer_panel.handle_fine_adjustment(false) {
                        // Update local state for display
                        match &update {
                            crate::tui::audio_bridge::ParameterUpdate::OscillatorVolume(vol) => {
                                self.synth_params.oscillator_volume = *vol;
                                // self.ui_state.status_message = Some(format!("Volume decreased to {:.0}%", vol * 100.0));
                            }
                            crate::tui::audio_bridge::ParameterUpdate::FilterCutoff(cutoff) => {
                                self.synth_params.filter_cutoff = *cutoff;
                                // self.ui_state.status_message = Some(format!("Cutoff decreased to {:.0} Hz", cutoff));
                            }
                            crate::tui::audio_bridge::ParameterUpdate::FilterCenterFrequency(center_freq) => {
                                self.synth_params.filter_center_frequency = *center_freq;
                                // self.ui_state.status_message = Some(format!("Center frequency decreased to {:.0} Hz", center_freq));
                            }
                            crate::tui::audio_bridge::ParameterUpdate::FilterBandwidth(bandwidth) => {
                                self.synth_params.filter_bandwidth = *bandwidth;
                                // self.ui_state.status_message = Some(format!("Bandwidth decreased to {:.0} Hz", bandwidth));
                            }
                            crate::tui::audio_bridge::ParameterUpdate::FilterResonance(resonance) => {
                                self.synth_params.filter_resonance = *resonance;
                                // self.ui_state.status_message = Some(format!("Resonance decreased to {:.0}%", resonance * 100.0));
                            }
                            crate::tui::audio_bridge::ParameterUpdate::FilterMix(mix) => {
                                self.synth_params.filter_mix = *mix;
                                // self.ui_state.status_message = Some(format!("Mix decreased to {:.0}%", mix * 100.0));
                            }
                            crate::tui::audio_bridge::ParameterUpdate::EnvelopeAttackTime(_val) => {
                                // self.ui_state.status_message = Some(format!("Attack Time decreased to {:.3}s", val));
                            }
                            crate::tui::audio_bridge::ParameterUpdate::EnvelopeAttackLevel(_val) => {
                                // self.ui_state.status_message = Some(format!("Attack Level decreased to {:.0}%", val * 100.0));
                            }
                            crate::tui::audio_bridge::ParameterUpdate::EnvelopeDecayTime(_val) => {
                                // self.ui_state.status_message = Some(format!("Decay Time decreased to {:.3}s", val));
                            }
                            crate::tui::audio_bridge::ParameterUpdate::EnvelopeDecayLevel(_val) => {
                                // self.ui_state.status_message = Some(format!("Decay Level decreased to {:.0}%", val * 100.0));
                            }
                            crate::tui::audio_bridge::ParameterUpdate::EnvelopeSustainTime(_val) => {
                                // self.ui_state.status_message = Some(format!("Sustain Time decreased to {:.3}s", val));
                            }
                            crate::tui::audio_bridge::ParameterUpdate::EnvelopeSustainLevel(_val) => {
                                // self.ui_state.status_message = Some(format!("Sustain Level decreased to {:.0}%", val * 100.0));
                            }
                            crate::tui::audio_bridge::ParameterUpdate::EnvelopeReleaseTime(_val) => {
                                // self.ui_state.status_message = Some(format!("Release Time decreased to {:.3}s", val));
                            }
                            crate::tui::audio_bridge::ParameterUpdate::EnvelopeReleaseLevel(_val) => {
                                // self.ui_state.status_message = Some(format!("Release Level decreased to {:.0}%", val * 100.0));
                            }
                            _ => {}
                        }
                        self.send_parameter_update_real_time(update)?;
                    } else {
                        self.ui_state.status_message = Some("No fine adjustment available for current control".to_string());
                    }
                } else {
                    self.ui_state.status_message = Some("Fine adjustment only works in Synthesizer sections".to_string());
                }
            }
            // Reset parameter to default with 'r'
            KeyCode::Char('r') => {
                self.reset_current_parameter()?;
            }
            _ => {}
        }
        Ok(false)
    }
    
    fn cycle_focus(&mut self) {
        self.current_focus = match self.current_focus {
            FocusArea::Synthesizer(SynthSection::Oscillator) => {
                self.synthesizer_panel.current_section = crate::tui::ui::synthesizer::SynthesizerSubSection::Filter(crate::tui::ui::synthesizer::FilterSubSection::Type);
                self.synthesizer_panel.filter.sub_focus = crate::tui::ui::synthesizer::FilterSubSection::Type;
                FocusArea::Synthesizer(SynthSection::Filter)
            }
            FocusArea::Synthesizer(SynthSection::Filter) => {
                self.synthesizer_panel.current_section = crate::tui::ui::synthesizer::SynthesizerSubSection::Envelope(crate::tui::ui::synthesizer::EnvelopeSubSection::AttackTime);
                self.synthesizer_panel.envelope.sub_focus = crate::tui::ui::synthesizer::EnvelopeSubSection::AttackTime;
                FocusArea::Synthesizer(SynthSection::Envelope)
            }
            FocusArea::Synthesizer(SynthSection::Envelope) => {
                self.synthesizer_panel.current_section = crate::tui::ui::synthesizer::SynthesizerSubSection::Effects(crate::tui::ui::synthesizer::EffectsSubSection::DelayTime);
                self.synthesizer_panel.effects.sub_focus = crate::tui::ui::synthesizer::EffectsSubSection::DelayTime;
                FocusArea::Synthesizer(SynthSection::Effects)
            }
            FocusArea::Synthesizer(SynthSection::Effects) => FocusArea::Sequencer,
            FocusArea::Sequencer => FocusArea::TrackVolume,
            FocusArea::TrackVolume => FocusArea::TrackPanning,
            FocusArea::TrackPanning => FocusArea::Transport,
            FocusArea::Transport => {
                self.synthesizer_panel.current_section = crate::tui::ui::synthesizer::SynthesizerSubSection::Oscillator(crate::tui::ui::synthesizer::OscillatorSubSection::Waveform);
                self.synthesizer_panel.oscillator.sub_focus = crate::tui::ui::synthesizer::OscillatorSubSection::Waveform;
                FocusArea::Synthesizer(SynthSection::Oscillator)
            }
        };
    }
    
    fn handle_navigation(&mut self, key_event: KeyEvent) -> Result<(), TuiError> {
        match &self.current_focus {
            FocusArea::Synthesizer(section) => {
                self.handle_synth_navigation(*section, key_event)?;
            }
            FocusArea::Sequencer => {
                let actions = self.sequencer_panel.handle_key_event(key_event);
                self.process_sequencer_actions(actions)?;
            }
            FocusArea::TrackVolume => {
                self.handle_track_volume_navigation(key_event)?;
            }
            FocusArea::TrackPanning => {
                self.handle_track_panning_navigation(key_event)?;
            }
            FocusArea::Transport => {
                self.handle_transport_navigation(key_event)?;
            }
        }
        Ok(())
    }
    
    fn handle_synth_navigation(&mut self, section: SynthSection, key_event: KeyEvent) -> Result<(), TuiError> {
        match section {
            SynthSection::Oscillator => {
                let updates = self.synthesizer_panel.handle_input(key_event);
                for update in updates {
                    // Update local state for display
                    match &update {
                        crate::tui::audio_bridge::ParameterUpdate::OscillatorVolume(vol) => {
                            self.synth_params.oscillator_volume = *vol;
                        }
                        crate::tui::audio_bridge::ParameterUpdate::OscillatorWaveform(waveform) => {
                            self.synth_params.oscillator_waveform = *waveform;
                        }
                        _ => {}
                    }
                    self.send_parameter_update_real_time(update)?;
                }
            }
            SynthSection::Filter => {
                let updates = self.synthesizer_panel.handle_input(key_event);
                for update in updates {
                    // Update local state for display
                    match &update {
                        crate::tui::audio_bridge::ParameterUpdate::FilterType(filter_type) => {
                            self.synth_params.filter_type = filter_type.clone();
                            self.filter_manager.update_filter_type(filter_type.clone());
                        }
                        crate::tui::audio_bridge::ParameterUpdate::FilterCutoff(cutoff) => {
                            self.synth_params.filter_cutoff = *cutoff;
                            self.filter_manager.update_frequency(*cutoff);
                        }
                        crate::tui::audio_bridge::ParameterUpdate::FilterCenterFrequency(center_freq) => {
                            self.synth_params.filter_center_frequency = *center_freq;
                            self.filter_manager.update_frequency(*center_freq);
                        }
                        crate::tui::audio_bridge::ParameterUpdate::FilterBandwidth(bandwidth) => {
                            self.synth_params.filter_bandwidth = *bandwidth;
                            self.filter_manager.update_bandwidth(*bandwidth);
                        }
                        crate::tui::audio_bridge::ParameterUpdate::FilterResonance(resonance) => {
                            self.synth_params.filter_resonance = *resonance;
                            self.filter_manager.update_resonance(*resonance);
                        }
                        crate::tui::audio_bridge::ParameterUpdate::FilterMix(mix) => {
                            self.synth_params.filter_mix = *mix;
                            self.filter_manager.update_mix(*mix);
                        }
                        _ => {}
                    }
                    self.send_parameter_update_real_time(update)?;
                }
                
                // Apply filter to all tracks
                self.apply_filter_to_tracks();
            }
            SynthSection::Envelope => {
                let updates = self.synthesizer_panel.handle_input(key_event);
                for update in updates {
                    // Update envelope parameter feedback
                    self.ui_state.status_message = match &update {
                        crate::tui::audio_bridge::ParameterUpdate::EnvelopeAttackTime(val) => Some(format!("Attack Time: {:.3}s", val)),
                        crate::tui::audio_bridge::ParameterUpdate::EnvelopeAttackLevel(val) => Some(format!("Attack Level: {:.0}%", val * 100.0)),
                        crate::tui::audio_bridge::ParameterUpdate::EnvelopeDecayTime(val) => Some(format!("Decay Time: {:.3}s", val)),
                        crate::tui::audio_bridge::ParameterUpdate::EnvelopeDecayLevel(val) => Some(format!("Decay Level: {:.0}%", val * 100.0)),
                        crate::tui::audio_bridge::ParameterUpdate::EnvelopeSustainTime(val) => Some(format!("Sustain Time: {:.3}s", val)),
                        crate::tui::audio_bridge::ParameterUpdate::EnvelopeSustainLevel(val) => Some(format!("Sustain Level: {:.0}%", val * 100.0)),
                        crate::tui::audio_bridge::ParameterUpdate::EnvelopeReleaseTime(val) => Some(format!("Release Time: {:.3}s", val)),
                        crate::tui::audio_bridge::ParameterUpdate::EnvelopeReleaseLevel(val) => Some(format!("Release Level: {:.0}%", val * 100.0)),
                        _ => None
                    };
                    self.send_parameter_update_real_time(update)?;
                }
            }
            SynthSection::Effects => {
                let updates = self.synthesizer_panel.handle_input(key_event);
                for update in updates {
                    // Update effect parameter feedback
                    self.ui_state.status_message = match &update {
                        crate::tui::audio_bridge::ParameterUpdate::DelayTime(val) => Some(format!("Delay Time: {:.2}s", val)),
                        crate::tui::audio_bridge::ParameterUpdate::DelayFeedback(val) => Some(format!("Delay Feedback: {:.0}%", val * 100.0)),
                        crate::tui::audio_bridge::ParameterUpdate::DelayMix(val) => Some(format!("Delay Mix: {:.0}%", val * 100.0)),
                        crate::tui::audio_bridge::ParameterUpdate::FlangerRate(val) => Some(format!("Flanger Rate: {:.1} Hz", val)),
                        crate::tui::audio_bridge::ParameterUpdate::FlangerDepth(val) => Some(format!("Flanger Depth: {:.0}%", val * 100.0)),
                        crate::tui::audio_bridge::ParameterUpdate::FlangerMix(val) => Some(format!("Flanger Mix: {:.0}%", val * 100.0)),
                        crate::tui::audio_bridge::ParameterUpdate::LfoRate(val) => Some(format!("LFO Rate: {:.1} Hz", val)),
                        crate::tui::audio_bridge::ParameterUpdate::LfoDepth(val) => Some(format!("LFO Depth: {:.0}%", val * 100.0)),
                        _ => None
                    };
                    self.send_parameter_update_real_time(update)?;
                }
            }
        }
        Ok(())
    }

    fn handle_track_volume_navigation(&mut self, key_event: KeyEvent) -> Result<(), TuiError> {
        match key_event.code {
            KeyCode::Up | KeyCode::Down => {
                // Navigate between tracks
                let track_delta = if key_event.code == KeyCode::Down { 1 } else { -1 };
                let new_track = (self.sequencer_panel.grid.cursor.track as i8 + track_delta)
                    .clamp(0, 7) as u8;
                self.sequencer_panel.grid.cursor.track = new_track;
                // Set focus to track controls and specifically to volume
                self.sequencer_panel.grid.cursor.focus_area = crate::tui::ui::widgets::CursorFocus::TrackControls;
                let track = &mut self.sequencer_panel.grid.tracks[new_track as usize];
                track.selected_control = crate::tui::ui::widgets::TrackControl::Volume;
            }
            KeyCode::Left | KeyCode::Right => {
                // Adjust volume for the current track
                let delta = if key_event.code == KeyCode::Right { 0.05 } else { -0.05 };
                let track_idx = self.sequencer_panel.grid.cursor.track;
                let track = &mut self.sequencer_panel.grid.tracks[track_idx as usize];
                track.adjust_volume(delta);
                self.ui_state.status_message = Some(format!("Track {} Volume: {:.0}%", 
                    track.track_number, track.volume * 100.0));
            }
            _ => {}
        }
        Ok(())
    }
    
    fn handle_track_panning_navigation(&mut self, key_event: KeyEvent) -> Result<(), TuiError> {
        match key_event.code {
            KeyCode::Up | KeyCode::Down => {
                // Navigate between tracks
                let track_delta = if key_event.code == KeyCode::Down { 1 } else { -1 };
                let new_track = (self.sequencer_panel.grid.cursor.track as i8 + track_delta)
                    .clamp(0, 7) as u8;
                self.sequencer_panel.grid.cursor.track = new_track;
                // Set focus to track controls and specifically to panning
                self.sequencer_panel.grid.cursor.focus_area = crate::tui::ui::widgets::CursorFocus::TrackControls;
                let track = &mut self.sequencer_panel.grid.tracks[new_track as usize];
                track.selected_control = crate::tui::ui::widgets::TrackControl::Pan;
            }
            KeyCode::Left | KeyCode::Right => {
                // Adjust panning for the current track
                let delta = if key_event.code == KeyCode::Right { 0.1 } else { -0.1 };
                let track_idx = self.sequencer_panel.grid.cursor.track;
                let track = &mut self.sequencer_panel.grid.tracks[track_idx as usize];
                track.adjust_pan(delta);
                self.ui_state.status_message = Some(format!("Track {} Pan: {:.1}", 
                    track.track_number, track.pan));
            }
            _ => {}
        }
        Ok(())
    }
    
    fn handle_transport_navigation(&mut self, key_event: KeyEvent) -> Result<(), TuiError> {
        match key_event.code {
            KeyCode::Left => {
                self.transport.focused_button = TransportButton::Play;
                self.ui_state.status_message = Some("Play button focused".to_string());
            }
            KeyCode::Right => {
                self.transport.focused_button = TransportButton::Stop;
                self.ui_state.status_message = Some("Stop button focused".to_string());
            }
            _ => {}
        }
        Ok(())
    }
    
    fn handle_activation(&mut self) -> Result<(), TuiError> {
        match &self.current_focus {
            FocusArea::Transport => {
                match self.transport.focused_button {
                    TransportButton::Play => {
                        self.transport.is_playing = true;
                        self.transport.last_step_time = std::time::Instant::now();
                        self.ui_state.status_message = Some("Playing".to_string());
                        let transport_cmd = crate::tui::audio_bridge::ParameterUpdate::TransportPlay;
                        self.send_parameter_update_real_time(transport_cmd)?;
                    }
                    TransportButton::Stop => {
                        self.transport.is_playing = false;
                        // Keep the current step position highlighted when stopped
                        // The grid will continue to show the green highlight on the current step
                        self.ui_state.status_message = Some("Stopped".to_string());
                        let transport_cmd = crate::tui::audio_bridge::ParameterUpdate::TransportStop;
                        self.send_parameter_update_real_time(transport_cmd)?;
                    }
                }
            }
            FocusArea::Sequencer => {
                // Handle Enter/Space in sequencer by creating a fake key event
                let key_event = crossterm::event::KeyEvent {
                    code: crossterm::event::KeyCode::Enter,
                    modifiers: crossterm::event::KeyModifiers::empty(),
                    kind: crossterm::event::KeyEventKind::Press,
                    state: crossterm::event::KeyEventState::empty(),
                };
                let actions = self.sequencer_panel.handle_key_event(key_event);
                self.process_sequencer_actions(actions)?;
            }
            _ => {}
        }
        Ok(())
    }
    
    // fn send_parameter_update(&mut self) -> Result<(), TuiError> {
    //     if let Some(_bridge) = &mut self.audio_bridge {
    //         // TODO: Send parameter updates to audio thread
    //     }
    //     Ok(())
    // }
    
    fn send_parameter_update_real_time(&mut self, update: crate::tui::audio_bridge::ParameterUpdate) -> Result<(), TuiError> {
        if let Some(bridge) = &mut self.audio_bridge {
            bridge.send_parameter_update(update)?;
            self.ui_state.status_message = Some("Parameter updated".to_string());
        }
        Ok(())
    }
    
    /// Sync sequencer grid data to audio engine state
    fn sync_sequencer_to_audio(&mut self) {
        if let Some(bridge) = &mut self.audio_bridge {
            let audio_state = bridge.get_audio_state();
            // Sync sequencer steps to audio state
            for track_idx in 0..8 {
                if track_idx < self.sequencer_panel.grid.tracks.len() {
                    let track = &self.sequencer_panel.grid.tracks[track_idx];
                    
                    // Set track volume
                    audio_state.track_volumes[track_idx].store(track.volume, std::sync::atomic::Ordering::Relaxed);
                    
                    // Sync step states and individual step frequencies
                    for step_idx in 0..16 {
                        if step_idx < track.steps.len() {
                            let audio_index = track_idx * 16 + step_idx;
                            let step = &track.steps[step_idx];
                            
                            // Set step enabled state
                            audio_state.track_steps[audio_index].store(step.enabled, std::sync::atomic::Ordering::Relaxed);
                            
                            // Set step-specific frequency
                            let step_frequency = step.frequency.get_frequency(3);
                            audio_state.step_frequencies[audio_index].store(step_frequency, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                }
            }
            
            // Sync transport state
            audio_state.tempo.store(self.transport.tempo, std::sync::atomic::Ordering::Relaxed);
            audio_state.current_step.store(self.transport.current_step, std::sync::atomic::Ordering::Relaxed);
            audio_state.is_playing.store(self.transport.is_playing, std::sync::atomic::Ordering::Relaxed);
        }
    }
    
    fn process_sequencer_actions(&mut self, actions: Vec<crate::tui::ui::sequencer::SequencerAction>) -> Result<(), TuiError> {
        use crate::tui::ui::sequencer::SequencerAction;
        
        for action in actions {
            match action {
                SequencerAction::StepToggled { track, step } => {
                    let enabled = self.sequencer_panel.grid.tracks[track as usize].steps[step as usize].enabled;
                    self.ui_state.status_message = Some(format!(
                        "Track {} Step {} {}", 
                        track + 1, 
                        step + 1, 
                        if enabled { "enabled" } else { "disabled" }
                    ));
                    
                    // Update audio state directly for immediate response
                    if let Some(bridge) = &mut self.audio_bridge {
                        let audio_state = bridge.get_audio_state();
                        if (track as usize) < 8 && (step as usize) < 16 {
                            let audio_index = (track as usize) * 16 + (step as usize);
                            audio_state.track_steps[audio_index].store(enabled, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                    
                    // Send to audio bridge if available
                    let update = crate::tui::audio_bridge::ParameterUpdate::SequencerStep {
                        track,
                        step,
                        enabled,
                    };
                    self.send_parameter_update_real_time(update)?;
                }
                SequencerAction::FrequencyChanged { track, step, frequency } => {
                    self.ui_state.status_message = Some(format!(
                        "Track {} Step {} frequency: {} ({:.1} Hz)", 
                        track + 1, 
                        step + 1,
                        frequency,
                        frequency.get_frequency(3)
                    ));
                    
                    // Update step frequency in audio state in real-time
                    if let Some(bridge) = &mut self.audio_bridge {
                        let audio_state = bridge.get_audio_state();
                        if (track as usize) < 8 && (step as usize) < 16 {
                            let audio_index = (track as usize) * 16 + (step as usize);
                            let step_frequency = frequency.get_frequency(3);
                            audio_state.step_frequencies[audio_index].store(step_frequency, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                }
                SequencerAction::TrackVolumeChanged { track, volume } => {
                    self.ui_state.status_message = Some(format!(
                        "Track {} volume: {:.0}%", 
                        track + 1, 
                        volume * 100.0
                    ));
                    
                    // Update audio state through bridge
                    if let Some(bridge) = &self.audio_bridge {
                        let audio_state = bridge.get_audio_state();
                        if (track as usize) < 8 {
                            audio_state.track_volumes[track as usize].store(volume, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                }
                SequencerAction::TrackPanChanged { track, pan } => {
                    self.ui_state.status_message = Some(format!(
                        "Track {} pan: {:.1}", 
                        track + 1, 
                        pan
                    ));
                }
                SequencerAction::TrackMuteToggled { track } => {
                    let muted = self.sequencer_panel.grid.tracks[track as usize].mute;
                    self.ui_state.status_message = Some(format!(
                        "Track {} {}", 
                        track + 1, 
                        if muted { "muted" } else { "unmuted" }
                    ));
                }
                SequencerAction::TrackSoloToggled { track } => {
                    let soloed = self.sequencer_panel.grid.tracks[track as usize].solo;
                    self.ui_state.status_message = Some(format!(
                        "Track {} {}", 
                        track + 1, 
                        if soloed { "soloed" } else { "unsoloed" }
                    ));
                }
                SequencerAction::TrackCleared { track } => {
                    self.ui_state.status_message = Some(format!("Track {} cleared", track + 1));
                }
                SequencerAction::PatternCopied => {
                    self.ui_state.status_message = Some("Pattern copied to clipboard".to_string());
                }
                SequencerAction::PatternPasted => {
                    self.ui_state.status_message = Some("Pattern pasted from clipboard".to_string());
                }
                SequencerAction::PatternStored { pattern_id: _ } => {
                    self.ui_state.status_message = Some("Pattern stored".to_string());
                }
                SequencerAction::PatternLoaded { pattern_id: _ } => {
                    self.ui_state.status_message = Some("Pattern loaded".to_string());
                }
                SequencerAction::PatternBrowserToggled => {
                    let visible = self.sequencer_panel.is_pattern_browser_visible();
                    self.ui_state.status_message = Some(format!(
                        "Pattern browser {}", 
                        if visible { "opened" } else { "closed" }
                    ));
                }
                SequencerAction::SelectionStarted => {
                    self.ui_state.status_message = Some("Selection started".to_string());
                }
                SequencerAction::SelectionCleared => {
                    self.ui_state.status_message = Some("Selection cleared".to_string());
                }
            }
        }
        Ok(())
    }
    
    fn reset_current_parameter(&mut self) -> Result<(), TuiError> {
        if let FocusArea::Synthesizer(SynthSection::Oscillator) = &self.current_focus {
            match self.synthesizer_panel.current_section {
                crate::tui::ui::synthesizer::SynthesizerSubSection::Oscillator(crate::tui::ui::synthesizer::OscillatorSubSection::Waveform) => {
                    self.synthesizer_panel.oscillator.waveform_selector.selected = 0; // Reset to Sine
                    self.synth_params.oscillator_waveform = self.synthesizer_panel.oscillator.waveform_selector.selected_waveform();
                    let update = crate::tui::audio_bridge::ParameterUpdate::OscillatorWaveform(
                        self.synthesizer_panel.oscillator.waveform_selector.selected_waveform()
                    );
                    self.send_parameter_update_real_time(update)?;
                    self.ui_state.status_message = Some("Waveform reset to Sine".to_string());
                }
                crate::tui::ui::synthesizer::SynthesizerSubSection::Oscillator(crate::tui::ui::synthesizer::OscillatorSubSection::Volume) => {
                    self.synthesizer_panel.oscillator.volume_slider.set_value(0.75);
                    self.synth_params.oscillator_volume = 0.75;
                    let update = crate::tui::audio_bridge::ParameterUpdate::OscillatorVolume(0.75);
                    self.send_parameter_update_real_time(update)?;
                    self.ui_state.status_message = Some("Volume reset to 75%".to_string());
                }
                _ => {}
            }
        } else if let FocusArea::Synthesizer(SynthSection::Filter) = &self.current_focus {
            match self.synthesizer_panel.current_section {
                crate::tui::ui::synthesizer::SynthesizerSubSection::Filter(crate::tui::ui::synthesizer::FilterSubSection::Type) => {
                    self.synthesizer_panel.filter.filter_type.selected = 0; // Reset to LowPass
                    self.synth_params.filter_type = self.synthesizer_panel.filter.filter_type.selected_filter().clone();
                    self.filter_manager.update_filter_type(self.synthesizer_panel.filter.filter_type.selected_filter().clone());
                    let update = crate::tui::audio_bridge::ParameterUpdate::FilterType(
                        self.synthesizer_panel.filter.filter_type.selected_filter().clone()
                    );
                    self.send_parameter_update_real_time(update)?;
                    self.ui_state.status_message = Some("Filter type reset to LowPass".to_string());
                    self.apply_filter_to_tracks();
                }
                crate::tui::ui::synthesizer::SynthesizerSubSection::Filter(crate::tui::ui::synthesizer::FilterSubSection::Frequency) => {
                    self.synthesizer_panel.filter.frequency_slider.set_value(1000.0);
                    self.synth_params.filter_cutoff = 1000.0;
                    self.filter_manager.update_frequency(1000.0);
                    let update = crate::tui::audio_bridge::ParameterUpdate::FilterCutoff(1000.0);
                    self.send_parameter_update_real_time(update)?;
                    self.ui_state.status_message = Some("Cutoff reset to 1000 Hz".to_string());
                    self.apply_filter_to_tracks();
                }
                crate::tui::ui::synthesizer::SynthesizerSubSection::Filter(crate::tui::ui::synthesizer::FilterSubSection::Bandwidth) => {
                    self.synthesizer_panel.filter.bandwidth_slider.set_value(200.0);
                    self.synth_params.filter_bandwidth = 200.0;
                    self.filter_manager.update_bandwidth(200.0);
                    let update = crate::tui::audio_bridge::ParameterUpdate::FilterBandwidth(200.0);
                    self.send_parameter_update_real_time(update)?;
                    self.ui_state.status_message = Some("Bandwidth reset to 200 Hz".to_string());
                    self.apply_filter_to_tracks();
                }
                crate::tui::ui::synthesizer::SynthesizerSubSection::Filter(crate::tui::ui::synthesizer::FilterSubSection::Resonance) => {
                    self.synthesizer_panel.filter.resonance_slider.set_value(0.3);
                    self.synth_params.filter_resonance = 0.3;
                    self.filter_manager.update_resonance(0.3);
                    let update = crate::tui::audio_bridge::ParameterUpdate::FilterResonance(0.3);
                    self.send_parameter_update_real_time(update)?;
                    self.ui_state.status_message = Some("Resonance reset to 30%".to_string());
                    self.apply_filter_to_tracks();
                }
                crate::tui::ui::synthesizer::SynthesizerSubSection::Filter(crate::tui::ui::synthesizer::FilterSubSection::Mix) => {
                    self.synthesizer_panel.filter.mix_slider.set_value(0.8);
                    self.synth_params.filter_mix = 0.8;
                    self.filter_manager.update_mix(0.8);
                    let update = crate::tui::audio_bridge::ParameterUpdate::FilterMix(0.8);
                    self.send_parameter_update_real_time(update)?;
                    self.ui_state.status_message = Some("Mix reset to 80%".to_string());
                    self.apply_filter_to_tracks();
                }
                _ => {}
            }
        }
        Ok(())
    }
    
    fn apply_filter_to_tracks(&mut self) {
        // Apply the current filter settings to all tracks
        for track in &mut self.tracks {
            self.filter_manager.apply_to_track_effects(&mut track.effects);
        }

        // Also apply to the sequencer panel tracks if they exist
        for _track in &mut self.sequencer_panel.grid.tracks {
            // Note: This assumes the sequencer tracks have effects
            // We may need to add effects to the sequencer track structure
        }
    }

    fn save_current_track_as_pattern(&mut self) {
        let track_idx = self.sequencer_panel.grid.cursor.track as usize;
        let track = &self.sequencer_panel.grid.tracks[track_idx];
        let pattern_count = self.sequencer_panel.get_pattern_manager().count();
        let pattern_name = format!("Pattern {} (Track {})", pattern_count + 1, track.track_number);

        if let Some(_pattern_id) = self.sequencer_panel.store_track_as_pattern(track_idx, pattern_name.clone()) {
            self.ui_state.status_message = Some(format!("Pattern saved: {}", pattern_name));

            // Sync to audio if needed
            self.sync_sequencer_to_audio();
        }
    }

    fn handle_pattern_browser_navigation(&mut self, key: KeyEvent) -> Result<(), TuiError> {
        let pattern_count = self.sequencer_panel.get_pattern_manager().count();
        if pattern_count == 0 {
            return Ok(());
        }

        match key.code {
            KeyCode::Up => {
                if self.ui_state.pattern_browser_selection > 0 {
                    self.ui_state.pattern_browser_selection -= 1;
                }
            }
            KeyCode::Down => {
                if self.ui_state.pattern_browser_selection < pattern_count.saturating_sub(1) {
                    self.ui_state.pattern_browser_selection += 1;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_pattern_browser_selection(&mut self) -> Result<(), TuiError> {
        let patterns: Vec<_> = self.sequencer_panel.get_pattern_manager().list_patterns()
            .into_iter()
            .map(|p| (p.id.clone(), p.name.clone()))
            .collect();

        if let Some((pattern_id, pattern_name)) = patterns.get(self.ui_state.pattern_browser_selection) {
            let track_idx = self.sequencer_panel.grid.cursor.track as usize;
            if self.sequencer_panel.load_pattern_to_track(pattern_id, track_idx) {
                self.ui_state.status_message = Some(format!("Loaded '{}' to Track {}", pattern_name, track_idx + 1));
                self.ui_state.show_pattern_browser = false;

                // Sync to audio
                self.sync_sequencer_to_audio();
            }
        }
        Ok(())
    }
    
    fn update_ui(&mut self, frame: &mut Frame) {
        let size = frame.size();

        if self.ui_state.show_help {
            self.render_help(frame, size);
            return;
        }

        if self.ui_state.show_pattern_browser {
            self.render_pattern_browser(frame, size);
            return;
        }
        
        // Main layout: synthesizer top, sequencer bottom
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(20), // Synthesizer (reduced further to ensure 8 tracks fit)
                Constraint::Min(24),        // Sequencer (8 tracks * 2 rows + borders + step numbers = 16 + 8 = 24)
                Constraint::Min(3),         // Status bar
            ])
            .split(size);
        
        self.render_synthesizer(frame, chunks[0]);
        self.render_sequencer_sections(frame, chunks[1]);
        self.render_status_bar(frame, chunks[2]);
    }
    
    fn render_synthesizer(&self, frame: &mut Frame, area: Rect) {
        let title = match &self.current_focus {
            FocusArea::Synthesizer(_) => "SYNTHESIZER [FOCUSED]",
            _ => "SYNTHESIZER",
        };
        
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL);
        
        // Split synthesizer into sections
        let synth_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25), // Oscillator
                Constraint::Percentage(25), // Filter
                Constraint::Percentage(25), // Envelope
                Constraint::Percentage(25), // Effects
            ])
            .split(block.inner(area));
        
        frame.render_widget(block, area);
        
        self.render_oscillator_section(frame, synth_chunks[0]);
        self.render_filter_section(frame, synth_chunks[1]);
        self.render_envelope_section(frame, synth_chunks[2]);
        self.render_effects_section(frame, synth_chunks[3]);
    }
    
    fn render_oscillator_section(&self, frame: &mut Frame, area: Rect) {
        let focused = matches!(self.current_focus, FocusArea::Synthesizer(SynthSection::Oscillator));
        let title = if focused { "1 - OSCILLATOR [FOCUSED]" } else { "1 - OSCILLATOR" };
        
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL);
        
        let inner = block.inner(area);
        frame.render_widget(block, area);
        
        // Split oscillator area vertically for controls
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Waveform
                Constraint::Length(2), // Volume
            ])
            .split(inner);
        
        // Render waveform control
        let waveform_focused = focused && self.synthesizer_panel.current_section == crate::tui::ui::synthesizer::SynthesizerSubSection::Oscillator(crate::tui::ui::synthesizer::OscillatorSubSection::Waveform);
        let waveform_style = if waveform_focused { 
            Style::default().fg(Color::Cyan) 
        } else { 
            Style::default().fg(Color::White) 
        };
        let waveform_text = format!("Wave: {:?} {}", 
            self.synthesizer_panel.get_waveform(),
            if waveform_focused { "◄" } else { "" }
        );
        frame.render_widget(Paragraph::new(waveform_text).style(waveform_style), chunks[0]);
        
        // Render volume control
        let vol_focused = focused && self.synthesizer_panel.current_section == crate::tui::ui::synthesizer::SynthesizerSubSection::Oscillator(crate::tui::ui::synthesizer::OscillatorSubSection::Volume);
        let vol_style = if vol_focused { 
            Style::default().fg(Color::Cyan) 
        } else { 
            Style::default().fg(Color::White) 
        };
        let vol_slider = &self.synthesizer_panel.oscillator.volume_slider;
        let vol_text = format!("Vol:  {} {:.0}% {}", 
            vol_slider.render_bar(),
            vol_slider.value * 100.0,
            if vol_focused { "◄" } else { "" }
        );
        frame.render_widget(Paragraph::new(vol_text).style(vol_style), chunks[1]);
    }
    
    fn render_filter_section(&self, frame: &mut Frame, area: Rect) {
        let focused = matches!(self.current_focus, FocusArea::Synthesizer(SynthSection::Filter));
        let title = if focused { "2 - FILTER [FOCUSED]" } else { "2 - FILTER" };
        
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL);
        
        let inner = block.inner(area);
        frame.render_widget(block, area);
        
        // Determine which controls to show based on filter type
        let filter_type = self.synthesizer_panel.get_filter_type();
        let has_bandwidth = matches!(filter_type, crate::tui::ui::widgets::selector::FilterType::BandPass | crate::tui::ui::widgets::selector::FilterType::Notch);
        
        // Create constraints based on filter type
        let constraints = if has_bandwidth {
            vec![
                Constraint::Length(2), // Type
                Constraint::Length(2), // Frequency (Center for BP/Notch)
                Constraint::Length(2), // Bandwidth
                Constraint::Length(2), // Resonance
                Constraint::Length(2), // Mix
            ]
        } else {
            vec![
                Constraint::Length(2), // Type
                Constraint::Length(2), // Frequency (Cutoff for LP/HP)
                Constraint::Length(2), // Resonance
                Constraint::Length(2), // Mix
            ]
        };
        
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(inner);
        
        // Render filter type control
        let filter_type_focused = focused && self.synthesizer_panel.current_section == crate::tui::ui::synthesizer::SynthesizerSubSection::Filter(crate::tui::ui::synthesizer::FilterSubSection::Type);
        let filter_type_style = if filter_type_focused { 
            Style::default().fg(Color::Cyan) 
        } else { 
            Style::default().fg(Color::White) 
        };
        let filter_type_text = format!("Type: {:?} {}", 
            self.synthesizer_panel.get_filter_type(),
            if filter_type_focused { "◄" } else { "" }
        );
        frame.render_widget(Paragraph::new(filter_type_text).style(filter_type_style), chunks[0]);
        
        // Render frequency control (label changes based on filter type)
        let freq_label = if has_bandwidth { "Center" } else { "Cutoff" };
        let freq_focused = focused && self.synthesizer_panel.current_section == crate::tui::ui::synthesizer::SynthesizerSubSection::Filter(crate::tui::ui::synthesizer::FilterSubSection::Frequency);
        let freq_style = if freq_focused { 
            Style::default().fg(Color::Cyan) 
        } else { 
            Style::default().fg(Color::White) 
        };
        let freq_slider = &self.synthesizer_panel.filter.frequency_slider;
        let freq_text = format!("{}: {} {:.0} Hz {}", 
            freq_label,
            freq_slider.render_bar(),
            freq_slider.value,
            if freq_focused { "◄" } else { "" }
        );
        frame.render_widget(Paragraph::new(freq_text).style(freq_style), chunks[1]);
        
        if has_bandwidth {
            // Render bandwidth slider (only for BP/Notch)
            let bw_focused = focused && self.synthesizer_panel.current_section == crate::tui::ui::synthesizer::SynthesizerSubSection::Filter(crate::tui::ui::synthesizer::FilterSubSection::Bandwidth);
            let bw_style = if bw_focused { 
                Style::default().fg(Color::Cyan) 
            } else { 
                Style::default().fg(Color::White) 
            };
            let bw_slider = &self.synthesizer_panel.filter.bandwidth_slider;
            let bw_text = format!("BW: {} {:.0} Hz {}", 
                bw_slider.render_bar(),
                bw_slider.value,
                if bw_focused { "◄" } else { "" }
            );
            frame.render_widget(Paragraph::new(bw_text).style(bw_style), chunks[2]);
            
            // Render resonance control
            let resonance_focused = focused && self.synthesizer_panel.current_section == crate::tui::ui::synthesizer::SynthesizerSubSection::Filter(crate::tui::ui::synthesizer::FilterSubSection::Resonance);
            let resonance_style = if resonance_focused { 
                Style::default().fg(Color::Cyan) 
            } else { 
                Style::default().fg(Color::White) 
            };
            let resonance_slider = &self.synthesizer_panel.filter.resonance_slider;
            let resonance_text = format!("Res: {} {:.0}% {}", 
                resonance_slider.render_bar(),
                resonance_slider.value * 100.0,
                if resonance_focused { "◄" } else { "" }
            );
            frame.render_widget(Paragraph::new(resonance_text).style(resonance_style), chunks[3]);
            
            // Render mix control
            let mix_focused = focused && self.synthesizer_panel.current_section == crate::tui::ui::synthesizer::SynthesizerSubSection::Filter(crate::tui::ui::synthesizer::FilterSubSection::Mix);
            let mix_style = if mix_focused { 
                Style::default().fg(Color::Cyan) 
            } else { 
                Style::default().fg(Color::White) 
            };
            let mix_slider = &self.synthesizer_panel.filter.mix_slider;
            let mix_text = format!("Mix: {} {:.0}% {}", 
                mix_slider.render_bar(),
                mix_slider.value * 100.0,
                if mix_focused { "◄" } else { "" }
            );
            frame.render_widget(Paragraph::new(mix_text).style(mix_style), chunks[4]);
        } else {
            // Render resonance control (for LP/HP)
            let resonance_focused = focused && self.synthesizer_panel.current_section == crate::tui::ui::synthesizer::SynthesizerSubSection::Filter(crate::tui::ui::synthesizer::FilterSubSection::Resonance);
            let resonance_style = if resonance_focused { 
                Style::default().fg(Color::Cyan) 
            } else { 
                Style::default().fg(Color::White) 
            };
            let resonance_slider = &self.synthesizer_panel.filter.resonance_slider;
            let resonance_text = format!("Res: {} {:.0}% {}", 
                resonance_slider.render_bar(),
                resonance_slider.value * 100.0,
                if resonance_focused { "◄" } else { "" }
            );
            frame.render_widget(Paragraph::new(resonance_text).style(resonance_style), chunks[2]);
            
            // Render mix control
            let mix_focused = focused && self.synthesizer_panel.current_section == crate::tui::ui::synthesizer::SynthesizerSubSection::Filter(crate::tui::ui::synthesizer::FilterSubSection::Mix);
            let mix_style = if mix_focused { 
                Style::default().fg(Color::Cyan) 
            } else { 
                Style::default().fg(Color::White) 
            };
            let mix_slider = &self.synthesizer_panel.filter.mix_slider;
            let mix_text = format!("Mix: {} {:.0}% {}", 
                mix_slider.render_bar(),
                mix_slider.value * 100.0,
                if mix_focused { "◄" } else { "" }
            );
            frame.render_widget(Paragraph::new(mix_text).style(mix_style), chunks[3]);
        }
    }
    
    fn render_envelope_section(&self, frame: &mut Frame, area: Rect) {
        let focused = matches!(self.current_focus, FocusArea::Synthesizer(SynthSection::Envelope));
        let title = if focused { "3 - ENVELOPE [FOCUSED]" } else { "3 - ENVELOPE" };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Render envelope controls using the synthesizer panel
        if let SynthesizerSubSection::Envelope(env_section) = self.synthesizer_panel.current_section {
            self.synthesizer_panel.envelope.render(inner, frame.buffer_mut(), focused, env_section);
        } else if focused {
            // Fallback - render with default section
            self.synthesizer_panel.envelope.render(inner, frame.buffer_mut(), focused, crate::tui::ui::synthesizer::EnvelopeSubSection::AttackTime);
        } else {
            // Non-focused, render with placeholder
            self.synthesizer_panel.envelope.render(inner, frame.buffer_mut(), false, crate::tui::ui::synthesizer::EnvelopeSubSection::AttackTime);
        }
    }

    fn render_effects_section(&self, frame: &mut Frame, area: Rect) {
        let focused = matches!(self.current_focus, FocusArea::Synthesizer(SynthSection::Effects));
        let title = if focused { "4 - EFFECTS [FOCUSED]" } else { "4 - EFFECTS" };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Render effects controls using the synthesizer panel
        if let SynthesizerSubSection::Effects(fx_section) = self.synthesizer_panel.current_section {
            self.synthesizer_panel.effects.render(inner, frame.buffer_mut(), focused, fx_section);
        } else if focused {
            // Fallback - render with default section
            self.synthesizer_panel.effects.render(inner, frame.buffer_mut(), focused, crate::tui::ui::synthesizer::EffectsSubSection::DelayTime);
        } else {
            // Non-focused, render with placeholder
            self.synthesizer_panel.effects.render(inner, frame.buffer_mut(), false, crate::tui::ui::synthesizer::EffectsSubSection::DelayTime);
        }
    }

    #[allow(dead_code)]
    fn render_placeholder_section(&self, frame: &mut Frame, area: Rect, title: &str) {
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL);
        
        let inner = block.inner(area);
        frame.render_widget(block, area);
        
        let paragraph = Paragraph::new("TODO");
        frame.render_widget(paragraph, inner);
    }
    
    fn render_sequencer_sections(&mut self, frame: &mut Frame, area: Rect) {
        // Split into three sections: grid, volume controls, panning controls, and transport
        let sections = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50), // Track grid section
                Constraint::Percentage(25), // Volume controls section
                Constraint::Percentage(25), // Panning controls section
            ])
            .split(area);
        
        // Each section needs to be split vertically to include transport at bottom
        let grid_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(20), // Grid area
                Constraint::Length(3), // Transport
            ])
            .split(sections[0]);
            
        let volume_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(20), // Volume controls
                Constraint::Length(3), // Empty space for alignment
            ])
            .split(sections[1]);
            
        let pan_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(20), // Pan controls
                Constraint::Length(3), // Empty space for alignment
            ])
            .split(sections[2]);
        
        // Render section 5: Track Grid
        self.render_track_grid_section(frame, grid_chunks[0]);
        
        // Render section 6: Volume Controls
        self.render_track_volume_section(frame, volume_chunks[0]);
        
        // Render section 7: Panning Controls
        self.render_track_panning_section(frame, pan_chunks[0]);
        
        // Render transport only once in the grid section
        self.render_transport(frame, grid_chunks[1]);
    }
    
    fn render_track_grid_section(&mut self, frame: &mut Frame, area: Rect) {
        let title = match &self.current_focus {
            FocusArea::Sequencer => "5 - TRACK GRID [FOCUSED]",
            _ => "5 - TRACK GRID",
        };
        
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL);
        
        let inner = block.inner(area);
        frame.render_widget(block, area);
        
        let focused = matches!(self.current_focus, FocusArea::Sequencer);
        self.sequencer_panel.grid.focused = focused;
        
        // Render only the grid part without volume/pan controls
        self.render_sequencer_grid_only(frame, inner);
    }
    
    fn render_track_volume_section(&mut self, frame: &mut Frame, area: Rect) {
        let title = match &self.current_focus {
            FocusArea::TrackVolume => "6 - TRACK VOLUME [FOCUSED]",
            _ => "6 - TRACK VOLUME",
        };
        
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL);
        
        let inner = block.inner(area);
        frame.render_widget(block, area);
        
        let focused = matches!(self.current_focus, FocusArea::TrackVolume);
        self.render_volume_controls_only(frame, inner, focused);
    }
    
    fn render_track_panning_section(&mut self, frame: &mut Frame, area: Rect) {
        let title = match &self.current_focus {
            FocusArea::TrackPanning => "7 - TRACK PANNING [FOCUSED]",
            _ => "7 - TRACK PANNING",
        };
        
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL);
        
        let inner = block.inner(area);
        frame.render_widget(block, area);
        
        let focused = matches!(self.current_focus, FocusArea::TrackPanning);
        self.render_panning_controls_only(frame, inner, focused);
    }
    
    fn render_sequencer_grid_only(&mut self, frame: &mut Frame, area: Rect) {
        // Create a custom grid widget that only shows the steps/frequency grid without controls
        let grid = self.sequencer_panel.grid.clone();
        frame.render_widget(GridOnlyWidget { grid }, area);
    }
    
    fn render_volume_controls_only(&mut self, frame: &mut Frame, area: Rect, focused: bool) {
        use ratatui::{
            style::{Color, Style},
            widgets::Paragraph,
        };
        
        let style = if focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::White)
        };
        
        // Render volume controls for each track
        for (track_idx, track) in self.sequencer_panel.grid.tracks.iter().enumerate() {
            let y_pos = area.y + track_idx as u16;
            
            // Stop if we're out of bounds
            if y_pos >= area.y + area.height {
                break;
            }
            
            let is_selected = focused && 
                             self.sequencer_panel.grid.cursor.track == track_idx as u8 &&
                             track.selected_control == crate::tui::ui::widgets::TrackControl::Volume;
            
            let vol_style = if is_selected {
                Style::default().fg(Color::Yellow).bg(Color::DarkGray)
            } else {
                style
            };
            
            let vol_percent = (track.volume * 100.0) as u8;
            let vol_bars = (track.volume * 10.0) as usize; // 10 blocks for compact display
            let vol_filled = "█".repeat(vol_bars);
            let vol_empty = "░".repeat(10 - vol_bars);
            let vol_display = format!("T{} {}{} {}%", track.track_number, vol_filled, vol_empty, vol_percent);
            
            let paragraph = Paragraph::new(vol_display).style(vol_style);
            let cell_area = Rect { x: area.x, y: y_pos, width: area.width, height: 1 };
            frame.render_widget(paragraph, cell_area);
        }
    }
    
    fn render_panning_controls_only(&mut self, frame: &mut Frame, area: Rect, focused: bool) {
        use ratatui::{
            style::{Color, Style},
            widgets::Paragraph,
        };
        
        let style = if focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::White)
        };
        
        // Render panning controls for each track
        for (track_idx, track) in self.sequencer_panel.grid.tracks.iter().enumerate() {
            let y_pos = area.y + track_idx as u16;
            
            // Stop if we're out of bounds
            if y_pos >= area.y + area.height {
                break;
            }
            
            let is_selected = focused && 
                             self.sequencer_panel.grid.cursor.track == track_idx as u8 &&
                             track.selected_control == crate::tui::ui::widgets::TrackControl::Pan;
            
            let pan_style = if is_selected {
                Style::default().fg(Color::Yellow).bg(Color::DarkGray)
            } else {
                style
            };
            
            let pan_percent = (track.pan * 100.0) as i8;
            let pan_pos = ((track.pan + 1.0) * 5.0) as usize; // 10 positions (0-9) for compact display
            let mut pan_display: Vec<char> = "░".repeat(10).chars().collect();
            
            // Mark center position
            pan_display[5] = '│'; // Center marker (position 5 out of 10)
            if pan_pos < 10 {
                pan_display[pan_pos] = '█'; // Current position
            }
            
            let pan_display: String = pan_display.into_iter().collect();
            let pan_text = format!("T{} L{} R {:+}%", track.track_number, pan_display, pan_percent);
            
            let paragraph = Paragraph::new(pan_text).style(pan_style);
            let cell_area = Rect { x: area.x, y: y_pos, width: area.width, height: 1 };
            frame.render_widget(paragraph, cell_area);
        }
    }
    
    fn render_transport(&self, frame: &mut Frame, area: Rect) {
        let title = match &self.current_focus {
            FocusArea::Transport => "8 - TRANSPORT [FOCUSED]",
            _ => "8 - TRANSPORT",
        };
        
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL);
        
        let inner = block.inner(area);
        frame.render_widget(block, area);
        
        // Create Play and Stop buttons with focus indication
        let focused_transport = matches!(self.current_focus, FocusArea::Transport);
        
        let play_button = if focused_transport && self.transport.focused_button == TransportButton::Play {
            "►[▶]◄"
        } else if self.transport.is_playing {
            "[▶]"
        } else {
            " ▶ "
        };
        
        let stop_button = if focused_transport && self.transport.focused_button == TransportButton::Stop {
            "►[■]◄"
        } else if !self.transport.is_playing {
            "[■]"
        } else {
            " ■ "
        };
        
        let content = format!(
            "{} {}   Tempo: {:.0} BPM   Position: {}.{}.{}",
            play_button,
            stop_button,
            self.transport.tempo,
            self.transport.position.measure,
            self.transport.position.beat,
            self.transport.position.tick
        );
        
        let paragraph = Paragraph::new(content);
        frame.render_widget(paragraph, inner);
    }
    
    fn render_status_bar(&self, frame: &mut Frame, area: Rect) {
        let status_msg = self.ui_state.status_message
            .as_deref()
            .unwrap_or("Ready");
        
        let current_section_info = match &self.current_focus {
            FocusArea::Synthesizer(SynthSection::Oscillator) => {
                match self.synthesizer_panel.current_section {
                    crate::tui::ui::synthesizer::SynthesizerSubSection::Oscillator(crate::tui::ui::synthesizer::OscillatorSubSection::Waveform) => "OSC:Waveform",
                    crate::tui::ui::synthesizer::SynthesizerSubSection::Oscillator(crate::tui::ui::synthesizer::OscillatorSubSection::Volume) => "OSC:Volume",
                    _ => "OSC:Unknown",
                }
            }
            FocusArea::Synthesizer(SynthSection::Filter) => "Filter",
            FocusArea::Synthesizer(SynthSection::Envelope) => "Envelope",
            FocusArea::Synthesizer(SynthSection::Effects) => "Effects",
            FocusArea::Sequencer => "Sequencer",
            FocusArea::TrackVolume => "Track Volume",
            FocusArea::TrackPanning => "Track Panning",
            FocusArea::Transport => "Transport",
        };
        
        let content = format!(
            "{} | {} | 1-8:Sections +/-:Adjust R:Reset F1:Help ESC:Quit",
            status_msg,
            current_section_info
        );
        
        let paragraph = Paragraph::new(content);
        frame.render_widget(paragraph, area);
    }
    
    fn render_help(&self, frame: &mut Frame, area: Rect) {
        let help_text = r#"
ROSCO TUI HELP - Week 2 Enhanced Controls

NAVIGATION:
  Tab        - Cycle focus areas
  Arrow Keys - Navigate within section / adjust parameters
  Enter      - Activate/toggle controls
  ESC        - Quit application

SYNTHESIZER CONTROLS:
  1-8        - Quick switch to Osc/Filter/Env/FX/Grid/Volume/Panning/Transport sections
  Up/Down    - Navigate between controls in section
  Left/Right - Adjust parameter values
  +/-        - Fine adjustment (Freq: ±0.1Hz, Vol: ±1%)
  R          - Reset current parameter to default

OSCILLATOR SECTION:
  Waveform   - Left/Right to change, Enter to expand
  Frequency  - Left/Right: 20 Hz - 20 kHz (logarithmic)
  Volume     - Left/Right: 0% - 100% (linear)

FILTER SECTION:
  Type       - Left/Right to change, Enter to expand
  Cutoff/Center - Left/Right: 20 Hz - 20 kHz (logarithmic)
  Bandwidth  - Left/Right: 10 Hz - 5 kHz (logarithmic, BP/Notch only)
  Resonance  - Left/Right: 0% - 100% (linear)
  Mix        - Left/Right: 0% - 100% (linear)

TRANSPORT (8):
  Left/Right - Navigate between Play ▶ and Stop ■ buttons
  Enter/Space - Activate focused button (►[▶]◄ shows focus)

TRACK GRID (5):
  Tab        - Cycle: Steps → Frequency
  Arrow Keys - Navigate grid (Up/Down: step/frequency rows)
  Enter/Space - Toggle step (Steps) / Open dropdown (Frequency)
  Up/Down    - Select pitch in dropdown mode
  Esc        - Exit dropdown mode
  [C] Normal / ▼C▲ Dropdown - Visual states

TRACK VOLUME (6):
  Up/Down    - Navigate between tracks
  Left/Right - Adjust track volume (±5%)

TRACK PANNING (7):
  Up/Down    - Navigate between tracks
  Left/Right - Adjust track panning (±10%)

REAL-TIME FEATURES:
  • Parameter updates <10ms latency
  • Visual feedback with colored focus indicators
  • Status messages for all parameter changes

PATTERN MANAGEMENT:
  F2         - Save current track as pattern
  F3         - Open/close pattern browser
  Ctrl+P     - Toggle pattern browser (alternative)
  Alt+S      - Quick save pattern
  Alt+L      - Load most recent pattern

COPY/PASTE (in Track Grid):
  Ctrl+C     - Copy current track (or selection if active)
  Ctrl+V     - Paste pattern at cursor position
  Ctrl+X     - Cut current track (copy + clear)
  Ctrl+S     - Start/clear selection
  Ctrl+A     - Select all steps in current track
  Alt+A      - Select all tracks at current step
  Delete     - Clear selected steps (or current step)

GLOBAL:
  F1         - Toggle this help
  ESC        - Close overlays / Quit application
        "#;
        
        let block = Block::default()
            .title("HELP")
            .borders(Borders::ALL);
        
        let inner = block.inner(area);
        frame.render_widget(Clear, area);
        frame.render_widget(block, area);
        
        let paragraph = Paragraph::new(help_text);
        frame.render_widget(paragraph, inner);
    }

    fn render_pattern_browser(&self, frame: &mut Frame, area: Rect) {
        // Create a centered popup area
        let popup_width = 60.min(area.width.saturating_sub(4));
        let popup_height = 20.min(area.height.saturating_sub(4));
        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;
        let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

        // Clear the popup area
        frame.render_widget(Clear, popup_area);

        let block = Block::default()
            .title("PATTERN BROWSER [F3 to close]")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));

        let inner = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        // Get patterns
        let patterns = self.sequencer_panel.get_pattern_manager().list_patterns();
        let current_track = self.sequencer_panel.grid.cursor.track + 1;

        if patterns.is_empty() {
            let text = format!(
                "No patterns saved.\n\nPress F2 to save the current track as a pattern.\n\nCurrent track: {}",
                current_track
            );
            let paragraph = Paragraph::new(text).style(Style::default().fg(Color::Yellow));
            frame.render_widget(paragraph, inner);
        } else {
            // Create header
            let header = format!("Select pattern to load into Track {}\n{}\n",
                current_track,
                "─".repeat((inner.width as usize).saturating_sub(2))
            );

            // Build pattern list
            let mut content = header;
            for (idx, pattern) in patterns.iter().enumerate() {
                let is_selected = idx == self.ui_state.pattern_browser_selection;
                let prefix = if is_selected { "► " } else { "  " };
                let step_count = pattern.steps.iter().filter(|s| s.enabled).count();
                let line = format!(
                    "{}{} ({} steps enabled)\n",
                    prefix,
                    pattern.name,
                    step_count
                );
                content.push_str(&line);
            }

            content.push_str(&format!("\n{}\n", "─".repeat((inner.width as usize).saturating_sub(2))));
            content.push_str("↑↓: Navigate  Enter: Load  Esc: Close  F2: Save New");

            let paragraph = Paragraph::new(content).style(Style::default().fg(Color::White));
            frame.render_widget(paragraph, inner);
        }
    }
}