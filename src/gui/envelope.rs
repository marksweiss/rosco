use eframe::egui;
use eframe::egui::{pos2, vec2, Pos2, Rect, Stroke};
use serde::{Deserialize, Serialize};

use super::theme::GuiTheme;

// GUI-local envelope types (the engine types are pub(crate), so we mirror them here)

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum CurveType {
    Linear,
    Exponential,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DragTarget {
    Attack,
    Decay,
    Sustain,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Segment {
    Attack,
    Decay,
    Sustain,
    Release,
}

/// GUI state for the interactive ADSR envelope editor.
/// Mirrors the engine's Envelope struct (src/envelope/envelope.rs) but
/// is owned by the GUI and communicated to audio via ParameterUpdate.
#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct EnvelopeState {
    // ADSR control points: (position 0.0–1.0, volume 0.0–1.0)
    pub attack: (f32, f32),
    pub decay: (f32, f32),
    pub sustain: (f32, f32),

    // Per-segment curve types
    pub attack_curve: CurveType,
    pub decay_curve: CurveType,
    pub sustain_curve: CurveType,
    pub release_curve: CurveType,

    // Per-segment exponential steepness (1.0–20.0, default 5.0)
    pub attack_steepness: f32,
    pub decay_steepness: f32,
    pub sustain_steepness: f32,
    pub release_steepness: f32,

    // Currently selected segment for steepness editing
    #[serde(skip)]
    pub selected_segment: Option<Segment>,

    // Drag interaction state
    #[serde(skip)]
    dragging: Option<DragTarget>,
}

impl Default for EnvelopeState {
    fn default() -> Self {
        Self {
            attack: (0.02, 1.0),
            decay: (0.51, 1.0),
            sustain: (0.98, 1.0),
            attack_curve: CurveType::Linear,
            decay_curve: CurveType::Linear,
            sustain_curve: CurveType::Linear,
            release_curve: CurveType::Linear,
            attack_steepness: 5.0,
            decay_steepness: 5.0,
            sustain_steepness: 5.0,
            release_steepness: 5.0,
            selected_segment: None,
            dragging: None,
        }
    }
}

const POINT_RADIUS: f32 = 6.0;
const HIT_RADIUS: f32 = 14.0;
const CURVE_SAMPLES: usize = 48;

impl EnvelopeState {
    /// Create a copy suitable for serialization (resets transient drag state).
    pub fn to_serializable(&self) -> Self {
        Self {
            attack: self.attack,
            decay: self.decay,
            sustain: self.sustain,
            attack_curve: self.attack_curve,
            decay_curve: self.decay_curve,
            sustain_curve: self.sustain_curve,
            release_curve: self.release_curve,
            attack_steepness: self.attack_steepness,
            decay_steepness: self.decay_steepness,
            sustain_steepness: self.sustain_steepness,
            release_steepness: self.release_steepness,
            selected_segment: None,
            dragging: None,
        }
    }

    /// Render the full envelope panel. Returns a status message if a parameter changed.
    pub fn render(&mut self, ui: &mut egui::Ui, theme: &GuiTheme) -> Option<String> {
        let mut status: Option<String> = None;

        // --- Curve type toggles (click to select segment + toggle Exp/Lin) ---
        let mut curve_status: Option<String> = None;
        ui.horizontal(|ui| {
            ui.label("Curves:");
            curve_status = curve_status.take().or(self.curve_toggle(ui, "Atk", Segment::Attack));
            curve_status = curve_status.take().or(self.curve_toggle(ui, "Dec", Segment::Decay));
            curve_status = curve_status.take().or(self.curve_toggle(ui, "Sus", Segment::Sustain));
            curve_status = curve_status.take().or(self.curve_toggle(ui, "Rel", Segment::Release));
        });
        status = status.or(curve_status);

        // --- Steepness slider (only for selected exponential segment) ---
        if let Some(seg) = self.selected_segment {
            let curve = match seg {
                Segment::Attack => self.attack_curve,
                Segment::Decay => self.decay_curve,
                Segment::Sustain => self.sustain_curve,
                Segment::Release => self.release_curve,
            };
            if curve == CurveType::Exponential {
                ui.add_space(4.0);
                let (label, steepness) = match seg {
                    Segment::Attack => ("Atk Steepness", &mut self.attack_steepness),
                    Segment::Decay => ("Dec Steepness", &mut self.decay_steepness),
                    Segment::Sustain => ("Sus Steepness", &mut self.sustain_steepness),
                    Segment::Release => ("Rel Steepness", &mut self.release_steepness),
                };
                let before = *steepness;
                ui.add(
                    egui::Slider::new(steepness, 1.0..=20.0)
                        .text(label),
                );
                if (*steepness - before).abs() > 0.01 {
                    status = Some(format!("Envelope {} → {:.1}", label.to_lowercase(), *steepness));
                }
            }
        }

        ui.add_space(4.0);

        // --- Envelope graph (fill remaining height, reserving space for readout) ---
        let readout_height = 20.0;
        let graph_height = (ui.available_height() - readout_height - ui.spacing().item_spacing.y).max(40.0);
        let desired_size = vec2(ui.available_width().max(200.0), graph_height);
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click_and_drag());

        let painter = ui.painter_at(rect);

        // Background
        painter.rect_filled(rect, 4.0, theme.envelope_bg_color());

        // Grid
        self.draw_grid(&painter, rect, theme);

        // Envelope curve
        self.draw_curve(&painter, rect, theme);

        // Control points
        self.draw_points(&painter, rect, theme);

        // Interaction
        if let Some(msg) = self.handle_interaction(&response, rect) {
            status = Some(msg);
        }

        // Value readout below graph
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 12.0;
            ui.small(format!(
                "A: ({:.2}, {:.2})",
                self.attack.0, self.attack.1
            ));
            ui.small(format!(
                "D: ({:.2}, {:.2})",
                self.decay.0, self.decay.1
            ));
            ui.small(format!(
                "S: ({:.2}, {:.2})",
                self.sustain.0, self.sustain.1
            ));
        });

        status
    }

    fn curve_toggle(&mut self, ui: &mut egui::Ui, label: &str, segment: Segment) -> Option<String> {
        let curve_ref = match segment {
            Segment::Attack => &mut self.attack_curve,
            Segment::Decay => &mut self.decay_curve,
            Segment::Sustain => &mut self.sustain_curve,
            Segment::Release => &mut self.release_curve,
        };
        let seg_name = match segment {
            Segment::Attack => "attack",
            Segment::Decay => "decay",
            Segment::Sustain => "sustain",
            Segment::Release => "release",
        };

        let is_exp = *curve_ref == CurveType::Exponential;
        let is_selected = self.selected_segment == Some(segment);
        let button_text = if is_exp {
            format!("{}: Exp", label)
        } else {
            format!("{}: Lin", label)
        };
        if ui.selectable_label(is_selected, button_text).clicked() {
            // Always select this segment
            self.selected_segment = Some(segment);
            // Toggle Exp/Lin
            *curve_ref = if is_exp {
                CurveType::Linear
            } else {
                CurveType::Exponential
            };
            let new_type = if *curve_ref == CurveType::Exponential { "Exponential" } else { "Linear" };
            return Some(format!("Envelope {} curve → {}", seg_name, new_type));
        }
        None
    }

    // --- Coordinate conversion ---

    fn to_screen(&self, point: (f32, f32), rect: Rect) -> Pos2 {
        let pad = POINT_RADIUS;
        let x = rect.min.x + point.0 * rect.width();
        let y = (rect.max.y - pad) - point.1 * (rect.height() - 2.0 * pad);
        pos2(x, y)
    }

    fn from_screen(&self, screen: Pos2, rect: Rect) -> (f32, f32) {
        let pad = POINT_RADIUS;
        let x = ((screen.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
        let y = (((rect.max.y - pad) - screen.y) / (rect.height() - 2.0 * pad)).clamp(0.0, 1.0);
        (x, y)
    }

    // --- Drawing ---

    fn draw_grid(&self, painter: &egui::Painter, rect: Rect, theme: &GuiTheme) {
        let stroke = Stroke::new(1.0, theme.envelope_grid_color());
        // Horizontal lines at 0.25, 0.5, 0.75
        for frac in [0.25, 0.5, 0.75] {
            let y = rect.max.y - frac * rect.height();
            painter.line_segment(
                [pos2(rect.min.x, y), pos2(rect.max.x, y)],
                stroke,
            );
        }
        // Vertical lines at 0.25, 0.5, 0.75
        for frac in [0.25, 0.5, 0.75] {
            let x = rect.min.x + frac * rect.width();
            painter.line_segment(
                [pos2(x, rect.min.y), pos2(x, rect.max.y)],
                stroke,
            );
        }
    }

    fn draw_curve(&self, painter: &egui::Painter, rect: Rect, theme: &GuiTheme) {
        let stroke = Stroke::new(2.0, theme.envelope_curve_color());
        let start = (0.0_f32, 0.0_f32);
        let release = (1.0_f32, 0.0_f32);

        let segments: [(f32, f32); 5] = [start, self.attack, self.decay, self.sustain, release];
        let curves = [
            self.attack_curve,
            self.decay_curve,
            self.sustain_curve,
            self.release_curve,
        ];
        let steepnesses = [
            self.attack_steepness,
            self.decay_steepness,
            self.sustain_steepness,
            self.release_steepness,
        ];

        for i in 0..4 {
            let seg_start = segments[i];
            let seg_end = segments[i + 1];
            let curve = curves[i];
            let steepness = steepnesses[i];
            self.draw_segment(painter, rect, seg_start, seg_end, curve, steepness, stroke);
        }
    }

    fn draw_segment(
        &self,
        painter: &egui::Painter,
        rect: Rect,
        start: (f32, f32),
        end: (f32, f32),
        curve: CurveType,
        steepness: f32,
        stroke: Stroke,
    ) {
        match curve {
            CurveType::Linear => {
                painter.line_segment(
                    [self.to_screen(start, rect), self.to_screen(end, rect)],
                    stroke,
                );
            }
            CurveType::Exponential => {
                let mut points = Vec::with_capacity(CURVE_SAMPLES + 1);
                for i in 0..=CURVE_SAMPLES {
                    let t = i as f32 / CURVE_SAMPLES as f32;
                    let position = start.0 + t * (end.0 - start.0);
                    let volume = Self::exponential_interp(start, end, t, steepness);
                    points.push(self.to_screen((position, volume), rect));
                }
                for pair in points.windows(2) {
                    painter.line_segment([pair[0], pair[1]], stroke);
                }
            }
        }
    }

    fn exponential_interp(start: (f32, f32), end: (f32, f32), t: f32, steepness: f32) -> f32 {
        // Matches src/envelope/envelope.rs:146-152
        let exp_t = 1.0 - (-steepness * t).exp();
        let exp_t_normalized = exp_t / (1.0 - (-steepness).exp());
        start.1 + (end.1 - start.1) * exp_t_normalized
    }

    fn draw_points(&self, painter: &egui::Painter, rect: Rect, theme: &GuiTheme) {
        // Fixed points (start and release)
        let start_pos = self.to_screen((0.0, 0.0), rect);
        let release_pos = self.to_screen((1.0, 0.0), rect);
        painter.circle_filled(start_pos, 4.0, theme.envelope_fixed_point_color());
        painter.circle_filled(release_pos, 4.0, theme.envelope_fixed_point_color());

        // Draggable points
        let targets = [
            (DragTarget::Attack, self.attack, "A"),
            (DragTarget::Decay, self.decay, "D"),
            (DragTarget::Sustain, self.sustain, "S"),
        ];
        for (target, point, label) in &targets {
            let screen_pos = self.to_screen(*point, rect);
            let is_dragging = self.dragging == Some(*target);
            let color = if is_dragging {
                theme.envelope_point_drag_color()
            } else {
                theme.envelope_point_color()
            };
            let radius = if is_dragging { POINT_RADIUS + 2.0 } else { POINT_RADIUS };
            painter.circle_filled(screen_pos, radius, color);
            painter.text(
                screen_pos + vec2(0.0, -radius - 4.0),
                egui::Align2::CENTER_BOTTOM,
                *label,
                egui::FontId::proportional(10.0),
                color,
            );
        }
    }

    // --- Interaction ---

    fn handle_interaction(&mut self, response: &egui::Response, rect: Rect) -> Option<String> {
        if response.drag_started() {
            if let Some(pos) = response.interact_pointer_pos() {
                let candidates = [
                    (DragTarget::Attack, self.attack),
                    (DragTarget::Decay, self.decay),
                    (DragTarget::Sustain, self.sustain),
                ];
                let mut best: Option<(DragTarget, f32)> = None;
                for (target, point) in &candidates {
                    let screen_pos = self.to_screen(*point, rect);
                    let dist = pos.distance(screen_pos);
                    if dist < HIT_RADIUS {
                        if best.is_none() || dist < best.unwrap().1 {
                            best = Some((*target, dist));
                        }
                    }
                }
                self.dragging = best.map(|(t, _)| t);
            }
        }

        let mut status = None;

        if response.dragged() {
            if let (Some(target), Some(pos)) = (self.dragging, response.interact_pointer_pos()) {
                let raw = self.from_screen(pos, rect);
                self.update_point(target, raw);
                let point = match target {
                    DragTarget::Attack => self.attack,
                    DragTarget::Decay => self.decay,
                    DragTarget::Sustain => self.sustain,
                };
                let name = match target {
                    DragTarget::Attack => "attack",
                    DragTarget::Decay => "decay",
                    DragTarget::Sustain => "sustain",
                };
                status = Some(format!(
                    "Envelope {} → ({:.2}, {:.2})",
                    name, point.0, point.1
                ));
            }
        }

        if response.drag_stopped() {
            self.dragging = None;
        }

        status
    }

    fn update_point(&mut self, target: DragTarget, raw: (f32, f32)) {
        let volume = raw.1.clamp(0.0, 1.0);
        match target {
            DragTarget::Attack => {
                let pos = raw.0.clamp(0.001, self.decay.0 - 0.001);
                self.attack = (pos, volume);
            }
            DragTarget::Decay => {
                let pos = raw.0.clamp(self.attack.0 + 0.001, self.sustain.0 - 0.001);
                self.decay = (pos, volume);
            }
            DragTarget::Sustain => {
                let pos = raw.0.clamp(self.decay.0 + 0.001, 0.999);
                self.sustain = (pos, volume);
            }
        }
    }
}
