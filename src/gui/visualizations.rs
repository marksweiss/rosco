use eframe::egui;
use eframe::egui::{pos2, vec2, Color32, Rect, Stroke};
use rustfft::{FftPlanner, num_complex::Complex};

use super::sequencer::NUM_TRACKS;
use super::theme::GuiTheme;

// --- Oscilloscope ---

pub struct OscilloscopeState {
    samples: [f32; 512],
}

impl Default for OscilloscopeState {
    fn default() -> Self {
        Self {
            samples: [0.0; 512],
        }
    }
}

impl OscilloscopeState {
    pub fn update(&mut self, data: &[f32; 512]) {
        self.samples = *data;
    }

    pub fn render(&self, ui: &mut egui::Ui, theme: &GuiTheme) {
        let desired_size = vec2(ui.available_width(), 120.0);
        let (rect, _) = ui.allocate_exact_size(desired_size, egui::Sense::hover());

        if !ui.is_rect_visible(rect) {
            return;
        }

        let painter = ui.painter_at(rect);

        // Background
        painter.rect_filled(rect, 4.0, theme.envelope_bg_color());

        // Zero-crossing trigger: find first upward zero-crossing for stable display
        let trigger_idx = self.find_trigger();

        // Draw waveform
        let stroke = Stroke::new(1.5, theme.envelope_curve_color());
        let mid_y = rect.center().y;
        let amp = rect.height() * 0.45;
        let display_samples = 256.min(512 - trigger_idx);

        let points: Vec<_> = (0..display_samples)
            .map(|i| {
                let t = i as f32 / display_samples as f32;
                let x = rect.min.x + t * rect.width();
                let sample = self.samples[trigger_idx + i];
                let y = mid_y - sample.clamp(-1.0, 1.0) * amp;
                pos2(x, y)
            })
            .collect();

        for pair in points.windows(2) {
            painter.line_segment([pair[0], pair[1]], stroke);
        }

        // Center line
        painter.line_segment(
            [pos2(rect.min.x, mid_y), pos2(rect.max.x, mid_y)],
            Stroke::new(0.5, theme.envelope_grid_color()),
        );
    }

    fn find_trigger(&self) -> usize {
        // Find first upward zero-crossing in the first half
        for i in 1..256 {
            if self.samples[i - 1] <= 0.0 && self.samples[i] > 0.0 {
                return i;
            }
        }
        0
    }
}

// --- Spectrum Analyzer ---

pub struct SpectrumState {
    magnitudes: [f32; 256],
    peaks: [f32; 256],
    peak_decay: f32,
}

impl Default for SpectrumState {
    fn default() -> Self {
        Self {
            magnitudes: [0.0; 256],
            peaks: [0.0; 256],
            peak_decay: 0.95,
        }
    }
}

impl SpectrumState {
    pub fn update_from_waveform(&mut self, samples: &[f32; 512]) {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(512);

        let mut buffer: Vec<Complex<f32>> = samples
            .iter()
            .map(|&s| Complex { re: s, im: 0.0 })
            .collect();

        fft.process(&mut buffer);

        // Take first 256 bins (positive frequencies), compute magnitudes
        for i in 0..256 {
            let mag = buffer[i].norm() / 512.0_f32.sqrt();
            self.magnitudes[i] = mag;
            // Peak hold with decay
            if mag > self.peaks[i] {
                self.peaks[i] = mag;
            } else {
                self.peaks[i] *= self.peak_decay;
            }
        }
    }

    pub fn update_from_spectrum(&mut self, data: &[f32; 256]) {
        self.magnitudes = *data;
        for i in 0..256 {
            if data[i] > self.peaks[i] {
                self.peaks[i] = data[i];
            } else {
                self.peaks[i] *= self.peak_decay;
            }
        }
    }

    pub fn render(&self, ui: &mut egui::Ui, theme: &GuiTheme) {
        let desired_size = vec2(ui.available_width(), 120.0);
        let (rect, _) = ui.allocate_exact_size(desired_size, egui::Sense::hover());

        if !ui.is_rect_visible(rect) {
            return;
        }

        let painter = ui.painter_at(rect);

        // Background
        painter.rect_filled(rect, 4.0, theme.envelope_bg_color());

        // Draw spectrum bars with log-frequency scaling
        let num_bars = 64;
        let bar_width = rect.width() / num_bars as f32;

        for bar in 0..num_bars {
            // Log-frequency mapping: map bar index to frequency bin
            let freq_frac = bar as f32 / num_bars as f32;
            let bin_idx = ((freq_frac * freq_frac) * 255.0) as usize; // quadratic for pseudo-log
            let bin_idx = bin_idx.min(255);

            let mag = self.magnitudes[bin_idx];
            let peak = self.peaks[bin_idx];

            // Color based on magnitude: green -> yellow -> red
            let color = magnitude_color(mag);

            let bar_height = mag.clamp(0.0, 1.0) * rect.height();
            let x = rect.min.x + bar as f32 * bar_width;
            let bar_rect = Rect::from_min_size(
                pos2(x, rect.max.y - bar_height),
                vec2(bar_width - 1.0, bar_height),
            );
            painter.rect_filled(bar_rect, 0.0, color);

            // Peak indicator
            let peak_y = rect.max.y - peak.clamp(0.0, 1.0) * rect.height();
            if peak > 0.01 {
                painter.line_segment(
                    [pos2(x, peak_y), pos2(x + bar_width - 1.0, peak_y)],
                    Stroke::new(1.0, Color32::WHITE),
                );
            }
        }
    }
}

fn magnitude_color(mag: f32) -> Color32 {
    let m = mag.clamp(0.0, 1.0);
    if m < 0.5 {
        // Green to yellow
        let t = m * 2.0;
        Color32::from_rgb((t * 255.0) as u8, 230, 0)
    } else {
        // Yellow to red
        let t = (m - 0.5) * 2.0;
        Color32::from_rgb(255, (230.0 * (1.0 - t)) as u8, 0)
    }
}

// --- Level Meters ---

pub struct LevelMeterState {
    pub levels: [f32; NUM_TRACKS],
    pub peaks: [f32; NUM_TRACKS],
}

impl Default for LevelMeterState {
    fn default() -> Self {
        Self {
            levels: [0.0; NUM_TRACKS],
            peaks: [0.0; NUM_TRACKS],
        }
    }
}

impl LevelMeterState {
    pub fn update_track(&mut self, track: usize, level: f32) {
        if track < NUM_TRACKS {
            self.levels[track] = level;
            if level > self.peaks[track] {
                self.peaks[track] = level;
            } else {
                self.peaks[track] *= 0.97; // decay
            }
        }
    }

    pub fn render_track_meter(&self, ui: &mut egui::Ui, track: usize) {
        let desired_size = vec2(8.0, 60.0);
        let (rect, _) = ui.allocate_exact_size(desired_size, egui::Sense::hover());

        if !ui.is_rect_visible(rect) || track >= NUM_TRACKS {
            return;
        }

        let painter = ui.painter_at(rect);

        // Background
        painter.rect_filled(rect, 1.0, Color32::from_rgb(30, 30, 35));

        // Level bar
        let level = self.levels[track].clamp(0.0, 1.0);
        let bar_height = level * rect.height();
        let color = magnitude_color(level);
        let bar_rect = Rect::from_min_size(
            pos2(rect.min.x, rect.max.y - bar_height),
            vec2(rect.width(), bar_height),
        );
        painter.rect_filled(bar_rect, 0.0, color);

        // Peak indicator
        let peak = self.peaks[track].clamp(0.0, 1.0);
        if peak > 0.01 {
            let peak_y = rect.max.y - peak * rect.height();
            painter.line_segment(
                [pos2(rect.min.x, peak_y), pos2(rect.max.x, peak_y)],
                Stroke::new(1.0, Color32::WHITE),
            );
        }
    }
}
