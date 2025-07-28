use crate::audio_gen;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

#[derive(Debug, Clone)]
pub struct WaveformSelector {
    pub options: Vec<audio_gen::Waveform>,
    pub selected: usize,
    pub expanded: bool,
    pub focused: bool,
}

impl WaveformSelector {
    pub fn new() -> Self {
        Self {
            options: vec![
                audio_gen::Waveform::Sine,
                audio_gen::Waveform::Square,
                audio_gen::Waveform::Triangle,
                audio_gen::Waveform::Saw,
                audio_gen::Waveform::GaussianNoise,
            ],
            selected: 0,
            expanded: false,
            focused: false,
        }
    }
    
    pub fn selected_waveform(&self) -> audio_gen::Waveform {
        self.options[self.selected]
    }
    
    pub fn next(&mut self) {
        self.selected = (self.selected + 1) % self.options.len();
    }
    
    pub fn previous(&mut self) {
        if self.selected == 0 {
            self.selected = self.options.len() - 1;
        } else {
            self.selected -= 1;
        }
    }
    
    pub fn toggle_expanded(&mut self) {
        self.expanded = !self.expanded;
    }
}

impl Widget for WaveformSelector {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = if self.focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::White)
        };
        
        let current_waveform = format!("{:?}", self.selected_waveform());
        let display = if self.expanded {
            let mut lines = vec![format!("Wave: {} ▼", current_waveform)];
            for (i, waveform) in self.options.iter().enumerate() {
                let marker = if i == self.selected { ">" } else { " " };
                lines.push(format!("{} {:?}", marker, waveform));
            }
            lines.join("\n")
        } else {
            format!("Wave: {} ▼", current_waveform)
        };
        
        let lines: Vec<&str> = display.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if i < area.height as usize {
                buf.set_string(area.x, area.y + i as u16, line, style);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum FilterType {
    LowPass,
    HighPass,
    BandPass,
    Notch,
}

#[derive(Debug, Clone)]
pub struct FilterTypeSelector {
    pub options: Vec<FilterType>,
    pub selected: usize,
    pub expanded: bool,
    pub focused: bool,
}

impl FilterTypeSelector {
    pub fn new() -> Self {
        Self {
            options: vec![
                FilterType::LowPass,
                FilterType::HighPass,
                FilterType::BandPass,
                FilterType::Notch,
            ],
            selected: 0,
            expanded: false,
            focused: false,
        }
    }
    
    pub fn selected_filter(&self) -> &FilterType {
        &self.options[self.selected]
    }
    
    pub fn next(&mut self) {
        self.selected = (self.selected + 1) % self.options.len();
    }
    
    pub fn previous(&mut self) {
        if self.selected == 0 {
            self.selected = self.options.len() - 1;
        } else {
            self.selected -= 1;
        }
    }
    
    pub fn toggle_expanded(&mut self) {
        self.expanded = !self.expanded;
    }
}

impl Widget for FilterTypeSelector {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = if self.focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::White)
        };
        
        let current_filter = match self.selected_filter() {
            FilterType::LowPass => "LowPass",
            FilterType::HighPass => "HighPass", 
            FilterType::BandPass => "BandPass",
            FilterType::Notch => "Notch",
        };
        
        let display = if self.expanded {
            let mut lines = vec![format!("Type: {} ▼", current_filter)];
            for (i, filter_type) in self.options.iter().enumerate() {
                let marker = if i == self.selected { ">" } else { " " };
                let name = match filter_type {
                    FilterType::LowPass => "LowPass",
                    FilterType::HighPass => "HighPass",
                    FilterType::BandPass => "BandPass", 
                    FilterType::Notch => "Notch",
                };
                lines.push(format!("{} {}", marker, name));
            }
            lines.join("\n")
        } else {
            format!("Type: {} ▼", current_filter)
        };
        
        let lines: Vec<&str> = display.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if i < area.height as usize {
                buf.set_string(area.x, area.y + i as u16, line, style);
            }
        }
    }
}