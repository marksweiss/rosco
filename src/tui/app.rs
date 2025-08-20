use crate::tui::{TuiError, audio_bridge::AudioBridge, config::TuiConfig, events::EventHandler};
use crate::tui::ui::{SynthesizerPanel, SequencerPanel};
use crate::audio_gen;
use crate::track::Track;
use crate::sequence::FixedTimeNoteSequence;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame, Terminal,
};
use std::io;

#[derive(Debug, Clone, PartialEq)]
pub enum FocusArea {
    Synthesizer(SynthSection),
    Sequencer,
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
    pub status_message: Option<String>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            show_help: false,
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
    
    // Sequencer State
    tracks: Vec<Track<FixedTimeNoteSequence>>,
    
    // Transport State
    transport: TransportState,
    
    // Configuration
    config: TuiConfig,
    
    // Event handling
    event_handler: EventHandler,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SynthParameters {
    pub oscillator_waveform: audio_gen::Waveform,
    pub oscillator_frequency: f32,
    pub oscillator_volume: f32,
}

impl Default for SynthParameters {
    fn default() -> Self {
        Self {
            oscillator_waveform: audio_gen::Waveform::Sine,
            oscillator_frequency: 440.0,
            oscillator_volume: 0.75,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TransportState {
    pub is_playing: bool,
    pub is_recording: bool,
    pub tempo: f32,
    pub position: PlaybackPosition,
}

impl Default for TransportState {
    fn default() -> Self {
        Self {
            is_playing: false,
            is_recording: false,
            tempo: 120.0,
            position: PlaybackPosition::default(),
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
            tracks: Vec::new(),
            transport,
            config,
            event_handler,
        })
    }
    
    pub async fn run(&mut self) -> Result<(), TuiError> {
        // Temporarily disable audio bridge to test if this fixes the string boundary panic
        println!("Skipping audio bridge initialization to test TUI without audio...");
        self.audio_bridge = None;
        
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
            terminal.draw(|f| self.update_ui(f))?;
            
            if self.handle_events().await? {
                break;
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
            KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
            KeyCode::F(1) => self.ui_state.show_help = !self.ui_state.show_help,
            KeyCode::Tab => self.cycle_focus(),
            KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => {
                self.handle_navigation(key)?;
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.handle_activation()?;
            }
            // Quick section switching with number keys
            KeyCode::Char('1') => {
                self.current_focus = FocusArea::Synthesizer(SynthSection::Oscillator);
                self.ui_state.status_message = Some("Oscillator section".to_string());
            }
            KeyCode::Char('2') => {
                self.current_focus = FocusArea::Synthesizer(SynthSection::Filter);
                self.ui_state.status_message = Some("Filter section".to_string());
            }
            KeyCode::Char('3') => {
                self.current_focus = FocusArea::Synthesizer(SynthSection::Envelope);
                self.ui_state.status_message = Some("Envelope section".to_string());
            }
            KeyCode::Char('4') => {
                self.current_focus = FocusArea::Synthesizer(SynthSection::Effects);
                self.ui_state.status_message = Some("Effects section".to_string());
            }
            KeyCode::Char('5') => {
                self.current_focus = FocusArea::Sequencer;
                self.ui_state.status_message = Some("Track Sequencer section".to_string());
            }
            // Fine adjustment with +/- keys
            KeyCode::Char('+') | KeyCode::Char('=') => {
                if let FocusArea::Synthesizer(SynthSection::Oscillator) = &self.current_focus {
                    if let Some(update) = self.synthesizer_panel.handle_fine_adjustment(true) {
                        // Update local state for display
                        match &update {
                            crate::tui::audio_bridge::ParameterUpdate::OscillatorFrequency(freq) => {
                                self.synth_params.oscillator_frequency = *freq;
                                self.ui_state.status_message = Some(format!("Freq increased to {:.1} Hz", freq));
                            }
                            crate::tui::audio_bridge::ParameterUpdate::OscillatorVolume(vol) => {
                                self.synth_params.oscillator_volume = *vol;
                                self.ui_state.status_message = Some(format!("Volume increased to {:.0}%", vol * 100.0));
                            }
                            _ => {}
                        }
                        self.send_parameter_update_real_time(update)?;
                    }
                }
            }
            KeyCode::Char('-') => {
                if let FocusArea::Synthesizer(SynthSection::Oscillator) = &self.current_focus {
                    if let Some(update) = self.synthesizer_panel.handle_fine_adjustment(false) {
                        // Update local state for display
                        match &update {
                            crate::tui::audio_bridge::ParameterUpdate::OscillatorFrequency(freq) => {
                                self.synth_params.oscillator_frequency = *freq;
                                self.ui_state.status_message = Some(format!("Freq decreased to {:.1} Hz", freq));
                            }
                            crate::tui::audio_bridge::ParameterUpdate::OscillatorVolume(vol) => {
                                self.synth_params.oscillator_volume = *vol;
                                self.ui_state.status_message = Some(format!("Volume decreased to {:.0}%", vol * 100.0));
                            }
                            _ => {}
                        }
                        self.send_parameter_update_real_time(update)?;
                    } else {
                        self.ui_state.status_message = Some("No fine adjustment available for current control".to_string());
                    }
                } else {
                    self.ui_state.status_message = Some("Fine adjustment only works in Oscillator section".to_string());
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
            FocusArea::Synthesizer(SynthSection::Oscillator) => FocusArea::Synthesizer(SynthSection::Filter),
            FocusArea::Synthesizer(SynthSection::Filter) => FocusArea::Synthesizer(SynthSection::Envelope),
            FocusArea::Synthesizer(SynthSection::Envelope) => FocusArea::Synthesizer(SynthSection::Effects),
            FocusArea::Synthesizer(SynthSection::Effects) => FocusArea::Sequencer,
            FocusArea::Sequencer => FocusArea::Transport,
            FocusArea::Transport => FocusArea::Synthesizer(SynthSection::Oscillator),
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
            FocusArea::Transport => {
                // TODO: Handle transport navigation
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
                        crate::tui::audio_bridge::ParameterUpdate::OscillatorFrequency(freq) => {
                            self.synth_params.oscillator_frequency = *freq;
                        }
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
            _ => {
                // TODO: Handle other synthesizer sections
            }
        }
        Ok(())
    }
    
    fn handle_activation(&mut self) -> Result<(), TuiError> {
        match &self.current_focus {
            FocusArea::Transport => {
                self.transport.is_playing = !self.transport.is_playing;
                let status_msg = if self.transport.is_playing { "Playing" } else { "Stopped" };
                self.ui_state.status_message = Some(status_msg.to_string());
                
                // Send transport command to audio bridge
                let transport_cmd = if self.transport.is_playing {
                    crate::tui::audio_bridge::ParameterUpdate::TransportPlay
                } else {
                    crate::tui::audio_bridge::ParameterUpdate::TransportStop
                };
                self.send_parameter_update_real_time(transport_cmd)?;
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
    
    fn send_parameter_update(&mut self) -> Result<(), TuiError> {
        if let Some(_bridge) = &mut self.audio_bridge {
            // TODO: Send parameter updates to audio thread
        }
        Ok(())
    }
    
    fn send_parameter_update_real_time(&mut self, update: crate::tui::audio_bridge::ParameterUpdate) -> Result<(), TuiError> {
        if let Some(bridge) = &mut self.audio_bridge {
            bridge.send_parameter_update(update)?;
            self.ui_state.status_message = Some("Parameter updated".to_string());
        }
        Ok(())
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
                }
                SequencerAction::TrackVolumeChanged { track, volume } => {
                    self.ui_state.status_message = Some(format!(
                        "Track {} volume: {:.0}%", 
                        track + 1, 
                        volume * 100.0
                    ));
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
                crate::tui::ui::synthesizer::OscillatorSubSection::Waveform => {
                    self.synthesizer_panel.oscillator.waveform_selector.selected = 0; // Reset to Sine
                    self.synth_params.oscillator_waveform = self.synthesizer_panel.oscillator.waveform_selector.selected_waveform();
                    let update = crate::tui::audio_bridge::ParameterUpdate::OscillatorWaveform(
                        self.synthesizer_panel.oscillator.waveform_selector.selected_waveform()
                    );
                    self.send_parameter_update_real_time(update)?;
                    self.ui_state.status_message = Some("Waveform reset to Sine".to_string());
                }
                crate::tui::ui::synthesizer::OscillatorSubSection::Frequency => {
                    self.synthesizer_panel.oscillator.frequency_slider.set_value(440.0);
                    self.synth_params.oscillator_frequency = 440.0;
                    let update = crate::tui::audio_bridge::ParameterUpdate::OscillatorFrequency(440.0);
                    self.send_parameter_update_real_time(update)?;
                    self.ui_state.status_message = Some("Frequency reset to 440 Hz".to_string());
                }
                crate::tui::ui::synthesizer::OscillatorSubSection::Volume => {
                    self.synthesizer_panel.oscillator.volume_slider.set_value(0.75);
                    self.synth_params.oscillator_volume = 0.75;
                    let update = crate::tui::audio_bridge::ParameterUpdate::OscillatorVolume(0.75);
                    self.send_parameter_update_real_time(update)?;
                    self.ui_state.status_message = Some("Volume reset to 75%".to_string());
                }
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
        self.render_sequencer(frame, chunks[1]);
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
        self.render_placeholder_section(frame, synth_chunks[1], "FILTER");
        self.render_placeholder_section(frame, synth_chunks[2], "ENVELOPE");
        self.render_placeholder_section(frame, synth_chunks[3], "EFFECTS");
    }
    
    fn render_oscillator_section(&self, frame: &mut Frame, area: Rect) {
        let focused = matches!(self.current_focus, FocusArea::Synthesizer(SynthSection::Oscillator));
        let title = if focused { "OSCILLATOR [FOCUSED]" } else { "OSCILLATOR" };
        
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
                Constraint::Length(2), // Frequency  
                Constraint::Length(2), // Volume
            ])
            .split(inner);
        
        // Render waveform control
        let waveform_focused = focused && self.synthesizer_panel.current_section == crate::tui::ui::synthesizer::OscillatorSubSection::Waveform;
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
        
        // Render frequency control
        let freq_focused = focused && self.synthesizer_panel.current_section == crate::tui::ui::synthesizer::OscillatorSubSection::Frequency;
        let freq_style = if freq_focused { 
            Style::default().fg(Color::Cyan) 
        } else { 
            Style::default().fg(Color::White) 
        };
        let freq_slider = &self.synthesizer_panel.oscillator.frequency_slider;
        let freq_text = format!("Freq: {} {:.1} Hz {}", 
            freq_slider.render_bar(),
            freq_slider.value,
            if freq_focused { "◄" } else { "" }
        );
        frame.render_widget(Paragraph::new(freq_text).style(freq_style), chunks[1]);
        
        // Render volume control
        let vol_focused = focused && self.synthesizer_panel.current_section == crate::tui::ui::synthesizer::OscillatorSubSection::Volume;
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
        frame.render_widget(Paragraph::new(vol_text).style(vol_style), chunks[2]);
    }
    
    fn render_placeholder_section(&self, frame: &mut Frame, area: Rect, title: &str) {
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL);
        
        let inner = block.inner(area);
        frame.render_widget(block, area);
        
        let paragraph = Paragraph::new("TODO");
        frame.render_widget(paragraph, inner);
    }
    
    fn render_sequencer(&mut self, frame: &mut Frame, area: Rect) {
        let title = match &self.current_focus {
            FocusArea::Sequencer => "5 - TRACK SEQUENCER [FOCUSED]",
            _ => "5 - TRACK SEQUENCER",
        };
        
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL);
        
        let inner = block.inner(area);
        frame.render_widget(block, area);
        
        // Split sequencer: grid + transport
        let seq_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(inner.height.saturating_sub(4)), // Give most space to grid
                Constraint::Min(3),  // Transport
            ])
            .split(inner);
        
        self.render_sequencer_grid(frame, seq_chunks[0]);
        self.render_transport(frame, seq_chunks[1]);
    }
    
    fn render_sequencer_grid(&mut self, frame: &mut Frame, area: Rect) {
        let focused = matches!(self.current_focus, FocusArea::Sequencer);
        self.sequencer_panel.grid.focused = focused;
        frame.render_widget(self.sequencer_panel.grid.clone(), area);
    }
    
    fn render_transport(&self, frame: &mut Frame, area: Rect) {
        let title = match &self.current_focus {
            FocusArea::Transport => "TRANSPORT [FOCUSED]",
            _ => "TRANSPORT",
        };
        
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL);
        
        let inner = block.inner(area);
        frame.render_widget(block, area);
        
        let play_symbol = if self.transport.is_playing { "■" } else { "▶" };
        let content = format!(
            "{} ● ●   Tempo: {:.0} BPM   Position: {}.{}.{}",
            play_symbol,
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
                    crate::tui::ui::synthesizer::OscillatorSubSection::Waveform => "OSC:Waveform",
                    crate::tui::ui::synthesizer::OscillatorSubSection::Frequency => "OSC:Frequency", 
                    crate::tui::ui::synthesizer::OscillatorSubSection::Volume => "OSC:Volume",
                }
            }
            FocusArea::Synthesizer(SynthSection::Filter) => "Filter",
            FocusArea::Synthesizer(SynthSection::Envelope) => "Envelope",
            FocusArea::Synthesizer(SynthSection::Effects) => "Effects",
            FocusArea::Sequencer => "Sequencer",
            FocusArea::Transport => "Transport",
        };
        
        let content = format!(
            "{} | {} | 1-5:Sections +/-:Adjust R:Reset F1:Help ESC:Quit",
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
  1-5        - Quick switch to Osc/Filter/Env/FX/Sequencer sections
  Up/Down    - Navigate between controls in section
  Left/Right - Adjust parameter values
  +/-        - Fine adjustment (Freq: ±0.1Hz, Vol: ±1%)
  R          - Reset current parameter to default

OSCILLATOR SECTION:
  Waveform   - Left/Right to change, Enter to expand
  Frequency  - Left/Right: 20 Hz - 20 kHz (logarithmic)
  Volume     - Left/Right: 0% - 100% (linear)

TRANSPORT:
  Enter/Space - Play/Stop toggle

SEQUENCER (5):
  Tab        - Cycle: Steps → Track Controls
  Arrow Keys - Navigate grid (Up/Down: step/frequency rows)
  Enter/Space - Toggle step (Steps) / Open dropdown (Frequency)
  Up/Down    - Select pitch in dropdown mode
  Esc        - Exit dropdown mode
  [C] Normal / ▼C▲ Dropdown - Visual states

REAL-TIME FEATURES:
  • Parameter updates <10ms latency
  • Visual feedback with colored focus indicators
  • Status messages for all parameter changes

GLOBAL:
  F1         - Toggle this help
  ESC        - Quit application
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
}