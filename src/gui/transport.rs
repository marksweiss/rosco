use std::time::Instant;

use eframe::egui;
use eframe::egui::{vec2, Color32};

use crate::tui::audio_bridge::ParameterUpdate;

use super::effects::EffectChange;
use super::sequencer::NUM_STEPS;
use super::theme::GuiTheme;

// --- Transport state ---

pub struct TransportState {
    pub is_playing: bool,
    pub tempo: f32,
    pub current_step: usize,
    pub position: PlaybackPosition,
    last_step_time: Option<Instant>,
}

#[derive(Clone, Debug)]
pub struct PlaybackPosition {
    pub measure: u32,
    pub beat: u8,
    pub tick: u16,
}

impl Default for PlaybackPosition {
    fn default() -> Self {
        Self {
            measure: 1,
            beat: 1,
            tick: 0,
        }
    }
}

impl Default for TransportState {
    fn default() -> Self {
        Self {
            is_playing: false,
            tempo: 120.0,
            current_step: 0,
            position: PlaybackPosition::default(),
            last_step_time: None,
        }
    }
}

impl TransportState {
    /// Render the transport bar. Returns parameter changes.
    pub fn render(&mut self, ui: &mut egui::Ui, theme: &GuiTheme) -> Vec<EffectChange> {
        let mut changes = Vec::new();

        ui.horizontal(|ui| {
            // Play/Stop buttons
            let play_label = if self.is_playing { "\u{23F8}" } else { "\u{25B6}" }; // ⏸ or ▶
            let play_color = if self.is_playing { theme.play_color() } else { Color32::WHITE };
            if ui
                .add(
                    egui::Button::new(egui::RichText::new(play_label).size(18.0).color(play_color))
                        .min_size(vec2(36.0, 28.0)),
                )
                .clicked()
            {
                self.is_playing = !self.is_playing;
                if self.is_playing {
                    self.last_step_time = Some(Instant::now());
                    changes.push(EffectChange {
                        update: ParameterUpdate::TransportPlay,
                        description: "Transport → Play".to_string(),
                    });
                } else {
                    self.last_step_time = None;
                    changes.push(EffectChange {
                        update: ParameterUpdate::TransportStop,
                        description: "Transport → Pause".to_string(),
                    });
                }
            }

            // Stop (rewind)
            if ui
                .add(
                    egui::Button::new(egui::RichText::new("\u{23F9}").size(18.0).color(theme.stop_color()))
                        .min_size(vec2(36.0, 28.0)),
                )
                .clicked()
            {
                self.is_playing = false;
                self.current_step = 0;
                self.last_step_time = None;
                self.position = PlaybackPosition::default();
                changes.push(EffectChange {
                    update: ParameterUpdate::TransportStop,
                    description: "Transport → Stop".to_string(),
                });
            }

            ui.separator();

            // Tempo
            let before_tempo = self.tempo;
            ui.label("BPM:");
            ui.add_sized(
                vec2(150.0, 18.0),
                egui::Slider::new(&mut self.tempo, 20.0..=300.0)
                    .clamping(egui::SliderClamping::Always),
            );
            if (self.tempo - before_tempo).abs() > 0.1 {
                changes.push(EffectChange {
                    update: ParameterUpdate::TempoChange(self.tempo),
                    description: format!("Tempo → {:.0} BPM", self.tempo),
                });
            }

            ui.separator();

            // Position display
            ui.monospace(format!(
                "{:03}:{:02}:{:03}",
                self.position.measure, self.position.beat, self.position.tick
            ));

            ui.separator();

            // Step indicator LEDs
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                for step in 0..NUM_STEPS {
                    let is_current = self.is_playing && self.current_step == step;
                    let color = if is_current { theme.transport_step_active() } else { theme.transport_step_inactive() };
                    let (rect, _) = ui.allocate_exact_size(vec2(8.0, 12.0), egui::Sense::hover());
                    if ui.is_rect_visible(rect) {
                        ui.painter().rect_filled(rect, 2.0, color);
                    }

                    // Beat group gap
                    if step % 4 == 3 && step < NUM_STEPS - 1 {
                        ui.add_space(4.0);
                    }
                }
            });
        });

        changes
    }

    pub fn start_timer(&mut self) {
        self.last_step_time = Some(Instant::now());
    }

    pub fn stop_timer(&mut self) {
        self.last_step_time = None;
    }

    /// Called each frame to advance the sequencer step based on tempo.
    /// Returns true if the step advanced this frame.
    pub fn tick(&mut self) -> bool {
        if !self.is_playing {
            return false;
        }

        let now = Instant::now();
        let step_duration_secs = 60.0 / self.tempo / 4.0; // 16th notes

        if let Some(last) = self.last_step_time {
            if now.duration_since(last).as_secs_f32() >= step_duration_secs {
                self.last_step_time = Some(now);
                self.current_step = (self.current_step + 1) % NUM_STEPS;
                // Update position display
                self.position.beat = (self.current_step / 4 % 4) as u8 + 1;
                if self.current_step == 0 {
                    self.position.measure += 1;
                }
                return true;
            }
        } else {
            self.last_step_time = Some(now);
        }

        false
    }
}
