mod config;
mod dsl_bridge;
mod effects;
mod envelope;
mod sequencer;
mod shortcuts;
pub mod theme;
mod transport;
mod visualizations;

use eframe::egui;
use eframe::egui::{pos2, vec2, Color32, Pos2, Rect, Stroke};

use crate::audio_gen::Waveform;
use crate::tui::audio_bridge::{AudioBridge, AudioFeedback, ParameterUpdate};
use crate::tui::app::SynthParameters;

use config::{GuiConfig, SessionState};
use effects::EffectsRackState;
use envelope::EnvelopeState;
use sequencer::SequencerState;
use shortcuts::{FocusPanel, ShortcutAction, UndoStack};
use theme::GuiTheme;
use transport::TransportState;
use visualizations::{LevelMeterState, OscilloscopeState, SpectrumState};

pub struct RoscoGuiApp {
    audio_bridge: AudioBridge,
    params: SynthParameters,
    envelope: EnvelopeState,
    effects: EffectsRackState,
    sequencer: SequencerState,
    transport: TransportState,
    theme: GuiTheme,
    config: GuiConfig,
    focused_panel: FocusPanel,
    undo_stack: UndoStack,
    oscilloscope: OscilloscopeState,
    spectrum: SpectrumState,
    level_meters: LevelMeterState,
    show_viz: bool,
    cpu_usage: f32,
    buffer_health: f32,
    status_message: String,
}

impl RoscoGuiApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let audio_bridge = AudioBridge::new()
            .expect("Failed to create AudioBridge");
        let config = GuiConfig::load();
        let theme = config.resolve_theme();
        Self {
            audio_bridge,
            params: SynthParameters::default(),
            envelope: EnvelopeState::default(),
            effects: EffectsRackState::default(),
            sequencer: SequencerState::default(),
            transport: TransportState::default(),
            theme,
            config,
            focused_panel: FocusPanel::Oscillator,
            undo_stack: UndoStack::new(),
            oscilloscope: OscilloscopeState::default(),
            spectrum: SpectrumState::default(),
            level_meters: LevelMeterState::default(),
            show_viz: true,
            cpu_usage: 0.0,
            buffer_health: 1.0,
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

    fn process_audio_feedback(&mut self) {
        let feedback = self.audio_bridge.receive_audio_feedback();
        for fb in feedback {
            match fb {
                AudioFeedback::LevelMeter { track, level } => {
                    self.level_meters.update_track(track as usize, level);
                }
                AudioFeedback::WaveformData(data) => {
                    self.oscilloscope.update(&data);
                    self.spectrum.update_from_waveform(&data);
                }
                AudioFeedback::SpectrumData(data) => {
                    self.spectrum.update_from_spectrum(&data);
                }
                AudioFeedback::StepAdvance(step) => {
                    self.transport.current_step = step;
                }
                AudioFeedback::CpuUsage(usage) => {
                    self.cpu_usage = usage;
                }
                AudioFeedback::BufferHealth(health) => {
                    self.buffer_health = health;
                }
                AudioFeedback::PlaybackPosition(pos) => {
                    // Update position display from playback
                    let _ = pos; // available for future use
                }
            }
        }
    }

    fn render_oscillator(&mut self, ui: &mut egui::Ui) {
        ui.heading("Oscillator");
        ui.add_space(8.0);

        // Waveform selector with visual previews
        ui.label("Waveform");
        let theme = self.theme.clone();
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
                if waveform_button(ui, *waveform, *label, selected, &theme).clicked() && !selected {
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

        let theme = self.theme.clone();
        if let Some(msg) = self.envelope.render(ui, &theme) {
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
        ui.columns(2, |cols| {
            // Left column: effects chain (LFO through Filter)
            cols[0].heading("Effects Rack");
            cols[0].add_space(4.0);
            let changes = self.effects.render_effects(&mut cols[0]);
            for change in changes {
                match self.audio_bridge.send_parameter_update(change.update) {
                    Ok(()) => self.status_message = change.description,
                    Err(e) => self.status_message = format!("Error: {}", e),
                }
            }

            // Right column: equalizer always visible
            cols[1].heading("Equalizer");
            cols[1].add_space(4.0);
            let changes = self.effects.render_equalizer_panel(&mut cols[1]);
            for change in changes {
                match self.audio_bridge.send_parameter_update(change.update) {
                    Ok(()) => self.status_message = change.description,
                    Err(e) => self.status_message = format!("Error: {}", e),
                }
            }
        });
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

        let theme = self.theme.clone();
        let changes = self.sequencer.render(ui, &theme);
        for change in changes {
            match self.audio_bridge.send_parameter_update(change.update) {
                Ok(()) => self.status_message = change.description,
                Err(e) => self.status_message = format!("Error: {}", e),
            }
        }
    }

    fn render_transport(&mut self, ui: &mut egui::Ui) {
        let theme = self.theme.clone();
        let changes = self.transport.render(ui, &theme);
        for change in changes {
            match self.audio_bridge.send_parameter_update(change.update) {
                Ok(()) => self.status_message = change.description,
                Err(e) => self.status_message = format!("Error: {}", e),
            }
        }
    }

    fn render_status_bar(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(format!("[{}]", self.focused_panel.label()));
            ui.separator();
            ui.monospace(format!("CPU: {:.0}%", self.cpu_usage * 100.0));
            ui.separator();
            ui.monospace(format!("Buf: {:.0}%", self.buffer_health * 100.0));
            ui.separator();
            ui.label("Status:");
            ui.monospace(&self.status_message);
        });
    }

    fn dispatch_shortcut(&mut self, action: ShortcutAction) {
        match action {
            ShortcutAction::PlayPause => {
                self.transport.is_playing = !self.transport.is_playing;
                let update = if self.transport.is_playing {
                    ParameterUpdate::TransportPlay
                } else {
                    ParameterUpdate::TransportStop
                };
                self.send_update(update);
            }
            ShortcutAction::Stop => {
                self.transport.is_playing = false;
                self.transport.current_step = 0;
                self.transport.position = transport::PlaybackPosition::default();
                self.send_update(ParameterUpdate::TransportStop);
            }
            ShortcutAction::NextPanel => {
                self.focused_panel = self.focused_panel.next();
                self.status_message = format!("Focus: {}", self.focused_panel.label());
            }
            ShortcutAction::PrevPanel => {
                self.focused_panel = self.focused_panel.prev();
                self.status_message = format!("Focus: {}", self.focused_panel.label());
            }
            ShortcutAction::Undo => {
                if let Some((update, desc)) = self.undo_stack.undo() {
                    match self.audio_bridge.send_parameter_update(update) {
                        Ok(()) => self.status_message = desc,
                        Err(e) => self.status_message = format!("Undo error: {}", e),
                    }
                } else {
                    self.status_message = "Nothing to undo".to_string();
                }
            }
            ShortcutAction::Redo => {
                if let Some((update, desc)) = self.undo_stack.redo() {
                    match self.audio_bridge.send_parameter_update(update) {
                        Ok(()) => self.status_message = desc,
                        Err(e) => self.status_message = format!("Redo error: {}", e),
                    }
                } else {
                    self.status_message = "Nothing to redo".to_string();
                }
            }
            ShortcutAction::SaveSession => {
                self.save_session();
                self.config.save();
                self.status_message = "Session saved".to_string();
            }
            ShortcutAction::LoadFile => {
                self.load_dsl_dialog();
            }
        }
    }

    fn save_session(&self) {
        let waveform_name = format!("{:?}", self.params.oscillator_waveform);
        let session = SessionState {
            oscillator_waveform: waveform_name,
            oscillator_frequency: self.params.oscillator_frequency,
            oscillator_volume: self.params.oscillator_volume,
            tempo: self.transport.tempo,
            envelope: self.envelope.to_serializable(),
            effects: self.effects.clone(),
            tracks: self.sequencer.tracks.to_vec(),
        };
        session.save();
    }

    fn load_dsl_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("DSL Files", &["dsl"])
            .add_filter("All Files", &["*"])
            .pick_file()
        {
            let path_str = path.display().to_string();
            match dsl_bridge::load_dsl_file(&path_str) {
                Ok(result) => {
                    self.envelope = result.envelope;
                    self.effects = result.effects;
                    self.sequencer = result.sequencer;
                    self.transport.tempo = result.tempo;
                    self.config.add_recent_file(&path_str);
                    self.status_message = result.status;
                }
                Err(e) => {
                    self.status_message = format!("DSL load error: {}", e);
                }
            }
        }
    }

    fn export_dsl_dialog(&self) {
        let dsl_text = dsl_bridge::export_dsl_string(
            &self.envelope,
            &self.effects,
            &self.sequencer,
            self.transport.tempo,
        );
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("DSL Files", &["dsl"])
            .set_file_name("export.dsl")
            .save_file()
        {
            match std::fs::write(&path, &dsl_text) {
                Ok(()) => {
                    // Can't mutate self here since &self, but that's fine —
                    // status update happens in the caller
                }
                Err(_) => {}
            }
        }
    }

    fn render_theme_selector(&mut self, ui: &mut egui::Ui) {
        let current = self.theme.name.clone();
        egui::ComboBox::from_label("Theme")
            .selected_text(&current)
            .show_ui(ui, |ui| {
                if ui.selectable_value(&mut self.theme.name, "Dark".to_string(), "Dark").changed() {
                    self.theme = GuiTheme::dark();
                    self.config.theme_name = "Dark".to_string();
                    self.status_message = "Theme -> Dark".to_string();
                }
                if ui.selectable_value(&mut self.theme.name, "Light".to_string(), "Light").changed() {
                    self.theme = GuiTheme::light();
                    self.config.theme_name = "Light".to_string();
                    self.status_message = "Theme -> Light".to_string();
                }
            });
    }
}

impl eframe::App for RoscoGuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process keyboard shortcuts
        if let Some(action) = shortcuts::process_shortcuts(ctx) {
            self.dispatch_shortcut(action);
        }

        // Process audio feedback from the bridge
        self.process_audio_feedback();

        // Top menu bar with file operations, theme selector, and viz toggle
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open DSL... (Ctrl+O)").clicked() {
                        ui.close_menu();
                        self.load_dsl_dialog();
                    }
                    if ui.button("Export DSL...").clicked() {
                        ui.close_menu();
                        self.export_dsl_dialog();
                    }
                    ui.separator();
                    if ui.button("Save Session (Ctrl+S)").clicked() {
                        ui.close_menu();
                        self.save_session();
                        self.config.save();
                        self.status_message = "Session saved".to_string();
                    }
                });
                ui.separator();
                self.render_theme_selector(ui);
                ui.separator();
                ui.toggle_value(&mut self.show_viz, "Viz");
            });
        });

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

        // Visualization panel (collapsible)
        if self.show_viz {
            egui::TopBottomPanel::bottom("visualizations")
                .resizable(true)
                .min_height(130.0)
                .max_height(200.0)
                .show(ctx, |ui| {
                    let theme = self.theme.clone();
                    ui.columns(2, |cols| {
                        cols[0].group(|ui| {
                            ui.label("Oscilloscope");
                            self.oscilloscope.render(ui, &theme);
                        });
                        cols[1].group(|ui| {
                            ui.label("Spectrum");
                            self.spectrum.render(ui, &theme);
                        });
                    });
                });
        }

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.separator();

            // Top row: Oscillator and Envelope side by side
            const TOP_ROW_HEIGHT: f32 = 220.0;
            ui.columns(2, |cols| {
                cols[0].group(|ui| {
                    ui.set_min_height(TOP_ROW_HEIGHT);
                    ui.set_max_height(TOP_ROW_HEIGHT);
                    self.render_oscillator(ui);
                });
                cols[1].group(|ui| {
                    ui.set_min_height(TOP_ROW_HEIGHT);
                    ui.set_max_height(TOP_ROW_HEIGHT);
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

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.config.theme_name = self.theme.name.clone();
        self.config.save();
        self.save_session();
    }
}

// --- Waveform preview button widget ---

const WAVE_BUTTON_SIZE: egui::Vec2 = vec2(64.0, 48.0);
const WAVE_PREVIEW_SAMPLES: usize = 32;

fn waveform_button(
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
        let bg = if selected { theme.wave_selected_bg() } else { theme.wave_normal_bg() };
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
