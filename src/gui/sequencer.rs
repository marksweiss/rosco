use eframe::egui;
use eframe::egui::{pos2, vec2, Color32, Rect};
use serde::{Deserialize, Serialize};

use crate::tui::audio_bridge::ParameterUpdate;

use super::effects::EffectChange;
use super::theme::GuiTheme;

// --- Constants ---

pub const NUM_TRACKS: usize = 8;
pub const NUM_STEPS: usize = 16;

const STEP_SIZE: f32 = 32.0;
const STEP_GAP: f32 = 3.0;
const ROW_GAP: f32 = 3.0;
const HEADER_HEIGHT: f32 = 14.0;
const TRACK_LABEL_WIDTH: f32 = 40.0;
const MIXER_WIDTH: f32 = 220.0;

// --- Step cell ---

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StepCell {
    pub enabled: bool,
    pub velocity: f32, // 0.0–1.0
}

impl Default for StepCell {
    fn default() -> Self {
        Self {
            enabled: false,
            velocity: 0.8,
        }
    }
}

// --- Track strip ---

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrackStrip {
    pub steps: [StepCell; NUM_STEPS],
    pub volume: f32,
    pub pan: f32,
    pub mute: bool,
    pub solo: bool,
}

impl Default for TrackStrip {
    fn default() -> Self {
        Self {
            steps: std::array::from_fn(|_| StepCell::default()),
            volume: 0.8,
            pan: 0.0,
            mute: false,
            solo: false,
        }
    }
}

// --- Sequencer state ---

pub struct SequencerState {
    pub tracks: [TrackStrip; NUM_TRACKS],
    pub playing_step: Option<usize>,
    drag_painting: Option<bool>, // Some(true) = enabling, Some(false) = disabling
}

impl Default for SequencerState {
    fn default() -> Self {
        Self {
            tracks: std::array::from_fn(|_| TrackStrip::default()),
            playing_step: None,
            drag_painting: None,
        }
    }
}

impl SequencerState {
    /// Render the full sequencer panel. Returns parameter changes.
    pub fn render(&mut self, ui: &mut egui::Ui, theme: &GuiTheme) -> Vec<EffectChange> {
        let mut changes = Vec::new();

        ui.horizontal(|ui| {
            // Left: step grid
            ui.vertical(|ui| {
                self.render_grid(ui, &mut changes, theme);
            });

            ui.separator();

            // Right: mixer strip
            ui.vertical(|ui| {
                ui.set_min_width(MIXER_WIDTH);
                self.render_mixer(ui, &mut changes);
            });
        });

        changes
    }

    // --- Step grid ---

    /// Given a pointer position relative to the grid origin, return the (track, step) cell indices
    /// if the pointer is within a valid cell (not in a gap or header/label area).
    fn hit_test_cell(rel: egui::Vec2) -> Option<(usize, usize)> {
        let col_pitch = STEP_SIZE + STEP_GAP;
        let row_pitch = STEP_SIZE + ROW_GAP;
        let local_x = rel.x - TRACK_LABEL_WIDTH - STEP_GAP;
        let local_y = rel.y - HEADER_HEIGHT - ROW_GAP;
        if local_x < 0.0 || local_y < 0.0 {
            return None;
        }
        let step = (local_x / col_pitch) as usize;
        let track = (local_y / row_pitch) as usize;
        if step >= NUM_STEPS || track >= NUM_TRACKS {
            return None;
        }
        // Check we're inside the cell, not in the gap
        let cell_offset_x = local_x - step as f32 * col_pitch;
        let cell_offset_y = local_y - track as f32 * row_pitch;
        if cell_offset_x > STEP_SIZE || cell_offset_y > STEP_SIZE {
            return None;
        }
        Some((track, step))
    }

    fn render_grid(&mut self, ui: &mut egui::Ui, changes: &mut Vec<EffectChange>, theme: &GuiTheme) {
        let text_color = ui.visuals().text_color();

        // Calculate total grid dimensions
        let grid_width = TRACK_LABEL_WIDTH + STEP_GAP
            + NUM_STEPS as f32 * STEP_SIZE
            + (NUM_STEPS as f32 - 1.0) * STEP_GAP;
        let grid_height = HEADER_HEIGHT + ROW_GAP
            + NUM_TRACKS as f32 * STEP_SIZE
            + (NUM_TRACKS as f32 - 1.0) * ROW_GAP;

        // Allocate the full grid area with click_and_drag so the grid claims
        // pointer events instead of the parent scroll area.
        let (full_rect, grid_response) =
            ui.allocate_exact_size(vec2(grid_width, grid_height), egui::Sense::click_and_drag());
        let origin = full_rect.min;
        let painter = ui.painter();

        // Helper: x position for a step column
        let step_x =
            |step: usize| origin.x + TRACK_LABEL_WIDTH + STEP_GAP + step as f32 * (STEP_SIZE + STEP_GAP);
        // Helper: y position for a track row
        let track_y =
            |track: usize| origin.y + HEADER_HEIGHT + ROW_GAP + track as f32 * (STEP_SIZE + ROW_GAP);

        // --- Handle click / drag-paint interaction via the single grid response ---
        let hover_cell: Option<(usize, usize)> = grid_response
            .hover_pos()
            .and_then(|pos| Self::hit_test_cell(pos - origin));

        // Use clicked() for reliable single-cell toggle (fires on pointer release)
        if grid_response.clicked() {
            if let Some(pos) = grid_response.interact_pointer_pos() {
                if let Some((track, step)) = Self::hit_test_cell(pos - origin) {
                    let new_state = !self.tracks[track].steps[step].enabled;
                    self.tracks[track].steps[step].enabled = new_state;
                    changes.push(EffectChange {
                        update: ParameterUpdate::SequencerStep {
                            track: track as u8,
                            step: step as u8,
                            enabled: new_state,
                        },
                        description: format!(
                            "T{}:{} → {}",
                            track + 1,
                            step + 1,
                            if new_state { "on" } else { "off" }
                        ),
                    });
                }
            }
        }

        // Drag-paint: hold and drag across cells to paint them on/off
        if grid_response.drag_started() {
            if let Some(pos) = grid_response.interact_pointer_pos() {
                if let Some((track, step)) = Self::hit_test_cell(pos - origin) {
                    let new_state = !self.tracks[track].steps[step].enabled;
                    self.drag_painting = Some(new_state);
                    self.tracks[track].steps[step].enabled = new_state;
                    changes.push(EffectChange {
                        update: ParameterUpdate::SequencerStep {
                            track: track as u8,
                            step: step as u8,
                            enabled: new_state,
                        },
                        description: format!(
                            "T{}:{} → {}",
                            track + 1,
                            step + 1,
                            if new_state { "on" } else { "off" }
                        ),
                    });
                }
            }
        } else if grid_response.dragged() {
            if let Some(paint_state) = self.drag_painting {
                if let Some(pos) = grid_response.interact_pointer_pos() {
                    if let Some((track, step)) = Self::hit_test_cell(pos - origin) {
                        if self.tracks[track].steps[step].enabled != paint_state {
                            self.tracks[track].steps[step].enabled = paint_state;
                            changes.push(EffectChange {
                                update: ParameterUpdate::SequencerStep {
                                    track: track as u8,
                                    step: step as u8,
                                    enabled: paint_state,
                                },
                                description: format!(
                                    "T{}:{} → {}",
                                    track + 1,
                                    step + 1,
                                    if paint_state { "on" } else { "off" }
                                ),
                            });
                        }
                    }
                }
            }
        }

        if grid_response.drag_stopped() {
            self.drag_painting = None;
        }

        // --- Header row ---
        for step in 0..NUM_STEPS {
            let x = step_x(step);
            let header_rect = Rect::from_min_size(pos2(x, origin.y), vec2(STEP_SIZE, HEADER_HEIGHT));
            if ui.is_rect_visible(header_rect) {
                painter.text(
                    header_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("{}", step + 1),
                    egui::FontId::proportional(10.0),
                    text_color,
                );
            }
        }

        // --- Track rows (draw only) ---
        for track_idx in 0..NUM_TRACKS {
            let row_y = track_y(track_idx);

            // Track label
            let label_rect =
                Rect::from_min_size(pos2(origin.x, row_y), vec2(TRACK_LABEL_WIDTH, STEP_SIZE));
            if ui.is_rect_visible(label_rect) {
                painter.text(
                    label_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("T{}", track_idx + 1),
                    egui::FontId::proportional(14.0),
                    Color32::WHITE,
                );
            }

            // Step cells — drawing only, interaction handled above
            for step_idx in 0..NUM_STEPS {
                let x = step_x(step_idx);
                let rect = Rect::from_min_size(pos2(x, row_y), vec2(STEP_SIZE, STEP_SIZE));

                if ui.is_rect_visible(rect) {
                    let enabled_now = self.tracks[track_idx].steps[step_idx].enabled;
                    let is_playing = self.playing_step == Some(step_idx);
                    let is_hovered = hover_cell == Some((track_idx, step_idx));

                    let step_enabled_c = theme.step_enabled();
                    let bg = if is_playing && enabled_now {
                        theme.step_playing()
                    } else if enabled_now {
                        let v = self.tracks[track_idx].steps[step_idx].velocity;
                        Color32::from_rgb(
                            (step_enabled_c.r() as f32 * v) as u8,
                            (step_enabled_c.g() as f32 * v) as u8,
                            (step_enabled_c.b() as f32 * v) as u8,
                        )
                    } else if is_playing {
                        Color32::from_rgb(60, 60, 30)
                    } else if is_hovered {
                        theme.step_hover()
                    } else {
                        theme.step_disabled()
                    };

                    painter.rect_filled(rect, 3.0, bg);
                }
            }
        }
    }

    // --- Mixer ---

    fn render_mixer(&mut self, ui: &mut egui::Ui, changes: &mut Vec<EffectChange>) {
        egui::Grid::new("mixer_grid")
            .num_columns(4)
            .spacing(vec2(8.0, 4.0))
            .show(ui, |ui| {
                // Header row — use add_sized with same dimensions as controls
                ui.add_sized(vec2(80.0, 18.0), egui::Label::new(egui::RichText::new("Vol").strong()));
                ui.add_sized(vec2(70.0, 18.0), egui::Label::new(egui::RichText::new("Pan").strong()));
                ui.add_sized(vec2(22.0, 18.0), egui::Label::new(egui::RichText::new("M").strong()));
                ui.add_sized(vec2(22.0, 18.0), egui::Label::new(egui::RichText::new("S").strong()));
                ui.end_row();

                // Track rows
                for track_idx in 0..NUM_TRACKS {
                    let track = &mut self.tracks[track_idx];

                    // Volume slider
                    let before_vol = track.volume;
                    ui.add_sized(
                        vec2(80.0, 18.0),
                        egui::Slider::new(&mut track.volume, 0.0..=1.0)
                            .show_value(false),
                    );
                    if (track.volume - before_vol).abs() > 0.001 {
                        changes.push(EffectChange {
                            update: ParameterUpdate::TrackVolume {
                                track: track_idx as u8,
                                volume: track.volume,
                            },
                            description: format!("T{} vol → {:.2}", track_idx + 1, track.volume),
                        });
                    }

                    // Pan slider
                    let before_pan = track.pan;
                    ui.add_sized(
                        vec2(70.0, 18.0),
                        egui::Slider::new(&mut track.pan, -1.0..=1.0)
                            .show_value(false),
                    );
                    if (track.pan - before_pan).abs() > 0.01 {
                        changes.push(EffectChange {
                            update: ParameterUpdate::TrackPan {
                                track: track_idx as u8,
                                pan: track.pan,
                            },
                            description: format!("T{} pan → {:.2}", track_idx + 1, track.pan),
                        });
                    }

                    // Mute toggle
                    let before_mute = track.mute;
                    let mute_text = if track.mute { "M" } else { "m" };
                    let mute_color = if track.mute {
                        Color32::from_rgb(255, 80, 60)
                    } else {
                        Color32::GRAY
                    };
                    if ui
                        .add_sized(vec2(22.0, 18.0), egui::Button::new(
                            egui::RichText::new(mute_text).color(mute_color),
                        ))
                        .clicked()
                    {
                        track.mute = !track.mute;
                    }
                    if track.mute != before_mute {
                        changes.push(EffectChange {
                            update: ParameterUpdate::TrackMute {
                                track: track_idx as u8,
                                muted: track.mute,
                            },
                            description: format!(
                                "T{} {}",
                                track_idx + 1,
                                if track.mute { "muted" } else { "unmuted" }
                            ),
                        });
                    }

                    // Solo toggle
                    let solo_text = if track.solo { "S" } else { "s" };
                    let solo_color = if track.solo {
                        Color32::from_rgb(255, 220, 50)
                    } else {
                        Color32::GRAY
                    };
                    if ui
                        .add_sized(vec2(22.0, 18.0), egui::Button::new(
                            egui::RichText::new(solo_text).color(solo_color),
                        ))
                        .clicked()
                    {
                        track.solo = !track.solo;
                    }
                    // Solo is handled locally (no bridge variant for solo yet)
                    ui.end_row();
                }
            });
    }
}
