use eframe::egui;
use eframe::egui::{pos2, vec2, Color32, Pos2, Rect, Stroke};

// GUI-local envelope types (the engine types are pub(crate), so we mirror them here)

#[derive(Clone, Copy, Debug, PartialEq)]
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

/// GUI state for the interactive ADSR envelope editor.
/// Mirrors the engine's Envelope struct (src/envelope/envelope.rs) but
/// is owned by the GUI and communicated to audio via ParameterUpdate.
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

    // Exponential steepness (1.0–20.0, default 5.0)
    pub steepness: f32,

    // Drag interaction state
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
            steepness: 5.0,
            dragging: None,
        }
    }
}

// Colors
const CURVE_COLOR: Color32 = Color32::from_rgb(0, 204, 204);
const POINT_COLOR: Color32 = Color32::WHITE;
const POINT_DRAG_COLOR: Color32 = Color32::from_rgb(255, 220, 50);
const GRID_COLOR: Color32 = Color32::from_rgb(50, 50, 55);
const BG_COLOR: Color32 = Color32::from_rgb(30, 30, 35);
const FIXED_POINT_COLOR: Color32 = Color32::from_rgb(120, 120, 130);

const POINT_RADIUS: f32 = 6.0;
const HIT_RADIUS: f32 = 14.0;
const CURVE_SAMPLES: usize = 48;

impl EnvelopeState {
    /// Render the full envelope panel. Returns a status message if a parameter changed.
    pub fn render(&mut self, ui: &mut egui::Ui) -> Option<String> {
        let mut status: Option<String> = None;

        // --- Curve type toggles ---
        let mut curve_status: Option<String> = None;
        ui.horizontal(|ui| {
            ui.label("Curves:");
            curve_status = curve_status.take().or(self.curve_toggle(ui, "Atk", DragTarget::Attack));
            curve_status = curve_status.take().or(self.curve_toggle(ui, "Dec", DragTarget::Decay));
            curve_status = curve_status.take().or(self.curve_toggle(ui, "Sus", DragTarget::Sustain));
            curve_status = curve_status.take().or(self.curve_toggle(ui, "Rel", DragTarget::Sustain));
        });
        status = status.or(curve_status);

        // --- Steepness slider (only when at least one segment is exponential) ---
        if self.has_exponential() {
            ui.add_space(4.0);
            let before = self.steepness;
            ui.add(
                egui::Slider::new(&mut self.steepness, 1.0..=20.0)
                    .text("Steepness"),
            );
            if (self.steepness - before).abs() > 0.01 {
                status = Some(format!("Envelope steepness → {:.1}", self.steepness));
            }
        }

        ui.add_space(4.0);

        // --- Envelope graph ---
        let desired_size = vec2(ui.available_width().max(200.0), 180.0);
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click_and_drag());

        let painter = ui.painter_at(rect);

        // Background
        painter.rect_filled(rect, 4.0, BG_COLOR);

        // Grid
        self.draw_grid(&painter, rect);

        // Envelope curve
        self.draw_curve(&painter, rect);

        // Control points
        self.draw_points(&painter, rect);

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

    fn curve_toggle(&mut self, ui: &mut egui::Ui, label: &str, target: DragTarget) -> Option<String> {
        let curve = match target {
            DragTarget::Attack => &mut self.attack_curve,
            DragTarget::Decay => &mut self.decay_curve,
            DragTarget::Sustain => &mut self.sustain_curve,
        };
        // Also handle release via the Sustain target's second call
        // (the caller passes "Rel" label with DragTarget::Sustain for release)
        let (curve_ref, seg_name) = if label == "Rel" {
            (&mut self.release_curve, "release")
        } else {
            (curve, match target {
                DragTarget::Attack => "attack",
                DragTarget::Decay => "decay",
                DragTarget::Sustain => "sustain",
            })
        };

        let is_exp = *curve_ref == CurveType::Exponential;
        let button_text = if is_exp {
            format!("{}: Exp", label)
        } else {
            format!("{}: Lin", label)
        };
        if ui.selectable_label(is_exp, button_text).clicked() {
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

    fn has_exponential(&self) -> bool {
        self.attack_curve == CurveType::Exponential
            || self.decay_curve == CurveType::Exponential
            || self.sustain_curve == CurveType::Exponential
            || self.release_curve == CurveType::Exponential
    }

    // --- Coordinate conversion ---

    fn to_screen(&self, point: (f32, f32), rect: Rect) -> Pos2 {
        let x = rect.min.x + point.0 * rect.width();
        let y = rect.max.y - point.1 * rect.height();
        pos2(x, y)
    }

    fn from_screen(&self, screen: Pos2, rect: Rect) -> (f32, f32) {
        let x = ((screen.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
        let y = ((rect.max.y - screen.y) / rect.height()).clamp(0.0, 1.0);
        (x, y)
    }

    // --- Drawing ---

    fn draw_grid(&self, painter: &egui::Painter, rect: Rect) {
        let stroke = Stroke::new(1.0, GRID_COLOR);
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

    fn draw_curve(&self, painter: &egui::Painter, rect: Rect) {
        let stroke = Stroke::new(2.0, CURVE_COLOR);
        let start = (0.0_f32, 0.0_f32);
        let release = (1.0_f32, 0.0_f32);

        let segments: [(f32, f32); 5] = [start, self.attack, self.decay, self.sustain, release];
        let curves = [
            self.attack_curve,
            self.decay_curve,
            self.sustain_curve,
            self.release_curve,
        ];

        for i in 0..4 {
            let seg_start = segments[i];
            let seg_end = segments[i + 1];
            let curve = curves[i];
            self.draw_segment(painter, rect, seg_start, seg_end, curve, stroke);
        }
    }

    fn draw_segment(
        &self,
        painter: &egui::Painter,
        rect: Rect,
        start: (f32, f32),
        end: (f32, f32),
        curve: CurveType,
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
                    let volume = self.exponential_interp(start, end, t);
                    points.push(self.to_screen((position, volume), rect));
                }
                for pair in points.windows(2) {
                    painter.line_segment([pair[0], pair[1]], stroke);
                }
            }
        }
    }

    fn exponential_interp(&self, start: (f32, f32), end: (f32, f32), t: f32) -> f32 {
        // Matches src/envelope/envelope.rs:146-152
        let exp_t = 1.0 - (-self.steepness * t).exp();
        let exp_t_normalized = exp_t / (1.0 - (-self.steepness).exp());
        start.1 + (end.1 - start.1) * exp_t_normalized
    }

    fn draw_points(&self, painter: &egui::Painter, rect: Rect) {
        // Fixed points (start and release)
        let start_pos = self.to_screen((0.0, 0.0), rect);
        let release_pos = self.to_screen((1.0, 0.0), rect);
        painter.circle_filled(start_pos, 4.0, FIXED_POINT_COLOR);
        painter.circle_filled(release_pos, 4.0, FIXED_POINT_COLOR);

        // Draggable points
        let targets = [
            (DragTarget::Attack, self.attack, "A"),
            (DragTarget::Decay, self.decay, "D"),
            (DragTarget::Sustain, self.sustain, "S"),
        ];
        for (target, point, label) in &targets {
            let screen_pos = self.to_screen(*point, rect);
            let is_dragging = self.dragging == Some(*target);
            let color = if is_dragging { POINT_DRAG_COLOR } else { POINT_COLOR };
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
