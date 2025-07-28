use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct LevelMeter {
    pub level: f32,
    pub peak: f32,
    pub peak_hold_time: Duration,
    pub orientation: MeterOrientation,
    pub width: usize,
    pub focused: bool,
}

#[derive(Debug, Clone)]
pub enum MeterOrientation {
    Horizontal,
    Vertical,
}

impl LevelMeter {
    pub fn new(width: usize, orientation: MeterOrientation) -> Self {
        Self {
            level: 0.0,
            peak: 0.0,
            peak_hold_time: Duration::from_millis(500),
            orientation,
            width,
            focused: false,
        }
    }
    
    pub fn update_level(&mut self, level: f32) {
        self.level = level.clamp(0.0, 1.0);
        if self.level > self.peak {
            self.peak = self.level;
        }
    }
    
    pub fn decay_peak(&mut self, _delta_time: Duration) {
        // Simple peak decay
        let decay_rate = 0.95;
        self.peak *= decay_rate;
        if self.peak < self.level {
            self.peak = self.level;
        }
    }
    
    pub fn render_ascii_meter(&self) -> String {
        let filled_chars = (self.level * self.width as f32) as usize;
        let peak_char = (self.peak * self.width as f32) as usize;
        
        let mut meter = String::new();
        for i in 0..self.width {
            if i < filled_chars {
                meter.push('█');
            } else if i == peak_char && peak_char > filled_chars {
                meter.push('▌');
            } else {
                meter.push('░');
            }
        }
        
        let db_level = if self.level > 0.0 {
            20.0 * self.level.log10()
        } else {
            -96.0
        };
        
        format!("{} {:+.1}dB", meter, db_level)
    }
}

impl Widget for LevelMeter {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = if self.focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::White)
        };
        
        let meter_display = self.render_ascii_meter();
        
        match self.orientation {
            MeterOrientation::Horizontal => {
                buf.set_string(area.x, area.y, &meter_display, style);
            }
            MeterOrientation::Vertical => {
                // For vertical meters, we'd need to render character by character
                // This is a simplified horizontal representation for now
                buf.set_string(area.x, area.y, &meter_display, style);
            }
        }
    }
}