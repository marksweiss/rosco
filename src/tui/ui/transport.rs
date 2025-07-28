use crate::tui::ui::widgets::LinearSlider;

#[derive(Debug)]
pub struct TransportPanel {
    pub play_button: Button,
    pub stop_button: Button,
    pub record_button: Button,
    pub tempo_slider: LinearSlider,
    pub position_display: PositionDisplay,
}

#[derive(Debug)]
pub struct Button {
    pub label: String,
    pub pressed: bool,
    pub focused: bool,
}

#[derive(Debug)]
pub struct PositionDisplay {
    pub measure: u32,
    pub beat: u8,
    pub tick: u16,
    pub format: PositionFormat,
}

#[derive(Debug)]
pub enum PositionFormat {
    MeasureBeatTick,
    TimeMinutesSeconds,
    SamplePosition,
}

impl TransportPanel {
    pub fn new() -> Self {
        Self {
            play_button: Button::new("▶"),
            stop_button: Button::new("■"),
            record_button: Button::new("●"),
            tempo_slider: LinearSlider::new("Tempo", 120.0, 60.0, 200.0, 8),
            position_display: PositionDisplay::new(),
        }
    }
}

impl Button {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            pressed: false,
            focused: false,
        }
    }
}

impl PositionDisplay {
    pub fn new() -> Self {
        Self {
            measure: 1,
            beat: 1,
            tick: 0,
            format: PositionFormat::MeasureBeatTick,
        }
    }
    
    pub fn format_position(&self) -> String {
        match self.format {
            PositionFormat::MeasureBeatTick => {
                format!("{}.{}.{}", self.measure, self.beat, self.tick)
            }
            PositionFormat::TimeMinutesSeconds => {
                // TODO: Convert to time format
                format!("0:00")
            }
            PositionFormat::SamplePosition => {
                // TODO: Convert to sample position
                format!("0")
            }
        }
    }
}