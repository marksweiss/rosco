use eframe::egui;

use crate::tui::audio_bridge::{FilterKind, ParameterUpdate};

/// A parameter change with a human-readable description.
pub struct EffectChange {
    pub update: ParameterUpdate,
    pub description: String,
}

// --- Per-effect state structs ---

pub struct LfoState {
    pub enabled: bool,
    pub frequency: f32,
    pub amplitude: f32,
}

impl Default for LfoState {
    fn default() -> Self {
        Self {
            enabled: false,
            frequency: 4410.0,
            amplitude: 0.5,
        }
    }
}

pub struct TremoloState {
    pub enabled: bool,
    pub mod_freq: f32,
    pub mod_depth: f32,
}

impl Default for TremoloState {
    fn default() -> Self {
        Self {
            enabled: false,
            mod_freq: 5.0,
            mod_depth: 0.5,
        }
    }
}

pub struct VibratoState {
    pub enabled: bool,
    pub avg_delay: f32,
    pub mod_width: f32,
    pub mod_freq: f32,
}

impl Default for VibratoState {
    fn default() -> Self {
        Self {
            enabled: false,
            avg_delay: 0.007,
            mod_width: 0.003,
            mod_freq: 5.0,
        }
    }
}

pub struct FlangerState {
    pub enabled: bool,
    pub delay_ms: f32,
    pub depth_ms: f32,
    pub rate_hz: f32,
    pub mix: f32,
    pub feedback: f32,
}

impl Default for FlangerState {
    fn default() -> Self {
        Self {
            enabled: false,
            delay_ms: 5.0,
            depth_ms: 4.0,
            rate_hz: 0.25,
            mix: 0.5,
            feedback: 0.3,
        }
    }
}

pub struct DelayState {
    pub enabled: bool,
    pub mix: f32,
    pub decay: f32,
    pub interval_ms: f32,
    pub duration_ms: f32,
    pub num_repeats: usize,
}

impl Default for DelayState {
    fn default() -> Self {
        Self {
            enabled: false,
            mix: 1.0,
            decay: 0.5,
            interval_ms: 100.0,
            duration_ms: 20.0,
            num_repeats: 4,
        }
    }
}

pub struct ChorusState {
    pub enabled: bool,
    pub chorus_count: usize,
    pub dry_gain: f32,
    pub voice_gains: Vec<f32>,
    pub voice_delays: Vec<f32>,
}

impl Default for ChorusState {
    fn default() -> Self {
        Self {
            enabled: false,
            chorus_count: 3,
            dry_gain: 0.7,
            voice_gains: vec![0.4, 0.4, 0.4],
            voice_delays: vec![0.015, 0.020, 0.030],
        }
    }
}

impl ChorusState {
    fn resize_voices(&mut self, new_count: usize) {
        while self.voice_gains.len() < new_count {
            self.voice_gains.push(0.4);
        }
        self.voice_gains.truncate(new_count);
        while self.voice_delays.len() < new_count {
            self.voice_delays.push(0.020);
        }
        self.voice_delays.truncate(new_count);
        self.chorus_count = new_count;
    }
}

pub struct FilterState {
    pub enabled: bool,
    pub kind: FilterKind,
    pub cutoff: f32,
    pub resonance: f32,
    pub mix: f32,
}

impl Default for FilterState {
    fn default() -> Self {
        Self {
            enabled: false,
            kind: FilterKind::LowPass,
            cutoff: 1000.0,
            resonance: 0.0,
            mix: 1.0,
        }
    }
}

pub struct EqualizerState {
    pub enabled: bool,
    pub gains: [f32; 8],
}

const EQ_LABELS: [&str; 8] = ["63", "125", "250", "500", "1k", "2k", "4k", "8k"];

impl Default for EqualizerState {
    fn default() -> Self {
        Self {
            enabled: false,
            gains: [0.0; 8],
        }
    }
}

// --- Main effects rack ---

pub struct EffectsRackState {
    pub lfo: LfoState,
    pub tremolo: TremoloState,
    pub vibrato: VibratoState,
    pub flanger: FlangerState,
    pub delay: DelayState,
    pub chorus: ChorusState,
    pub filter: FilterState,
    pub equalizer: EqualizerState,
}

impl Default for EffectsRackState {
    fn default() -> Self {
        Self {
            lfo: LfoState::default(),
            tremolo: TremoloState::default(),
            vibrato: VibratoState::default(),
            flanger: FlangerState::default(),
            delay: DelayState::default(),
            chorus: ChorusState::default(),
            filter: FilterState::default(),
            equalizer: EqualizerState::default(),
        }
    }
}

impl EffectsRackState {
    /// Render the full effects rack. Returns a list of parameter changes.
    pub fn render(&mut self, ui: &mut egui::Ui) -> Vec<EffectChange> {
        let mut changes = Vec::new();

        // Effects in processing chain order
        self.render_lfo(ui, &mut changes);
        self.render_tremolo(ui, &mut changes);
        self.render_vibrato(ui, &mut changes);
        self.render_flanger(ui, &mut changes);
        self.render_delay(ui, &mut changes);
        self.render_chorus(ui, &mut changes);
        self.render_filter(ui, &mut changes);
        self.render_equalizer(ui, &mut changes);

        changes
    }

    // --- LFO ---

    fn render_lfo(&mut self, ui: &mut egui::Ui, changes: &mut Vec<EffectChange>) {
        let id = ui.make_persistent_id("fx_lfo");
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false)
            .show_header(ui, |ui| {
                ui.checkbox(&mut self.lfo.enabled, "");
                ui.strong("LFO");
            })
            .body(|ui| {
                if !self.lfo.enabled { ui.disable(); }
                let before_freq = self.lfo.frequency;
                ui.add(
                    egui::Slider::new(&mut self.lfo.frequency, 0.01..=22050.0)
                        .logarithmic(true)
                        .text("Freq (Hz)"),
                );
                if (self.lfo.frequency - before_freq).abs() > 0.001 {
                    changes.push(EffectChange {
                        update: ParameterUpdate::LfoFrequency(self.lfo.frequency),
                        description: format!("LFO frequency → {:.1} Hz", self.lfo.frequency),
                    });
                }

                let before_amp = self.lfo.amplitude;
                ui.add(egui::Slider::new(&mut self.lfo.amplitude, 0.0..=1.0).text("Amplitude"));
                if (self.lfo.amplitude - before_amp).abs() > 0.001 {
                    changes.push(EffectChange {
                        update: ParameterUpdate::LfoAmplitude(self.lfo.amplitude),
                        description: format!("LFO amplitude → {:.2}", self.lfo.amplitude),
                    });
                }
            });
    }

    // --- Tremolo ---

    fn render_tremolo(&mut self, ui: &mut egui::Ui, changes: &mut Vec<EffectChange>) {
        let id = ui.make_persistent_id("fx_tremolo");
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false)
            .show_header(ui, |ui| {
                ui.checkbox(&mut self.tremolo.enabled, "");
                ui.strong("Tremolo");
            })
            .body(|ui| {
                if !self.tremolo.enabled { ui.disable(); }
                let before = self.tremolo.mod_freq;
                ui.add(egui::Slider::new(&mut self.tremolo.mod_freq, 0.1..=20.0).text("Mod Freq (Hz)"));
                if (self.tremolo.mod_freq - before).abs() > 0.01 {
                    changes.push(EffectChange {
                        update: ParameterUpdate::TremoloModFreq(self.tremolo.mod_freq),
                        description: format!("Tremolo mod freq → {:.1} Hz", self.tremolo.mod_freq),
                    });
                }

                let before = self.tremolo.mod_depth;
                ui.add(egui::Slider::new(&mut self.tremolo.mod_depth, 0.0..=1.0).text("Mod Depth"));
                if (self.tremolo.mod_depth - before).abs() > 0.001 {
                    changes.push(EffectChange {
                        update: ParameterUpdate::TremoloModDepth(self.tremolo.mod_depth),
                        description: format!("Tremolo mod depth → {:.2}", self.tremolo.mod_depth),
                    });
                }
            });
    }

    // --- Vibrato ---

    fn render_vibrato(&mut self, ui: &mut egui::Ui, changes: &mut Vec<EffectChange>) {
        let id = ui.make_persistent_id("fx_vibrato");
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false)
            .show_header(ui, |ui| {
                ui.checkbox(&mut self.vibrato.enabled, "");
                ui.strong("Vibrato");
            })
            .body(|ui| {
                if !self.vibrato.enabled { ui.disable(); }
                let before = self.vibrato.avg_delay;
                ui.add(
                    egui::Slider::new(&mut self.vibrato.avg_delay, 0.001..=0.020)
                        .text("Avg Delay (s)")
                        .custom_formatter(|v, _| format!("{:.1} ms", v * 1000.0)),
                );
                if (self.vibrato.avg_delay - before).abs() > 0.0001 {
                    changes.push(EffectChange {
                        update: ParameterUpdate::VibratoAvgDelay(self.vibrato.avg_delay),
                        description: format!("Vibrato avg delay → {:.1} ms", self.vibrato.avg_delay * 1000.0),
                    });
                }

                let before = self.vibrato.mod_width;
                ui.add(
                    egui::Slider::new(&mut self.vibrato.mod_width, 0.001..=0.010)
                        .text("Mod Width (s)")
                        .custom_formatter(|v, _| format!("{:.1} ms", v * 1000.0)),
                );
                if (self.vibrato.mod_width - before).abs() > 0.0001 {
                    changes.push(EffectChange {
                        update: ParameterUpdate::VibratoModWidth(self.vibrato.mod_width),
                        description: format!("Vibrato mod width → {:.1} ms", self.vibrato.mod_width * 1000.0),
                    });
                }

                let before = self.vibrato.mod_freq;
                ui.add(egui::Slider::new(&mut self.vibrato.mod_freq, 0.1..=20.0).text("Mod Freq (Hz)"));
                if (self.vibrato.mod_freq - before).abs() > 0.01 {
                    changes.push(EffectChange {
                        update: ParameterUpdate::VibratoModFreq(self.vibrato.mod_freq),
                        description: format!("Vibrato mod freq → {:.1} Hz", self.vibrato.mod_freq),
                    });
                }
            });
    }

    // --- Flanger ---

    fn render_flanger(&mut self, ui: &mut egui::Ui, changes: &mut Vec<EffectChange>) {
        let id = ui.make_persistent_id("fx_flanger");
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false)
            .show_header(ui, |ui| {
                ui.checkbox(&mut self.flanger.enabled, "");
                ui.strong("Flanger");
            })
            .body(|ui| {
                if !self.flanger.enabled { ui.disable(); }
                let before = self.flanger.delay_ms;
                ui.add(egui::Slider::new(&mut self.flanger.delay_ms, 1.0..=10.0).text("Delay (ms)"));
                if (self.flanger.delay_ms - before).abs() > 0.01 {
                    changes.push(EffectChange {
                        update: ParameterUpdate::FlangerDelayMs(self.flanger.delay_ms),
                        description: format!("Flanger delay → {:.1} ms", self.flanger.delay_ms),
                    });
                }

                let before = self.flanger.depth_ms;
                ui.add(egui::Slider::new(&mut self.flanger.depth_ms, 0.1..=10.0).text("Depth (ms)"));
                if (self.flanger.depth_ms - before).abs() > 0.01 {
                    changes.push(EffectChange {
                        update: ParameterUpdate::FlangerDepthMs(self.flanger.depth_ms),
                        description: format!("Flanger depth → {:.1} ms", self.flanger.depth_ms),
                    });
                }

                let before = self.flanger.rate_hz;
                ui.add(egui::Slider::new(&mut self.flanger.rate_hz, 0.01..=10.0).text("Rate (Hz)"));
                if (self.flanger.rate_hz - before).abs() > 0.001 {
                    changes.push(EffectChange {
                        update: ParameterUpdate::FlangerRateHz(self.flanger.rate_hz),
                        description: format!("Flanger rate → {:.2} Hz", self.flanger.rate_hz),
                    });
                }

                let before = self.flanger.mix;
                ui.add(egui::Slider::new(&mut self.flanger.mix, 0.0..=1.0).text("Mix"));
                if (self.flanger.mix - before).abs() > 0.001 {
                    changes.push(EffectChange {
                        update: ParameterUpdate::FlangerMix(self.flanger.mix),
                        description: format!("Flanger mix → {:.2}", self.flanger.mix),
                    });
                }

                let before = self.flanger.feedback;
                ui.add(egui::Slider::new(&mut self.flanger.feedback, 0.0..=0.99).text("Feedback"));
                if (self.flanger.feedback - before).abs() > 0.001 {
                    changes.push(EffectChange {
                        update: ParameterUpdate::FlangerFeedback(self.flanger.feedback),
                        description: format!("Flanger feedback → {:.2}", self.flanger.feedback),
                    });
                }
            });
    }

    // --- Delay ---

    fn render_delay(&mut self, ui: &mut egui::Ui, changes: &mut Vec<EffectChange>) {
        let id = ui.make_persistent_id("fx_delay");
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false)
            .show_header(ui, |ui| {
                ui.checkbox(&mut self.delay.enabled, "");
                ui.strong("Delay");
            })
            .body(|ui| {
                if !self.delay.enabled { ui.disable(); }
                let before = self.delay.mix;
                ui.add(egui::Slider::new(&mut self.delay.mix, 0.0..=1.0).text("Mix"));
                if (self.delay.mix - before).abs() > 0.001 {
                    changes.push(EffectChange {
                        update: ParameterUpdate::DelayMix(self.delay.mix),
                        description: format!("Delay mix → {:.2}", self.delay.mix),
                    });
                }

                let before = self.delay.decay;
                ui.add(egui::Slider::new(&mut self.delay.decay, 0.0..=1.0).text("Decay"));
                if (self.delay.decay - before).abs() > 0.001 {
                    changes.push(EffectChange {
                        update: ParameterUpdate::DelayDecay(self.delay.decay),
                        description: format!("Delay decay → {:.2}", self.delay.decay),
                    });
                }

                let before = self.delay.interval_ms;
                ui.add(egui::Slider::new(&mut self.delay.interval_ms, 1.0..=1000.0).text("Interval (ms)"));
                if (self.delay.interval_ms - before).abs() > 0.1 {
                    changes.push(EffectChange {
                        update: ParameterUpdate::DelayIntervalMs(self.delay.interval_ms),
                        description: format!("Delay interval → {:.1} ms", self.delay.interval_ms),
                    });
                }

                let before = self.delay.duration_ms;
                ui.add(egui::Slider::new(&mut self.delay.duration_ms, 1.0..=500.0).text("Duration (ms)"));
                if (self.delay.duration_ms - before).abs() > 0.1 {
                    changes.push(EffectChange {
                        update: ParameterUpdate::DelayDurationMs(self.delay.duration_ms),
                        description: format!("Delay duration → {:.1} ms", self.delay.duration_ms),
                    });
                }

                let before = self.delay.num_repeats;
                let mut repeats_f = self.delay.num_repeats as f32;
                ui.add(
                    egui::Slider::new(&mut repeats_f, 1.0..=16.0)
                        .step_by(1.0)
                        .text("Repeats"),
                );
                self.delay.num_repeats = repeats_f as usize;
                if self.delay.num_repeats != before {
                    changes.push(EffectChange {
                        update: ParameterUpdate::DelayNumRepeats(self.delay.num_repeats),
                        description: format!("Delay repeats → {}", self.delay.num_repeats),
                    });
                }
            });
    }

    // --- Chorus ---

    fn render_chorus(&mut self, ui: &mut egui::Ui, changes: &mut Vec<EffectChange>) {
        let id = ui.make_persistent_id("fx_chorus");
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false)
            .show_header(ui, |ui| {
                ui.checkbox(&mut self.chorus.enabled, "");
                ui.strong("Chorus");
            })
            .body(|ui| {
                if !self.chorus.enabled { ui.disable(); }

                // Voice count
                let before_count = self.chorus.chorus_count;
                let mut count_f = self.chorus.chorus_count as f32;
                ui.add(
                    egui::Slider::new(&mut count_f, 1.0..=6.0)
                        .step_by(1.0)
                        .text("Voices"),
                );
                let new_count = count_f as usize;
                if new_count != before_count {
                    self.chorus.resize_voices(new_count);
                    changes.push(EffectChange {
                        update: ParameterUpdate::ChorusCount(new_count),
                        description: format!("Chorus voices → {}", new_count),
                    });
                }

                // Dry gain
                let before = self.chorus.dry_gain;
                ui.add(egui::Slider::new(&mut self.chorus.dry_gain, 0.0..=1.0).text("Dry Gain"));
                if (self.chorus.dry_gain - before).abs() > 0.001 {
                    changes.push(EffectChange {
                        update: ParameterUpdate::ChorusDryGain(self.chorus.dry_gain),
                        description: format!("Chorus dry gain → {:.2}", self.chorus.dry_gain),
                    });
                }

                // Per-voice controls
                for v in 0..self.chorus.chorus_count {
                    ui.horizontal(|ui| {
                        ui.label(format!("V{}:", v + 1));

                        let before_gain = self.chorus.voice_gains[v];
                        ui.add(
                            egui::Slider::new(&mut self.chorus.voice_gains[v], 0.0..=1.0)
                                .text("Gain"),
                        );
                        if (self.chorus.voice_gains[v] - before_gain).abs() > 0.001 {
                            changes.push(EffectChange {
                                update: ParameterUpdate::ChorusVoiceGain {
                                    voice: v,
                                    gain: self.chorus.voice_gains[v],
                                },
                                description: format!(
                                    "Chorus voice {} gain → {:.2}",
                                    v + 1,
                                    self.chorus.voice_gains[v]
                                ),
                            });
                        }

                        let before_delay = self.chorus.voice_delays[v];
                        ui.add(
                            egui::Slider::new(&mut self.chorus.voice_delays[v], 0.001..=0.100)
                                .text("Delay (s)")
                                .custom_formatter(|val, _| format!("{:.1} ms", val * 1000.0)),
                        );
                        if (self.chorus.voice_delays[v] - before_delay).abs() > 0.0001 {
                            changes.push(EffectChange {
                                update: ParameterUpdate::ChorusVoiceDelay {
                                    voice: v,
                                    delay: self.chorus.voice_delays[v],
                                },
                                description: format!(
                                    "Chorus voice {} delay → {:.1} ms",
                                    v + 1,
                                    self.chorus.voice_delays[v] * 1000.0
                                ),
                            });
                        }
                    });
                }
            });
    }

    // --- Filter ---

    fn render_filter(&mut self, ui: &mut egui::Ui, changes: &mut Vec<EffectChange>) {
        let id = ui.make_persistent_id("fx_filter");
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false)
            .show_header(ui, |ui| {
                ui.checkbox(&mut self.filter.enabled, "");
                ui.strong("Filter");
            })
            .body(|ui| {
                if !self.filter.enabled { ui.disable(); }

                // Filter type selector
                ui.horizontal(|ui| {
                    ui.label("Type:");
                    let kinds = [
                        (FilterKind::LowPass, "LowPass"),
                        (FilterKind::HighPass, "HighPass"),
                        (FilterKind::BandPass, "BandPass"),
                        (FilterKind::Notch, "Notch"),
                    ];
                    for (kind, label) in &kinds {
                        let selected = self.filter.kind == *kind;
                        if ui.selectable_label(selected, *label).clicked() && !selected {
                            self.filter.kind = *kind;
                            changes.push(EffectChange {
                                update: ParameterUpdate::FilterType(*kind),
                                description: format!("Filter type → {}", label),
                            });
                        }
                    }
                });

                let before = self.filter.cutoff;
                ui.add(
                    egui::Slider::new(&mut self.filter.cutoff, 20.0..=20000.0)
                        .logarithmic(true)
                        .text("Cutoff (Hz)"),
                );
                if (self.filter.cutoff - before).abs() > 0.1 {
                    changes.push(EffectChange {
                        update: ParameterUpdate::FilterCutoff(self.filter.cutoff),
                        description: format!("Filter cutoff → {:.1} Hz", self.filter.cutoff),
                    });
                }

                let before = self.filter.resonance;
                ui.add(egui::Slider::new(&mut self.filter.resonance, 0.0..=20.0).text("Resonance"));
                if (self.filter.resonance - before).abs() > 0.01 {
                    changes.push(EffectChange {
                        update: ParameterUpdate::FilterResonance(self.filter.resonance),
                        description: format!("Filter resonance → {:.2}", self.filter.resonance),
                    });
                }

                let before = self.filter.mix;
                ui.add(egui::Slider::new(&mut self.filter.mix, 0.0..=1.0).text("Mix"));
                if (self.filter.mix - before).abs() > 0.001 {
                    changes.push(EffectChange {
                        update: ParameterUpdate::FilterMix(self.filter.mix),
                        description: format!("Filter mix → {:.2}", self.filter.mix),
                    });
                }
            });
    }

    // --- Equalizer ---

    fn render_equalizer(&mut self, ui: &mut egui::Ui, changes: &mut Vec<EffectChange>) {
        let id = ui.make_persistent_id("fx_eq");
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false)
            .show_header(ui, |ui| {
                ui.checkbox(&mut self.equalizer.enabled, "");
                ui.strong("Equalizer (8-band)");
            })
            .body(|ui| {
                if !self.equalizer.enabled { ui.disable(); }

                // EQ band sliders in a horizontal row
                let band_width = (ui.available_width() / 8.0).min(80.0);

                ui.horizontal(|ui| {
                    for band in 0..8 {
                        ui.vertical(|ui| {
                            ui.set_width(band_width);
                            ui.label(EQ_LABELS[band]);

                            let before = self.equalizer.gains[band];
                            ui.add(
                                egui::Slider::new(&mut self.equalizer.gains[band], -12.0..=12.0)
                                    .vertical()
                                    .text("dB")
                                    .custom_formatter(|v, _| format!("{:+.1}", v)),
                            );
                            if (self.equalizer.gains[band] - before).abs() > 0.05 {
                                changes.push(EffectChange {
                                    update: ParameterUpdate::EqualizerBandGain {
                                        band,
                                        gain_db: self.equalizer.gains[band],
                                    },
                                    description: format!(
                                        "EQ {} Hz → {:+.1} dB",
                                        EQ_LABELS[band], self.equalizer.gains[band]
                                    ),
                                });
                            }
                        });
                    }
                });

                // Reset button
                if ui.small_button("Reset All Bands").clicked() {
                    for band in 0..8 {
                        if self.equalizer.gains[band] != 0.0 {
                            self.equalizer.gains[band] = 0.0;
                            changes.push(EffectChange {
                                update: ParameterUpdate::EqualizerBandGain {
                                    band,
                                    gain_db: 0.0,
                                },
                                description: "EQ reset all bands → 0 dB".to_string(),
                            });
                        }
                    }
                }
            });
    }
}
