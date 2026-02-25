use eframe::egui::{self, pos2, vec2, Color32, Pos2, Rect, Stroke};
use serde::{Deserialize, Serialize};

use crate::audio_gen::Waveform;
use crate::tui::audio_bridge::ParameterUpdate;

use super::effects::EffectChange;
use super::theme::GuiTheme;

// --- Constants ---

pub const WAVE_BUTTON_SIZE: egui::Vec2 = vec2(64.0, 48.0);
const WAVE_PREVIEW_SAMPLES: usize = 32;
const MAX_OSCILLATORS_PER_CHAIN: usize = 4;
const NUM_CHAINS: usize = 8;
const CHAIN_SLOT_SIZE: egui::Vec2 = vec2(52.0, 40.0);

// --- Data structures ---

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OscillatorChain {
    pub oscillators: Vec<Waveform>,
    pub active_index: Option<usize>,
}

impl Default for OscillatorChain {
    fn default() -> Self {
        Self {
            oscillators: Vec::new(),
            active_index: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum OscillatorTab {
    Chain(usize),
    Mixer,
}

impl Default for OscillatorTab {
    fn default() -> Self {
        Self::Chain(0)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OscillatorChainsState {
    pub chains: [OscillatorChain; NUM_CHAINS],
    pub mixer_levels: [f32; NUM_CHAINS],
    pub active_tab: OscillatorTab,
    pub volume: f32,
}

impl Default for OscillatorChainsState {
    fn default() -> Self {
        let mut chains: [OscillatorChain; NUM_CHAINS] = Default::default();
        // Chain 0 starts with a single Sine oscillator
        chains[0].oscillators.push(Waveform::Sine);
        chains[0].active_index = Some(0);

        Self {
            chains,
            mixer_levels: [1.0; NUM_CHAINS],
            active_tab: OscillatorTab::default(),
            volume: 0.75,
        }
    }
}

impl OscillatorChainsState {
    /// Insert an oscillator into the current chain after the active cursor.
    /// Returns true if the insertion succeeded.
    pub fn insert_oscillator(&mut self, chain_idx: usize, waveform: Waveform) -> bool {
        let chain = &mut self.chains[chain_idx];
        if chain.oscillators.len() >= MAX_OSCILLATORS_PER_CHAIN {
            return false;
        }
        let insert_pos = match chain.active_index {
            Some(idx) => (idx + 1).min(chain.oscillators.len()),
            None => chain.oscillators.len(),
        };
        chain.oscillators.insert(insert_pos, waveform);
        chain.active_index = Some(insert_pos);
        true
    }

    /// Remove the oscillator at the given index from the chain.
    pub fn remove_oscillator(&mut self, chain_idx: usize, osc_idx: usize) {
        let chain = &mut self.chains[chain_idx];
        if osc_idx < chain.oscillators.len() {
            chain.oscillators.remove(osc_idx);
            if chain.oscillators.is_empty() {
                chain.active_index = None;
            } else {
                chain.active_index = Some(osc_idx.min(chain.oscillators.len() - 1));
            }
        }
    }

    /// Clear all oscillators from a chain.
    pub fn reset_chain(&mut self, chain_idx: usize) {
        self.chains[chain_idx].oscillators.clear();
        self.chains[chain_idx].active_index = None;
    }

    /// Set the active cursor within a chain.
    pub fn set_active(&mut self, chain_idx: usize, osc_idx: usize) {
        let chain = &mut self.chains[chain_idx];
        if osc_idx < chain.oscillators.len() {
            chain.active_index = Some(osc_idx);
        }
    }

    /// Render the full oscillator chains panel. Returns effect changes to send.
    pub fn render(&mut self, ui: &mut egui::Ui, theme: &GuiTheme) -> Vec<EffectChange> {
        let mut changes = Vec::new();

        ui.heading("Oscillator");
        ui.add_space(4.0);

        // --- Tab bar ---
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(2.0, 2.0);
            for i in 0..NUM_CHAINS {
                let selected = self.active_tab == OscillatorTab::Chain(i);
                let non_empty = !self.chains[i].oscillators.is_empty();
                let label = format!("{}", i + 1);
                let text = if non_empty {
                    egui::RichText::new(&label).strong()
                } else {
                    egui::RichText::new(&label)
                };
                if ui.selectable_label(selected, text).clicked() {
                    self.active_tab = OscillatorTab::Chain(i);
                }
            }
            let mixer_selected = self.active_tab == OscillatorTab::Mixer;
            if ui.selectable_label(mixer_selected, "Mix").clicked() {
                self.active_tab = OscillatorTab::Mixer;
            }
        });

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // --- Tab content ---
        match self.active_tab {
            OscillatorTab::Chain(chain_idx) => {
                self.render_chain_view(ui, chain_idx, theme, &mut changes);
            }
            OscillatorTab::Mixer => {
                self.render_mixer_view(ui, theme, &mut changes);
            }
        }

        changes
    }

    fn render_chain_view(
        &mut self,
        ui: &mut egui::Ui,
        chain_idx: usize,
        theme: &GuiTheme,
        changes: &mut Vec<EffectChange>,
    ) {
        // --- Waveform buttons (insert into chain) ---
        ui.label("Waveform");
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
            let waveforms = [
                (Waveform::Sine, "Sine"),
                (Waveform::Saw, "Saw"),
                (Waveform::Square, "Square"),
                (Waveform::Triangle, "Triangle"),
                (Waveform::Noise, "Noise"),
                (Waveform::GaussianNoise, "Gauss"),
            ];
            for (waveform, label) in &waveforms {
                // No button appears "selected" since chains can have multiples
                if waveform_button(ui, *waveform, label, false, theme).clicked() {
                    if self.insert_oscillator(chain_idx, *waveform) {
                        changes.push(EffectChange {
                            update: ParameterUpdate::OscillatorChainUpdate {
                                chain: chain_idx as u8,
                                oscillators: self.chains[chain_idx].oscillators.clone(),
                            },
                            description: format!(
                                "Chain {} add {:?}",
                                chain_idx + 1,
                                waveform
                            ),
                        });
                    }
                }
            }
        });

        ui.add_space(6.0);

        // --- Chain display strip ---
        ui.label("Chain");

        // Snapshot chain info to avoid borrow conflicts
        let chain_len = self.chains[chain_idx].oscillators.len();
        let active_idx = self.chains[chain_idx].active_index;
        let slot_waveforms: Vec<Waveform> = self.chains[chain_idx].oscillators.clone();

        // Track deferred mutations
        let mut clicked_slot: Option<usize> = None;
        let mut removed_slot: Option<usize> = None;
        let mut reset_clicked = false;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(4.0, 0.0);

            for slot in 0..MAX_OSCILLATORS_PER_CHAIN {
                if slot < chain_len {
                    let is_active = active_idx == Some(slot);
                    let waveform = slot_waveforms[slot];
                    let resp = chain_slot_button(ui, waveform, is_active, theme);

                    if resp.clicked() {
                        clicked_slot = Some(slot);
                    }
                    if resp.secondary_clicked() {
                        removed_slot = Some(slot);
                    }
                } else {
                    empty_chain_slot(ui, theme);
                }
            }

            if ui.button("Reset").clicked() {
                reset_clicked = true;
            }
        });

        // Apply deferred mutations
        if let Some(slot) = clicked_slot {
            self.set_active(chain_idx, slot);
        }
        if let Some(slot) = removed_slot {
            self.remove_oscillator(chain_idx, slot);
            changes.push(EffectChange {
                update: ParameterUpdate::OscillatorChainUpdate {
                    chain: chain_idx as u8,
                    oscillators: self.chains[chain_idx].oscillators.clone(),
                },
                description: format!("Chain {} remove osc {}", chain_idx + 1, slot + 1),
            });
        }
        if reset_clicked {
            self.reset_chain(chain_idx);
            changes.push(EffectChange {
                update: ParameterUpdate::OscillatorChainUpdate {
                    chain: chain_idx as u8,
                    oscillators: Vec::new(),
                },
                description: format!("Chain {} reset", chain_idx + 1),
            });
        }

        ui.add_space(6.0);

        // --- Volume slider (global) ---
        ui.label(format!("Volume: {:.2}", self.volume));
        let vol_before = self.volume;
        ui.add(
            egui::Slider::new(&mut self.volume, 0.0..=1.0)
                .clamping(egui::SliderClamping::Always)
                .text(""),
        );
        if (self.volume - vol_before).abs() > 0.001 {
            changes.push(EffectChange {
                update: ParameterUpdate::OscillatorVolume(self.volume),
                description: format!("Volume {:.2}", self.volume),
            });
        }
    }

    fn render_mixer_view(
        &mut self,
        ui: &mut egui::Ui,
        _theme: &GuiTheme,
        changes: &mut Vec<EffectChange>,
    ) {
        ui.label("Mixer — chain levels (effective = level / 8)");
        ui.add_space(4.0);

        // Render 8 sliders in a horizontal layout
        // Use columns for side-by-side vertical sliders
        ui.columns(NUM_CHAINS, |cols| {
            for (i, col) in cols.iter_mut().enumerate() {
                let non_empty = !self.chains[i].oscillators.is_empty();

                // Chain label
                let label = format!("{}", i + 1);
                let text = if non_empty {
                    egui::RichText::new(&label).strong()
                } else {
                    egui::RichText::new(&label).weak()
                };
                col.label(text);

                // Vertical slider
                let level_before = self.mixer_levels[i];
                let slider = egui::Slider::new(&mut self.mixer_levels[i], 0.0..=1.0)
                    .clamping(egui::SliderClamping::Always)
                    .vertical()
                    .text("");

                let enabled = non_empty;
                col.add_enabled(enabled, slider);

                if (self.mixer_levels[i] - level_before).abs() > 0.001 {
                    changes.push(EffectChange {
                        update: ParameterUpdate::OscillatorChainLevel {
                            chain: i as u8,
                            level: self.mixer_levels[i],
                        },
                        description: format!(
                            "Chain {} level {:.0}%",
                            i + 1,
                            self.mixer_levels[i] * 100.0
                        ),
                    });
                }

                // Percentage readout
                col.label(format!("{:.0}%", self.mixer_levels[i] * 100.0));
            }
        });

        ui.add_space(4.0);

        // Global volume slider below mixer
        ui.label(format!("Volume: {:.2}", self.volume));
        let vol_before = self.volume;
        ui.add(
            egui::Slider::new(&mut self.volume, 0.0..=1.0)
                .clamping(egui::SliderClamping::Always)
                .text(""),
        );
        if (self.volume - vol_before).abs() > 0.001 {
            changes.push(EffectChange {
                update: ParameterUpdate::OscillatorVolume(self.volume),
                description: format!("Volume {:.2}", self.volume),
            });
        }
    }
}

// --- Waveform preview button widget ---

pub fn waveform_button(
    ui: &mut egui::Ui,
    waveform: Waveform,
    label: &str,
    selected: bool,
    theme: &GuiTheme,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(WAVE_BUTTON_SIZE, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();

        // Background
        let bg = if selected {
            theme.wave_selected_bg()
        } else {
            theme.wave_normal_bg()
        };
        let stroke_color = if selected {
            Color32::from_rgb(0, 200, 200)
        } else {
            Color32::from_rgb(80, 80, 85)
        };
        painter.rect(rect, 4.0, bg, Stroke::new(1.0, stroke_color));

        // Waveform preview area (top portion)
        let wave_rect = Rect::from_min_max(
            rect.min + vec2(6.0, 4.0),
            pos2(rect.max.x - 6.0, rect.max.y - 16.0),
        );

        // Generate and draw waveform shape
        let points = generate_waveform_points(waveform, wave_rect);
        let wave_stroke = Stroke::new(1.5, theme.wave_color());
        for pair in points.windows(2) {
            painter.line_segment([pair[0], pair[1]], wave_stroke);
        }

        // Label at bottom
        let text_pos = pos2(rect.center().x, rect.max.y - 10.0);
        let text_color = if selected {
            Color32::WHITE
        } else {
            Color32::from_rgb(170, 170, 175)
        };
        painter.text(
            text_pos,
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(10.0),
            text_color,
        );
    }

    response
}

fn generate_waveform_points(waveform: Waveform, rect: Rect) -> Vec<Pos2> {
    let n = WAVE_PREVIEW_SAMPLES;
    let mid_y = rect.center().y;
    let amp = rect.height() * 0.4;

    (0..n)
        .map(|i| {
            let t = i as f32 / (n - 1) as f32;
            let x = rect.min.x + t * rect.width();
            let sample = waveform_sample(waveform, t, i);
            let y = mid_y - sample * amp;
            pos2(x, y)
        })
        .collect()
}

fn waveform_sample(waveform: Waveform, t: f32, index: usize) -> f32 {
    let phase = t * std::f32::consts::TAU;
    match waveform {
        Waveform::Sine => phase.sin(),
        Waveform::Saw => 2.0 * t - 1.0,
        Waveform::Square => {
            if t < 0.5 {
                1.0
            } else {
                -1.0
            }
        }
        Waveform::Triangle => {
            if t < 0.5 {
                4.0 * t - 1.0
            } else {
                3.0 - 4.0 * t
            }
        }
        Waveform::Noise | Waveform::GaussianNoise => {
            let seed = (index as u32).wrapping_mul(2654435761);
            let normalized = (seed % 1000) as f32 / 500.0 - 1.0;
            normalized
        }
    }
}

// --- Chain slot widgets ---

/// A small button showing a waveform in a chain slot. Supports left-click and right-click.
fn chain_slot_button(
    ui: &mut egui::Ui,
    waveform: Waveform,
    is_active: bool,
    theme: &GuiTheme,
) -> egui::Response {
    let (rect, response) =
        ui.allocate_exact_size(CHAIN_SLOT_SIZE, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();

        let bg = theme.chain_slot_bg();
        let border_color = if is_active {
            theme.chain_active_border()
        } else {
            Color32::from_rgb(60, 60, 65)
        };
        let border_width = if is_active { 2.0 } else { 1.0 };
        painter.rect(rect, 3.0, bg, Stroke::new(border_width, border_color));

        // Mini waveform preview
        let wave_rect = Rect::from_min_max(
            rect.min + vec2(4.0, 3.0),
            pos2(rect.max.x - 4.0, rect.max.y - 12.0),
        );
        let points = generate_waveform_points(waveform, wave_rect);
        let wave_stroke = Stroke::new(1.0, theme.wave_color());
        for pair in points.windows(2) {
            painter.line_segment([pair[0], pair[1]], wave_stroke);
        }

        // Short label
        let label = match waveform {
            Waveform::Sine => "Sin",
            Waveform::Saw => "Saw",
            Waveform::Square => "Sqr",
            Waveform::Triangle => "Tri",
            Waveform::Noise => "Noi",
            Waveform::GaussianNoise => "Gau",
        };
        let text_pos = pos2(rect.center().x, rect.max.y - 7.0);
        painter.text(
            text_pos,
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(9.0),
            Color32::from_rgb(170, 170, 175),
        );
    }

    response
}

/// An empty chain slot placeholder.
fn empty_chain_slot(ui: &mut egui::Ui, theme: &GuiTheme) {
    let (rect, _response) = ui.allocate_exact_size(CHAIN_SLOT_SIZE, egui::Sense::hover());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let bg = theme.chain_empty_slot();
        painter.rect(
            rect,
            3.0,
            bg,
            Stroke::new(1.0, Color32::from_rgb(45, 45, 50)),
        );
    }
}
