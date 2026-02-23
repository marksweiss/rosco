use eframe::egui::Color32;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct GuiTheme {
    pub name: String,

    // Waveform button colors (mod.rs oscillator)
    pub wave_color: [u8; 3],
    pub wave_selected_bg: [u8; 3],
    pub wave_normal_bg: [u8; 3],

    // Envelope colors
    pub envelope_curve_color: [u8; 3],
    pub envelope_point_color: [u8; 3],
    pub envelope_point_drag_color: [u8; 3],
    pub envelope_bg_color: [u8; 3],
    pub envelope_grid_color: [u8; 3],
    pub envelope_fixed_point_color: [u8; 3],

    // Sequencer colors
    pub step_enabled: [u8; 3],
    pub step_disabled: [u8; 3],
    pub step_playing: [u8; 3],
    pub step_hover: [u8; 3],

    // Transport colors
    pub play_color: [u8; 3],
    pub stop_color: [u8; 3],
    pub transport_step_active: [u8; 3],
    pub transport_step_inactive: [u8; 3],
}

impl GuiTheme {
    pub fn dark() -> Self {
        Self {
            name: "Dark".to_string(),

            wave_color: [0, 180, 210],
            wave_selected_bg: [40, 60, 80],
            wave_normal_bg: [35, 35, 40],

            envelope_curve_color: [0, 204, 204],
            envelope_point_color: [255, 255, 255],
            envelope_point_drag_color: [255, 220, 50],
            envelope_bg_color: [30, 30, 35],
            envelope_grid_color: [50, 50, 55],
            envelope_fixed_point_color: [120, 120, 130],

            step_enabled: [0, 230, 100],
            step_disabled: [50, 50, 55],
            step_playing: [255, 255, 60],
            step_hover: [80, 80, 90],

            play_color: [0, 220, 80],
            stop_color: [220, 60, 50],
            transport_step_active: [255, 255, 60],
            transport_step_inactive: [55, 55, 60],
        }
    }

    pub fn light() -> Self {
        Self {
            name: "Light".to_string(),

            wave_color: [0, 120, 160],
            wave_selected_bg: [200, 220, 240],
            wave_normal_bg: [230, 230, 235],

            envelope_curve_color: [0, 140, 140],
            envelope_point_color: [40, 40, 40],
            envelope_point_drag_color: [200, 160, 0],
            envelope_bg_color: [240, 240, 245],
            envelope_grid_color: [200, 200, 210],
            envelope_fixed_point_color: [140, 140, 150],

            step_enabled: [0, 180, 70],
            step_disabled: [210, 210, 215],
            step_playing: [220, 200, 0],
            step_hover: [180, 180, 190],

            play_color: [0, 170, 60],
            stop_color: [200, 50, 40],
            transport_step_active: [220, 200, 0],
            transport_step_inactive: [200, 200, 210],
        }
    }

    // Convenience accessors that return Color32

    pub fn wave_color(&self) -> Color32 {
        rgb(self.wave_color)
    }
    pub fn wave_selected_bg(&self) -> Color32 {
        rgb(self.wave_selected_bg)
    }
    pub fn wave_normal_bg(&self) -> Color32 {
        rgb(self.wave_normal_bg)
    }

    pub fn envelope_curve_color(&self) -> Color32 {
        rgb(self.envelope_curve_color)
    }
    pub fn envelope_point_color(&self) -> Color32 {
        rgb(self.envelope_point_color)
    }
    pub fn envelope_point_drag_color(&self) -> Color32 {
        rgb(self.envelope_point_drag_color)
    }
    pub fn envelope_bg_color(&self) -> Color32 {
        rgb(self.envelope_bg_color)
    }
    pub fn envelope_grid_color(&self) -> Color32 {
        rgb(self.envelope_grid_color)
    }
    pub fn envelope_fixed_point_color(&self) -> Color32 {
        rgb(self.envelope_fixed_point_color)
    }

    pub fn step_enabled(&self) -> Color32 {
        rgb(self.step_enabled)
    }
    pub fn step_disabled(&self) -> Color32 {
        rgb(self.step_disabled)
    }
    pub fn step_playing(&self) -> Color32 {
        rgb(self.step_playing)
    }
    pub fn step_hover(&self) -> Color32 {
        rgb(self.step_hover)
    }

    pub fn play_color(&self) -> Color32 {
        rgb(self.play_color)
    }
    pub fn stop_color(&self) -> Color32 {
        rgb(self.stop_color)
    }
    pub fn transport_step_active(&self) -> Color32 {
        rgb(self.transport_step_active)
    }
    pub fn transport_step_inactive(&self) -> Color32 {
        rgb(self.transport_step_inactive)
    }
}

fn rgb(c: [u8; 3]) -> Color32 {
    Color32::from_rgb(c[0], c[1], c[2])
}
