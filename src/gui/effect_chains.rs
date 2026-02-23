use eframe::egui::{self, pos2, vec2, Color32, Pos2, Rect, Stroke};
use serde::{Deserialize, Serialize};

use crate::tui::audio_bridge::ParameterUpdate;

use super::effects::{
    render_chorus_params, render_delay_params, render_filter_params, render_flanger_params,
    render_lfo_params, render_tremolo_params, render_vibrato_params, ChorusState, DelayState,
    EffectChange, FilterState, FlangerState, LfoState, TremoloState, VibratoState,
};
use super::theme::GuiTheme;

// --- Constants ---

const MAX_EFFECTS_PER_CHAIN: usize = 4;
const NUM_CHAINS: usize = 8;
const EFFECT_BUTTON_SIZE: egui::Vec2 = vec2(64.0, 48.0);
const EFFECT_SLOT_SIZE: egui::Vec2 = vec2(52.0, 40.0);

// --- Effect type enum ---

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectType {
    Lfo,
    Tremolo,
    Vibrato,
    Flanger,
    Delay,
    Chorus,
    Filter,
}

impl EffectType {
    pub const ALL: [EffectType; 7] = [
        Self::Lfo,
        Self::Tremolo,
        Self::Vibrato,
        Self::Flanger,
        Self::Delay,
        Self::Chorus,
        Self::Filter,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Lfo => "LFO",
            Self::Tremolo => "Trem",
            Self::Vibrato => "Vib",
            Self::Flanger => "Flng",
            Self::Delay => "Dly",
            Self::Chorus => "Chor",
            Self::Filter => "Filt",
        }
    }

    pub fn full_label(&self) -> &'static str {
        match self {
            Self::Lfo => "LFO",
            Self::Tremolo => "Tremolo",
            Self::Vibrato => "Vibrato",
            Self::Flanger => "Flanger",
            Self::Delay => "Delay",
            Self::Chorus => "Chorus",
            Self::Filter => "Filter",
        }
    }
}

// --- Effect instance enum ---

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EffectInstance {
    Lfo(LfoState),
    Tremolo(TremoloState),
    Vibrato(VibratoState),
    Flanger(FlangerState),
    Delay(DelayState),
    Chorus(ChorusState),
    Filter(FilterState),
}

impl EffectInstance {
    pub fn from_type(et: EffectType) -> Self {
        match et {
            EffectType::Lfo => Self::Lfo(LfoState { enabled: true, ..LfoState::default() }),
            EffectType::Tremolo => Self::Tremolo(TremoloState { enabled: true, ..TremoloState::default() }),
            EffectType::Vibrato => Self::Vibrato(VibratoState { enabled: true, ..VibratoState::default() }),
            EffectType::Flanger => Self::Flanger(FlangerState { enabled: true, ..FlangerState::default() }),
            EffectType::Delay => Self::Delay(DelayState { enabled: true, ..DelayState::default() }),
            EffectType::Chorus => Self::Chorus(ChorusState { enabled: true, ..ChorusState::default() }),
            EffectType::Filter => Self::Filter(FilterState { enabled: true, ..FilterState::default() }),
        }
    }

    pub fn effect_type(&self) -> EffectType {
        match self {
            Self::Lfo(_) => EffectType::Lfo,
            Self::Tremolo(_) => EffectType::Tremolo,
            Self::Vibrato(_) => EffectType::Vibrato,
            Self::Flanger(_) => EffectType::Flanger,
            Self::Delay(_) => EffectType::Delay,
            Self::Chorus(_) => EffectType::Chorus,
            Self::Filter(_) => EffectType::Filter,
        }
    }

    pub fn short_label(&self) -> &'static str {
        self.effect_type().label()
    }
}

// --- Effect chain ---

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EffectChain {
    pub effects: Vec<EffectInstance>,
    pub active_index: Option<usize>,
}

impl Default for EffectChain {
    fn default() -> Self {
        Self {
            effects: Vec::new(),
            active_index: None,
        }
    }
}

// --- Tab enum ---

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum EffectChainsTab {
    Chain(usize),
    Mixer,
}

impl Default for EffectChainsTab {
    fn default() -> Self {
        Self::Chain(0)
    }
}

// --- Top-level state ---

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct EffectChainsState {
    pub chains: [EffectChain; NUM_CHAINS],
    pub dry_wet: [f32; NUM_CHAINS],
    pub active_tab: EffectChainsTab,
}

impl Default for EffectChainsState {
    fn default() -> Self {
        Self {
            chains: Default::default(),
            dry_wet: [0.5; NUM_CHAINS],
            active_tab: EffectChainsTab::default(),
        }
    }
}

impl EffectChainsState {
    pub fn insert_effect(&mut self, chain_idx: usize, et: EffectType) -> bool {
        let chain = &mut self.chains[chain_idx];
        if chain.effects.len() >= MAX_EFFECTS_PER_CHAIN {
            return false;
        }
        let insert_pos = match chain.active_index {
            Some(idx) => (idx + 1).min(chain.effects.len()),
            None => chain.effects.len(),
        };
        chain.effects.insert(insert_pos, EffectInstance::from_type(et));
        chain.active_index = Some(insert_pos);
        true
    }

    pub fn remove_effect(&mut self, chain_idx: usize, effect_idx: usize) {
        let chain = &mut self.chains[chain_idx];
        if effect_idx < chain.effects.len() {
            chain.effects.remove(effect_idx);
            if chain.effects.is_empty() {
                chain.active_index = None;
            } else {
                chain.active_index = Some(effect_idx.min(chain.effects.len() - 1));
            }
        }
    }

    pub fn reset_chain(&mut self, chain_idx: usize) {
        self.chains[chain_idx].effects.clear();
        self.chains[chain_idx].active_index = None;
    }

    pub fn set_active(&mut self, chain_idx: usize, effect_idx: usize) {
        let chain = &mut self.chains[chain_idx];
        if effect_idx < chain.effects.len() {
            chain.active_index = Some(effect_idx);
        }
    }

    fn emit_chain_update(chain_idx: usize, chain: &EffectChain) -> EffectChange {
        let json = serde_json::to_string(&chain.effects).unwrap_or_default();
        EffectChange {
            update: ParameterUpdate::EffectChainUpdate {
                chain: chain_idx as u8,
                effects_json: json,
            },
            description: format!("Effect chain {} updated", chain_idx + 1),
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui, theme: &GuiTheme) -> Vec<EffectChange> {
        let mut changes = Vec::new();

        ui.heading("Effects");
        ui.add_space(4.0);

        // --- Tab bar ---
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(2.0, 2.0);
            for i in 0..NUM_CHAINS {
                let selected = self.active_tab == EffectChainsTab::Chain(i);
                let non_empty = !self.chains[i].effects.is_empty();
                let label = format!("{}", i + 1);
                let text = if non_empty {
                    egui::RichText::new(&label).strong()
                } else {
                    egui::RichText::new(&label)
                };
                if ui.selectable_label(selected, text).clicked() {
                    self.active_tab = EffectChainsTab::Chain(i);
                }
            }
            let mixer_selected = self.active_tab == EffectChainsTab::Mixer;
            if ui.selectable_label(mixer_selected, "Mix").clicked() {
                self.active_tab = EffectChainsTab::Mixer;
            }
        });

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        match self.active_tab {
            EffectChainsTab::Chain(chain_idx) => {
                self.render_chain_view(ui, chain_idx, theme, &mut changes);
            }
            EffectChainsTab::Mixer => {
                self.render_mixer_view(ui, &mut changes);
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
        // --- Effect type buttons ---
        ui.label("Add Effect");
        let full = self.chains[chain_idx].effects.len() >= MAX_EFFECTS_PER_CHAIN;
        ui.scope(|ui| {
            if full { ui.disable(); }
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
                for et in EffectType::ALL {
                    if effect_type_button(ui, et, theme).clicked() {
                        if self.insert_effect(chain_idx, et) {
                            changes.push(Self::emit_chain_update(chain_idx, &self.chains[chain_idx]));
                        }
                    }
                }
            });
        });

        ui.add_space(6.0);

        // --- Chain display strip ---
        ui.label("Chain");

        let chain_len = self.chains[chain_idx].effects.len();
        let active_idx = self.chains[chain_idx].active_index;
        let slot_types: Vec<EffectType> = self.chains[chain_idx]
            .effects.iter().map(|e| e.effect_type()).collect();

        let mut clicked_slot: Option<usize> = None;
        let mut removed_slot: Option<usize> = None;
        let mut reset_clicked = false;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(4.0, 0.0);

            for slot in 0..MAX_EFFECTS_PER_CHAIN {
                if slot < chain_len {
                    let is_active = active_idx == Some(slot);
                    let resp = effect_chain_slot(ui, slot_types[slot], is_active, theme);
                    if resp.clicked() {
                        clicked_slot = Some(slot);
                    }
                    if resp.secondary_clicked() {
                        removed_slot = Some(slot);
                    }
                } else {
                    empty_effect_chain_slot(ui, theme);
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
            self.remove_effect(chain_idx, slot);
            changes.push(Self::emit_chain_update(chain_idx, &self.chains[chain_idx]));
        }
        if reset_clicked {
            self.reset_chain(chain_idx);
            changes.push(Self::emit_chain_update(chain_idx, &self.chains[chain_idx]));
        }

        ui.add_space(6.0);
        ui.separator();
        ui.add_space(4.0);

        // --- Parameter controls for selected effect ---
        Self::render_effect_params(&mut self.chains[chain_idx], ui, chain_idx, changes);
    }

    fn render_effect_params(
        chain: &mut EffectChain,
        ui: &mut egui::Ui,
        chain_idx: usize,
        changes: &mut Vec<EffectChange>,
    ) {
        if let Some(idx) = chain.active_index {
            if idx < chain.effects.len() {
                let et = chain.effects[idx].effect_type();
                ui.strong(format!("{} Settings", et.full_label()));
                ui.add_space(4.0);

                let param_changes = match &mut chain.effects[idx] {
                    EffectInstance::Lfo(s) => render_lfo_params(s, ui),
                    EffectInstance::Tremolo(s) => render_tremolo_params(s, ui),
                    EffectInstance::Vibrato(s) => render_vibrato_params(s, ui),
                    EffectInstance::Flanger(s) => render_flanger_params(s, ui),
                    EffectInstance::Delay(s) => render_delay_params(s, ui),
                    EffectInstance::Chorus(s) => render_chorus_params(s, ui),
                    EffectInstance::Filter(s) => render_filter_params(s, ui),
                };

                // If any parameter changed, emit a full chain update instead of individual updates
                if !param_changes.is_empty() {
                    changes.push(Self::emit_chain_update(chain_idx, chain));
                }
                return;
            }
        }
        ui.weak("No effect selected");
    }

    fn render_mixer_view(
        &mut self,
        ui: &mut egui::Ui,
        changes: &mut Vec<EffectChange>,
    ) {
        ui.label("Mixer — per-chain dry/wet");
        ui.add_space(4.0);

        ui.columns(NUM_CHAINS, |cols| {
            for (i, col) in cols.iter_mut().enumerate() {
                let non_empty = !self.chains[i].effects.is_empty();

                let label = format!("{}", i + 1);
                let text = if non_empty {
                    egui::RichText::new(&label).strong()
                } else {
                    egui::RichText::new(&label).weak()
                };
                col.label(text);

                let before = self.dry_wet[i];
                let slider = egui::Slider::new(&mut self.dry_wet[i], 0.0..=1.0)
                    .clamping(egui::SliderClamping::Always)
                    .vertical()
                    .text("");

                col.add_enabled(non_empty, slider);

                if (self.dry_wet[i] - before).abs() > 0.001 {
                    let dry_pct = ((1.0 - self.dry_wet[i]) * 100.0) as u32;
                    let wet_pct = (self.dry_wet[i] * 100.0) as u32;
                    changes.push(EffectChange {
                        update: ParameterUpdate::EffectChainDryWet {
                            chain: i as u8,
                            dry_wet: self.dry_wet[i],
                        },
                        description: format!(
                            "Chain {} D {}% / W {}%",
                            i + 1, dry_pct, wet_pct
                        ),
                    });
                }

                let dry_pct = ((1.0 - self.dry_wet[i]) * 100.0) as u32;
                let wet_pct = (self.dry_wet[i] * 100.0) as u32;
                col.label(format!("D {}%\nW {}%", dry_pct, wet_pct));
            }
        });
    }
}

// --- Effect type button widget ---

fn effect_type_button(
    ui: &mut egui::Ui,
    et: EffectType,
    theme: &GuiTheme,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(EFFECT_BUTTON_SIZE, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();

        let bg = theme.wave_normal_bg();
        let stroke_color = if response.hovered() {
            Color32::from_rgb(0, 200, 200)
        } else {
            Color32::from_rgb(80, 80, 85)
        };
        painter.rect(rect, 4.0, bg, Stroke::new(1.0, stroke_color));

        // Icon area (upper portion)
        let icon_rect = Rect::from_min_max(
            rect.min + vec2(6.0, 4.0),
            pos2(rect.max.x - 6.0, rect.max.y - 16.0),
        );
        draw_effect_icon(painter, et, icon_rect, theme);

        // Label at bottom
        let text_pos = pos2(rect.center().x, rect.max.y - 10.0);
        let text_color = Color32::from_rgb(170, 170, 175);
        painter.text(
            text_pos,
            egui::Align2::CENTER_CENTER,
            et.label(),
            egui::FontId::proportional(10.0),
            text_color,
        );
    }

    response
}

// --- Effect chain slot widget ---

fn effect_chain_slot(
    ui: &mut egui::Ui,
    et: EffectType,
    is_active: bool,
    theme: &GuiTheme,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(EFFECT_SLOT_SIZE, egui::Sense::click());

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

        // Mini icon
        let icon_rect = Rect::from_min_max(
            rect.min + vec2(4.0, 3.0),
            pos2(rect.max.x - 4.0, rect.max.y - 12.0),
        );
        draw_effect_icon(painter, et, icon_rect, theme);

        // Short label
        let text_pos = pos2(rect.center().x, rect.max.y - 7.0);
        painter.text(
            text_pos,
            egui::Align2::CENTER_CENTER,
            et.label(),
            egui::FontId::proportional(9.0),
            Color32::from_rgb(170, 170, 175),
        );
    }

    response
}

fn empty_effect_chain_slot(ui: &mut egui::Ui, theme: &GuiTheme) {
    let (rect, _response) = ui.allocate_exact_size(EFFECT_SLOT_SIZE, egui::Sense::hover());

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

// --- Effect icon drawing ---

fn draw_effect_icon(painter: &egui::Painter, et: EffectType, rect: Rect, theme: &GuiTheme) {
    let color = theme.wave_color();
    let stroke = Stroke::new(1.5, color);
    let mid_y = rect.center().y;
    let amp = rect.height() * 0.4;
    let w = rect.width();

    match et {
        EffectType::Lfo => {
            // Sine wave, 3 cycles
            let points: Vec<Pos2> = (0..32)
                .map(|i| {
                    let t = i as f32 / 31.0;
                    let x = rect.min.x + t * w;
                    let y = mid_y - (t * 3.0 * std::f32::consts::TAU).sin() * amp;
                    pos2(x, y)
                })
                .collect();
            for pair in points.windows(2) {
                painter.line_segment([pair[0], pair[1]], stroke);
            }
        }
        EffectType::Tremolo => {
            // Amplitude envelope wave
            let points: Vec<Pos2> = (0..32)
                .map(|i| {
                    let t = i as f32 / 31.0;
                    let x = rect.min.x + t * w;
                    let envelope = 0.3 + 0.7 * (t * 2.0 * std::f32::consts::TAU).sin().abs();
                    let y = mid_y - (t * 6.0 * std::f32::consts::TAU).sin() * amp * envelope;
                    pos2(x, y)
                })
                .collect();
            for pair in points.windows(2) {
                painter.line_segment([pair[0], pair[1]], stroke);
            }
        }
        EffectType::Vibrato => {
            // Frequency-modulated wave
            let points: Vec<Pos2> = (0..32)
                .map(|i| {
                    let t = i as f32 / 31.0;
                    let x = rect.min.x + t * w;
                    let phase = t * 4.0 * std::f32::consts::TAU
                        + 0.5 * (t * std::f32::consts::TAU).sin();
                    let y = mid_y - phase.sin() * amp;
                    pos2(x, y)
                })
                .collect();
            for pair in points.windows(2) {
                painter.line_segment([pair[0], pair[1]], stroke);
            }
        }
        EffectType::Flanger => {
            // Two offset waves
            let dim_stroke = Stroke::new(1.0, Color32::from_rgb(color.r() / 2, color.g() / 2, color.b() / 2));
            for (s, offset) in [(stroke, 0.0f32), (dim_stroke, 0.3)] {
                let points: Vec<Pos2> = (0..32)
                    .map(|i| {
                        let t = i as f32 / 31.0;
                        let x = rect.min.x + t * w;
                        let y = mid_y - ((t + offset) * 2.0 * std::f32::consts::TAU).sin() * amp;
                        pos2(x, y)
                    })
                    .collect();
                for pair in points.windows(2) {
                    painter.line_segment([pair[0], pair[1]], s);
                }
            }
        }
        EffectType::Delay => {
            // Echo dots (decreasing size)
            for i in 0..5 {
                let t = (i as f32 + 0.5) / 5.0;
                let x = rect.min.x + t * w;
                let radius = 3.0 - i as f32 * 0.5;
                let alpha = 255 - i as u8 * 45;
                let dot_color = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha);
                painter.circle_filled(pos2(x, mid_y), radius.max(1.0), dot_color);
            }
        }
        EffectType::Chorus => {
            // Three parallel wavy lines
            for offset in [-1.0f32, 0.0, 1.0] {
                let y_offset = offset * amp * 0.6;
                let points: Vec<Pos2> = (0..32)
                    .map(|i| {
                        let t = i as f32 / 31.0;
                        let x = rect.min.x + t * w;
                        let y = mid_y + y_offset - (t * 2.0 * std::f32::consts::TAU).sin() * amp * 0.3;
                        pos2(x, y)
                    })
                    .collect();
                for pair in points.windows(2) {
                    painter.line_segment([pair[0], pair[1]], stroke);
                }
            }
        }
        EffectType::Filter => {
            // Diagonal cutoff line with dimmed right side
            let cutoff_x = rect.min.x + w * 0.4;
            // Flat line left of cutoff
            painter.line_segment(
                [pos2(rect.min.x, mid_y - amp * 0.5), pos2(cutoff_x, mid_y - amp * 0.5)],
                stroke,
            );
            // Slope down
            painter.line_segment(
                [pos2(cutoff_x, mid_y - amp * 0.5), pos2(cutoff_x + w * 0.2, mid_y + amp * 0.5)],
                stroke,
            );
            // Flat low line
            let dim_stroke = Stroke::new(1.0, Color32::from_rgb(color.r() / 2, color.g() / 2, color.b() / 2));
            painter.line_segment(
                [pos2(cutoff_x + w * 0.2, mid_y + amp * 0.5), pos2(rect.max.x, mid_y + amp * 0.5)],
                dim_stroke,
            );
        }
    }
}
