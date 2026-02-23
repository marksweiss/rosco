mod effects;
mod envelope;
mod sequencer;
mod transport;

use eframe::egui;
use eframe::egui::{pos2, vec2, Color32, Pos2, Rect, Stroke};

use crate::audio_gen::Waveform;
use crate::tui::audio_bridge::{AudioBridge, ParameterUpdate};
use crate::tui::app::SynthParameters;

use effects::EffectsRackState;
use envelope::EnvelopeState;
use sequencer::SequencerState;
use transport::TransportState;

pub struct RoscoGuiApp {
    audio_bridge: AudioBridge,
    params: SynthParameters,
    envelope: EnvelopeState,
    effects: EffectsRackState,
    sequencer: SequencerState,
    transport: TransportState,
    status_message: String,
}

impl RoscoGuiApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let audio_bridge = AudioBridge::new()
            .expect("Failed to create AudioBridge");
        Self {
            audio_bridge,
            params: SynthParameters::default(),
            envelope: EnvelopeState::default(),
            effects: EffectsRackState::default(),
            sequencer: SequencerState::default(),
            transport: TransportState::default(),
            status_message: "Ready".to_string(),
        }
    }

    fn send_update(&mut self, update: ParameterUpdate) {
        let description = format!("{:?}", &update);
        match self.audio_bridge.send_parameter_update(update) {
            Ok(()) => self.status_message = description,
            Err(e) => self.status_message = format!("Error: {}", e),
        }
    }

    fn render_oscillator(&mut self, ui: &mut egui::Ui) {
        ui.heading("Oscillator");
        ui.add_space(8.0);

        // Waveform selector with visual previews
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
                let selected = self.params.oscillator_waveform == *waveform;
                if waveform_button(ui, *waveform, *label, selected).clicked() && !selected {
                    self.params.oscillator_waveform = *waveform;
                    let update = ParameterUpdate::OscillatorWaveform(*waveform);
                    self.send_update(update);
                }
            }
        });

        ui.add_space(8.0);

        // Frequency slider (log-scale)
        ui.label(format!("Frequency: {:.1} Hz", self.params.oscillator_frequency));
        let freq_before = self.params.oscillator_frequency;
        ui.add(
            egui::Slider::new(&mut self.params.oscillator_frequency, 20.0..=20000.0)
                .logarithmic(true)
                .clamping(egui::SliderClamping::Always)
                .text("Hz"),
        );
        if (self.params.oscillator_frequency - freq_before).abs() > 0.01 {
            let update = ParameterUpdate::OscillatorFrequency(self.params.oscillator_frequency);
            self.send_update(update);
        }

        ui.add_space(8.0);

        // Volume slider
        ui.label(format!("Volume: {:.2}", self.params.oscillator_volume));
        let vol_before = self.params.oscillator_volume;
        ui.add(
            egui::Slider::new(&mut self.params.oscillator_volume, 0.0..=1.0)
                .clamping(egui::SliderClamping::Always)
                .text(""),
        );
        if (self.params.oscillator_volume - vol_before).abs() > 0.001 {
            let update = ParameterUpdate::OscillatorVolume(self.params.oscillator_volume);
            self.send_update(update);
        }
    }

    fn render_envelope(&mut self, ui: &mut egui::Ui) {
        ui.heading("Envelope");
        ui.add_space(4.0);

        if let Some(msg) = self.envelope.render(ui) {
            // Send bridge updates for the envelope changes
            let _ = self.audio_bridge.send_parameter_update(
                ParameterUpdate::EnvelopeAttack(self.envelope.attack.0),
            );
            let _ = self.audio_bridge.send_parameter_update(
                ParameterUpdate::EnvelopeDecay(self.envelope.decay.0),
            );
            let _ = self.audio_bridge.send_parameter_update(
                ParameterUpdate::EnvelopeSustain(self.envelope.sustain.0),
            );
            self.status_message = msg;
        }
    }

    fn render_effects(&mut self, ui: &mut egui::Ui) {
        ui.heading("Effects Rack");
        ui.add_space(4.0);

        let changes = self.effects.render(ui);
        for change in changes {
            match self.audio_bridge.send_parameter_update(change.update) {
                Ok(()) => self.status_message = change.description,
                Err(e) => self.status_message = format!("Error: {}", e),
            }
        }
    }

    fn render_sequencer(&mut self, ui: &mut egui::Ui) {
        ui.heading("Sequencer");
        ui.add_space(4.0);

        // Sync playing step from transport
        self.sequencer.playing_step = if self.transport.is_playing {
            Some(self.transport.current_step)
        } else {
            None
        };

        let changes = self.sequencer.render(ui);
        for change in changes {
            match self.audio_bridge.send_parameter_update(change.update) {
                Ok(()) => self.status_message = change.description,
                Err(e) => self.status_message = format!("Error: {}", e),
            }
        }
    }

    fn render_transport(&mut self, ui: &mut egui::Ui) {
        let changes = self.transport.render(ui);
        for change in changes {
            match self.audio_bridge.send_parameter_update(change.update) {
                Ok(()) => self.status_message = change.description,
                Err(e) => self.status_message = format!("Error: {}", e),
            }
        }
    }

    fn render_status_bar(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Status:");
            ui.monospace(&self.status_message);
        });
    }
}

impl eframe::App for RoscoGuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Status bar at the bottom
        egui::TopBottomPanel::bottom("status_bar")
            .exact_height(28.0)
            .show(ctx, |ui| {
                self.render_status_bar(ui);
            });

        // Transport bar above status
        egui::TopBottomPanel::bottom("transport")
            .resizable(false)
            .min_height(40.0)
            .show(ctx, |ui| {
                self.render_transport(ui);
            });

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Rosco Synthesizer");
            ui.separator();

            // Top row: Oscillator and Envelope side by side
            ui.columns(2, |cols| {
                cols[0].group(|ui| {
                    self.render_oscillator(ui);
                });
                cols[1].group(|ui| {
                    self.render_envelope(ui);
                });
            });

            ui.add_space(8.0);

            // Scrollable area for effects rack + sequencer
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Effects rack
                ui.group(|ui| {
                    ui.set_min_width(ui.available_width());
                    self.render_effects(ui);
                });

                ui.add_space(8.0);

                // Sequencer
                ui.group(|ui| {
                    ui.set_min_width(ui.available_width());
                    self.render_sequencer(ui);
                });
            });
        });

        // Request continuous repainting for real-time feel
        ctx.request_repaint();
    }
}

// --- Waveform preview button widget ---

const WAVE_BUTTON_SIZE: egui::Vec2 = vec2(64.0, 48.0);
const WAVE_PREVIEW_SAMPLES: usize = 32;
const WAVE_COLOR: Color32 = Color32::from_rgb(0, 180, 210);
const WAVE_SELECTED_BG: Color32 = Color32::from_rgb(40, 60, 80);
const WAVE_NORMAL_BG: Color32 = Color32::from_rgb(35, 35, 40);

fn waveform_button(
    ui: &mut egui::Ui,
    waveform: Waveform,
    label: &str,
    selected: bool,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(WAVE_BUTTON_SIZE, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();

        // Background
        let bg = if selected { WAVE_SELECTED_BG } else { WAVE_NORMAL_BG };
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
        let wave_stroke = Stroke::new(1.5, WAVE_COLOR);
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
            let t = i as f32 / (n - 1) as f32; // 0.0 to 1.0
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
            if t < 0.5 { 1.0 } else { -1.0 }
        }
        Waveform::Triangle => {
            if t < 0.5 {
                4.0 * t - 1.0
            } else {
                3.0 - 4.0 * t
            }
        }
        Waveform::Noise | Waveform::GaussianNoise => {
            // Deterministic pseudo-random for consistent preview
            let seed = (index as u32).wrapping_mul(2654435761);
            let normalized = (seed % 1000) as f32 / 500.0 - 1.0;
            normalized
        }
    }
}
