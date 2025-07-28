use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

#[derive(Debug, Clone)]
pub struct LinearSlider {
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub label: String,
    pub width: usize,
    pub focused: bool,
}

impl LinearSlider {
    pub fn new(label: &str, value: f32, min: f32, max: f32, width: usize) -> Self {
        Self {
            value,
            min,
            max,
            label: label.to_string(),
            width,
            focused: false,
        }
    }
    
    pub fn set_value(&mut self, value: f32) {
        self.value = value.clamp(self.min, self.max);
    }
    
    pub fn adjust(&mut self, delta: f32) {
        self.set_value(self.value + delta);
    }
    
    pub fn normalized_value(&self) -> f32 {
        if self.max == self.min {
            0.0
        } else {
            (self.value - self.min) / (self.max - self.min)
        }
    }
    
    pub fn render_bar(&self) -> String {
        let filled_chars = (self.normalized_value() * self.width as f32) as usize;
        let empty_chars = self.width.saturating_sub(filled_chars);
        format!("{}{}", "█".repeat(filled_chars), "░".repeat(empty_chars))
    }
    
    pub fn render_with_value(&self) -> String {
        format!("{}: {} {:.2}", self.label, self.render_bar(), self.value)
    }
}

impl Widget for LinearSlider {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = if self.focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::White)
        };
        
        let content = self.render_with_value();
        let lines: Vec<&str> = content.lines().collect();
        
        for (i, line) in lines.iter().enumerate() {
            if i < area.height as usize {
                buf.set_string(area.x, area.y + i as u16, line, style);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct LogSlider {
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub label: String,
    pub width: usize,
    pub focused: bool,
}

impl LogSlider {
    pub fn new(label: &str, value: f32, min: f32, max: f32, width: usize) -> Self {
        Self {
            value,
            min,
            max,
            label: label.to_string(),
            width,
            focused: false,
        }
    }
    
    pub fn set_value(&mut self, value: f32) {
        self.value = value.clamp(self.min, self.max);
    }
    
    pub fn adjust_log(&mut self, factor: f32) {
        self.set_value(self.value * factor);
    }
    
    pub fn adjust_linear(&mut self, delta: f32) {
        self.set_value(self.value + delta);
    }
    
    pub fn normalized_value(&self) -> f32 {
        if self.max == self.min {
            0.0
        } else {
            (self.value.ln() - self.min.ln()) / (self.max.ln() - self.min.ln())
        }
    }
    
    pub fn render_bar(&self) -> String {
        let filled_chars = (self.normalized_value() * self.width as f32) as usize;
        let empty_chars = self.width.saturating_sub(filled_chars);
        format!("{}{}", "█".repeat(filled_chars), "░".repeat(empty_chars))
    }
    
    pub fn render_with_value(&self) -> String {
        format!("{}: {} {:.1}", self.label, self.render_bar(), self.value)
    }
}

impl Widget for LogSlider {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = if self.focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::White)
        };
        
        let content = self.render_with_value();
        let lines: Vec<&str> = content.lines().collect();
        
        for (i, line) in lines.iter().enumerate() {
            if i < area.height as usize {
                buf.set_string(area.x, area.y + i as u16, line, style);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct TimeSlider {
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub label: String,
    pub width: usize,
    pub focused: bool,
}

impl TimeSlider {
    pub fn new(label: &str, value: f32, min: f32, max: f32, width: usize) -> Self {
        Self {
            value,
            min,
            max,
            label: label.to_string(),
            width,
            focused: false,
        }
    }
    
    pub fn set_value(&mut self, value: f32) {
        self.value = value.clamp(self.min, self.max);
    }
    
    pub fn adjust(&mut self, delta: f32) {
        self.set_value(self.value + delta);
    }
    
    pub fn normalized_value(&self) -> f32 {
        if self.max == self.min {
            0.0
        } else {
            (self.value - self.min) / (self.max - self.min)
        }
    }
    
    pub fn render_bar(&self) -> String {
        let filled_chars = (self.normalized_value() * self.width as f32) as usize;
        let empty_chars = self.width.saturating_sub(filled_chars);
        format!("{}{}", "█".repeat(filled_chars), "░".repeat(empty_chars))
    }
    
    pub fn render_with_value(&self) -> String {
        let unit = if self.value < 1.0 {
            format!("{:.0}ms", self.value * 1000.0)
        } else {
            format!("{:.1}s", self.value)
        };
        format!("{}: {} {}", self.label, self.render_bar(), unit)
    }
}

impl Widget for TimeSlider {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = if self.focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::White)
        };
        
        let content = self.render_with_value();
        let lines: Vec<&str> = content.lines().collect();
        
        for (i, line) in lines.iter().enumerate() {
            if i < area.height as usize {
                buf.set_string(area.x, area.y + i as u16, line, style);
            }
        }
    }
}