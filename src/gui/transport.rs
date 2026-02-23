use eframe::egui;
use eframe::egui::{vec2, Color32};

use crate::tui::audio_bridge::ParameterUpdate;

use super::effects::EffectChange;
use super::sequencer::NUM_STEPS;

// --- Transport state ---

pub struct TransportState {
    pub is_playing: bool,
    pub tempo: f32,
    pub current_step: usize,
    pub position: PlaybackPosition,
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
        }
    }
}

// Colors
const PLAY_COLOR: Color32 = Color32::from_rgb(0, 220, 80);
const STOP_COLOR: Color32 = Color32::from_rgb(220, 60, 50);
const STEP_ACTIVE: Color32 = Color32::from_rgb(255, 255, 60);
const STEP_INACTIVE: Color32 = Color32::from_rgb(55, 55, 60);

impl TransportState {
    /// Render the transport bar. Returns parameter changes.
    pub fn render(&mut self, ui: &mut egui::Ui) -> Vec<EffectChange> {
        let mut changes = Vec::new();

        ui.horizontal(|ui| {
            // Play/Stop buttons
            let play_label = if self.is_playing { "\u{23F8}" } else { "\u{25B6}" }; // ⏸ or ▶
            let play_color = if self.is_playing { PLAY_COLOR } else { Color32::WHITE };
            if ui
                .add(
                    egui::Button::new(egui::RichText::new(play_label).size(18.0).color(play_color))
                        .min_size(vec2(36.0, 28.0)),
                )
                .clicked()
            {
                self.is_playing = !self.is_playing;
                if self.is_playing {
                    changes.push(EffectChange {
                        update: ParameterUpdate::TransportPlay,
                        description: "Transport → Play".to_string(),
                    });
                } else {
                    changes.push(EffectChange {
                        update: ParameterUpdate::TransportStop,
                        description: "Transport → Pause".to_string(),
                    });
                }
            }

            // Stop (rewind)
            if ui
                .add(
                    egui::Button::new(egui::RichText::new("\u{23F9}").size(18.0).color(STOP_COLOR))
                        .min_size(vec2(36.0, 28.0)),
                )
                .clicked()
            {
                self.is_playing = false;
                self.current_step = 0;
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
                    let color = if is_current { STEP_ACTIVE } else { STEP_INACTIVE };
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

    /// Advance the step (called externally when simulating playback).
    pub fn advance_step(&mut self) {
        if self.is_playing {
            self.current_step = (self.current_step + 1) % NUM_STEPS;
            // Update position display
            let total_steps = self.current_step;
            self.position.beat = (total_steps / 4 % 4) as u8 + 1;
            if total_steps == 0 && self.position.measure > 0 {
                self.position.measure += 1;
            }
        }
    }
}
