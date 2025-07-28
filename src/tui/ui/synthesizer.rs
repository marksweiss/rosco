use crate::tui::ui::widgets::{LinearSlider, LogSlider, WaveformSelector, FilterTypeSelector};
use crate::tui::audio_bridge::ParameterUpdate;
use crate::audio_gen::Waveform;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Widget},
};

#[derive(Debug)]
pub struct SynthesizerPanel {
    pub oscillator: OscillatorControls,
    pub filter: FilterControls,
    pub envelope: EnvelopeControls,
    pub effects: EffectsControls,
    pub current_section: OscillatorSubSection,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OscillatorSubSection {
    Waveform,
    Frequency,
    Volume,
}

#[derive(Debug)]
pub struct OscillatorControls {
    pub waveform_selector: WaveformSelector,
    pub frequency_slider: LogSlider,
    pub volume_slider: LinearSlider,
    pub sub_focus: OscillatorSubSection,
}

#[derive(Debug)]
pub struct FilterControls {
    pub filter_type: FilterTypeSelector,
    pub cutoff_slider: LogSlider,
    pub resonance_slider: LinearSlider,
    pub mix_slider: LinearSlider,
}

#[derive(Debug)]
pub struct EnvelopeControls {
    // TODO: Implement envelope controls
}

#[derive(Debug)]
pub struct EffectsControls {
    // TODO: Implement effects controls
}

impl SynthesizerPanel {
    pub fn new() -> Self {
        Self {
            oscillator: OscillatorControls::new(),
            filter: FilterControls::new(),
            envelope: EnvelopeControls {},
            effects: EffectsControls {},
            current_section: OscillatorSubSection::Waveform,
        }
    }
    
    pub fn handle_input(&mut self, key: KeyEvent) -> Vec<ParameterUpdate> {
        let mut updates = Vec::new();
        
        match key.code {
            KeyCode::Up | KeyCode::Down => {
                self.current_section = match self.current_section {
                    OscillatorSubSection::Waveform => OscillatorSubSection::Frequency,
                    OscillatorSubSection::Frequency => OscillatorSubSection::Volume,
                    OscillatorSubSection::Volume => OscillatorSubSection::Waveform,
                };
            }
            KeyCode::Left | KeyCode::Right => {
                if let Some(update) = self.handle_parameter_adjustment(key.code) {
                    updates.push(update);
                }
            }
            KeyCode::Enter => {
                if let Some(update) = self.handle_activation() {
                    updates.push(update);
                }
            }
            _ => {}
        }
        
        updates
    }
    
    pub fn handle_fine_adjustment(&mut self, increase: bool) -> Option<ParameterUpdate> {
        match self.current_section {
            OscillatorSubSection::Frequency => {
                let delta = if increase { 0.1 } else { -0.1 };
                self.oscillator.frequency_slider.adjust_linear(delta);
                Some(ParameterUpdate::OscillatorFrequency(
                    self.oscillator.frequency_slider.value
                ))
            }
            OscillatorSubSection::Volume => {
                let delta = if increase { 0.01 } else { -0.01 };
                self.oscillator.volume_slider.adjust(delta);
                Some(ParameterUpdate::OscillatorVolume(
                    self.oscillator.volume_slider.value
                ))
            }
            _ => None
        }
    }
    
    fn handle_parameter_adjustment(&mut self, key_code: KeyCode) -> Option<ParameterUpdate> {
        match self.current_section {
            OscillatorSubSection::Waveform => {
                match key_code {
                    KeyCode::Left => self.oscillator.waveform_selector.previous(),
                    KeyCode::Right => self.oscillator.waveform_selector.next(),
                    _ => {}
                }
                Some(ParameterUpdate::OscillatorWaveform(
                    self.oscillator.waveform_selector.selected_waveform()
                ))
            }
            OscillatorSubSection::Frequency => {
                match key_code {
                    KeyCode::Left => {
                        self.oscillator.frequency_slider.adjust_log(0.95);
                        Some(ParameterUpdate::OscillatorFrequency(
                            self.oscillator.frequency_slider.value
                        ))
                    }
                    KeyCode::Right => {
                        self.oscillator.frequency_slider.adjust_log(1.05);
                        Some(ParameterUpdate::OscillatorFrequency(
                            self.oscillator.frequency_slider.value
                        ))
                    }
                    _ => None
                }
            }
            OscillatorSubSection::Volume => {
                match key_code {
                    KeyCode::Left => {
                        self.oscillator.volume_slider.adjust(-0.05);
                        Some(ParameterUpdate::OscillatorVolume(
                            self.oscillator.volume_slider.value
                        ))
                    }
                    KeyCode::Right => {
                        self.oscillator.volume_slider.adjust(0.05);
                        Some(ParameterUpdate::OscillatorVolume(
                            self.oscillator.volume_slider.value
                        ))
                    }
                    _ => None
                }
            }
        }
    }
    
    fn handle_activation(&mut self) -> Option<ParameterUpdate> {
        match self.current_section {
            OscillatorSubSection::Waveform => {
                self.oscillator.waveform_selector.toggle_expanded();
                Some(ParameterUpdate::OscillatorWaveform(
                    self.oscillator.waveform_selector.selected_waveform()
                ))
            }
            _ => None
        }
    }
    
    pub fn get_waveform(&self) -> Waveform {
        self.oscillator.waveform_selector.selected_waveform()
    }
    
    pub fn get_frequency(&self) -> f32 {
        self.oscillator.frequency_slider.value
    }
    
    pub fn get_volume(&self) -> f32 {
        self.oscillator.volume_slider.value
    }
}

impl OscillatorControls {
    pub fn new() -> Self {
        Self {
            waveform_selector: WaveformSelector::new(),
            frequency_slider: LogSlider::new("Freq", 440.0, 20.0, 20000.0, 10),
            volume_slider: LinearSlider::new("Vol", 0.75, 0.0, 1.0, 10),
            sub_focus: OscillatorSubSection::Waveform,
        }
    }
    
    pub fn render(&self, area: Rect, buf: &mut Buffer, focused: bool, current_section: OscillatorSubSection) {
        let title = if focused { "OSCILLATOR [FOCUSED]" } else { "OSCILLATOR" };
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL);
        
        let inner = block.inner(area);
        block.render(area, buf);
        
        // Split oscillator area vertically
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Waveform
                Constraint::Length(2), // Frequency  
                Constraint::Length(2), // Volume
            ])
            .split(inner);
        
        // Render waveform selector
        let mut waveform_selector = self.waveform_selector.clone();
        waveform_selector.focused = focused && current_section == OscillatorSubSection::Waveform;
        waveform_selector.render(chunks[0], buf);
        
        // Render frequency slider
        let mut freq_slider = self.frequency_slider.clone();
        freq_slider.focused = focused && current_section == OscillatorSubSection::Frequency;
        freq_slider.render(chunks[1], buf);
        
        // Render volume slider
        let mut vol_slider = self.volume_slider.clone();
        vol_slider.focused = focused && current_section == OscillatorSubSection::Volume;
        vol_slider.render(chunks[2], buf);
    }
}

impl FilterControls {
    pub fn new() -> Self {
        Self {
            filter_type: FilterTypeSelector::new(),
            cutoff_slider: LogSlider::new("Cutoff", 8000.0, 20.0, 20000.0, 8),
            resonance_slider: LinearSlider::new("Res", 0.3, 0.0, 1.0, 8),
            mix_slider: LinearSlider::new("Mix", 0.8, 0.0, 1.0, 8),
        }
    }
}