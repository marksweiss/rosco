use eframe::egui;
use serde::{Deserialize, Serialize};

use crate::tui::audio_bridge::{FilterKind, ParameterUpdate};

/// A parameter change with a human-readable description.
pub struct EffectChange {
    pub update: ParameterUpdate,
    pub description: String,
}

// --- Per-effect state structs ---

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct FilterState {
    pub enabled: bool,
    pub kind: FilterKind,
    pub cutoff: f32,
    pub resonance: f32,
    pub mix: f32,
    pub bandwidth: f32,
}

impl Default for FilterState {
    fn default() -> Self {
        Self {
            enabled: false,
            kind: FilterKind::LowPass,
            cutoff: 1000.0,
            resonance: 0.0,
            mix: 1.0,
            bandwidth: 500.0,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
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

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EqualizersState {
    pub equalizers: [EqualizerState; 8],
    #[serde(skip)]
    pub selected: usize,
}

impl Default for EqualizersState {
    fn default() -> Self {
        Self {
            equalizers: std::array::from_fn(|_| EqualizerState::default()),
            selected: 0,
        }
    }
}

impl EqualizersState {
    pub fn render(&mut self, ui: &mut egui::Ui) -> Vec<EffectChange> {
        let mut changes = Vec::new();

        // Tab bar (1-8)
        ui.horizontal_wrapped(|ui| {
            for i in 0..8 {
                if ui.selectable_label(self.selected == i, format!("{}", i + 1)).clicked() {
                    self.selected = i;
                }
            }
        });
        ui.separator();

        let chain = self.selected as u8;
        let eq = &mut self.equalizers[self.selected];

        // Enable checkbox
        let before_enabled = eq.enabled;
        ui.horizontal(|ui| {
            ui.checkbox(&mut eq.enabled, "");
            ui.strong("Equalizer (8-band)");
        });
        if eq.enabled != before_enabled {
            changes.push(EffectChange {
                update: ParameterUpdate::EqualizerEnabled { chain, enabled: eq.enabled },
                description: format!("EQ {} {}", chain + 1, if eq.enabled { "on" } else { "off" }),
            });
        }

        ui.add_space(4.0);

        ui.scope(|ui| {
            if !eq.enabled { ui.disable(); }

            // EQ band sliders in a horizontal row
            let band_width = (ui.available_width() / 8.0).min(80.0);

            ui.horizontal(|ui| {
                for band in 0..8 {
                    ui.vertical(|ui| {
                        ui.set_width(band_width);
                        ui.label(EQ_LABELS[band]);

                        let before = eq.gains[band];
                        ui.add(
                            egui::Slider::new(&mut eq.gains[band], -12.0..=12.0)
                                .vertical()
                                .text("dB")
                                .custom_formatter(|v, _| format!("{:+.1}", v)),
                        );
                        if (eq.gains[band] - before).abs() > 0.05 {
                            changes.push(EffectChange {
                                update: ParameterUpdate::EqualizerBandGain {
                                    chain,
                                    band,
                                    gain_db: eq.gains[band],
                                },
                                description: format!(
                                    "EQ {} {} Hz → {:+.1} dB",
                                    chain + 1, EQ_LABELS[band], eq.gains[band]
                                ),
                            });
                        }
                    });
                }
            });

            // Reset button
            if ui.small_button("Reset All Bands").clicked() {
                for band in 0..8 {
                    if eq.gains[band] != 0.0 {
                        eq.gains[band] = 0.0;
                        changes.push(EffectChange {
                            update: ParameterUpdate::EqualizerBandGain {
                                chain,
                                band,
                                gain_db: 0.0,
                            },
                            description: format!("EQ {} reset all bands → 0 dB", chain + 1),
                        });
                    }
                }
            }
        });

        changes
    }
}

// --- Effect tab selection ---

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EffectTab {
    Lfo,
    Tremolo,
    Vibrato,
    Flanger,
    Delay,
    Chorus,
    Filter,
}

impl Default for EffectTab {
    fn default() -> Self {
        Self::Lfo
    }
}

impl EffectTab {
    fn label(&self) -> &'static str {
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

    const ALL: [EffectTab; 7] = [
        Self::Lfo,
        Self::Tremolo,
        Self::Vibrato,
        Self::Flanger,
        Self::Delay,
        Self::Chorus,
        Self::Filter,
    ];
}

// --- Main effects rack ---

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EffectsRackState {
    pub lfo: LfoState,
    pub tremolo: TremoloState,
    pub vibrato: VibratoState,
    pub flanger: FlangerState,
    pub delay: DelayState,
    pub chorus: ChorusState,
    pub filter: FilterState,
    pub active_tab: EffectTab,
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
            active_tab: EffectTab::default(),
        }
    }
}

impl EffectsRackState {
    /// Render all effects EXCEPT the equalizer as a tabbed panel. Returns a list of parameter changes.
    pub fn render_effects(&mut self, ui: &mut egui::Ui) -> Vec<EffectChange> {
        let mut changes = Vec::new();

        // Tab bar
        ui.horizontal_wrapped(|ui| {
            for tab in EffectTab::ALL {
                let selected = self.active_tab == tab;
                if ui.selectable_label(selected, tab.label()).clicked() {
                    self.active_tab = tab;
                }
            }
        });

        ui.separator();
        ui.add_space(4.0);

        // Render active tab content
        match self.active_tab {
            EffectTab::Lfo => self.render_lfo(ui, &mut changes),
            EffectTab::Tremolo => self.render_tremolo(ui, &mut changes),
            EffectTab::Vibrato => self.render_vibrato(ui, &mut changes),
            EffectTab::Flanger => self.render_flanger(ui, &mut changes),
            EffectTab::Delay => self.render_delay(ui, &mut changes),
            EffectTab::Chorus => self.render_chorus(ui, &mut changes),
            EffectTab::Filter => self.render_filter(ui, &mut changes),
        }

        changes
    }

    // --- LFO ---

    fn render_lfo(&mut self, ui: &mut egui::Ui, changes: &mut Vec<EffectChange>) {
        ui.checkbox(&mut self.lfo.enabled, "Enabled");
        ui.add_space(4.0);
        ui.scope(|ui| {
            if !self.lfo.enabled { ui.disable(); }
            changes.extend(render_lfo_params(&mut self.lfo, ui));
        });
    }

    // --- Tremolo ---

    fn render_tremolo(&mut self, ui: &mut egui::Ui, changes: &mut Vec<EffectChange>) {
        ui.checkbox(&mut self.tremolo.enabled, "Enabled");
        ui.add_space(4.0);
        ui.scope(|ui| {
            if !self.tremolo.enabled { ui.disable(); }
            changes.extend(render_tremolo_params(&mut self.tremolo, ui));
        });
    }

    // --- Vibrato ---

    fn render_vibrato(&mut self, ui: &mut egui::Ui, changes: &mut Vec<EffectChange>) {
        ui.checkbox(&mut self.vibrato.enabled, "Enabled");
        ui.add_space(4.0);
        ui.scope(|ui| {
            if !self.vibrato.enabled { ui.disable(); }
            changes.extend(render_vibrato_params(&mut self.vibrato, ui));
        });
    }

    // --- Flanger ---

    fn render_flanger(&mut self, ui: &mut egui::Ui, changes: &mut Vec<EffectChange>) {
        ui.checkbox(&mut self.flanger.enabled, "Enabled");
        ui.add_space(4.0);
        ui.scope(|ui| {
            if !self.flanger.enabled { ui.disable(); }
            changes.extend(render_flanger_params(&mut self.flanger, ui));
        });
    }

    // --- Delay ---

    fn render_delay(&mut self, ui: &mut egui::Ui, changes: &mut Vec<EffectChange>) {
        ui.checkbox(&mut self.delay.enabled, "Enabled");
        ui.add_space(4.0);
        ui.scope(|ui| {
            if !self.delay.enabled { ui.disable(); }
            changes.extend(render_delay_params(&mut self.delay, ui));
        });
    }

    // --- Chorus ---

    fn render_chorus(&mut self, ui: &mut egui::Ui, changes: &mut Vec<EffectChange>) {
        ui.checkbox(&mut self.chorus.enabled, "Enabled");
        ui.add_space(4.0);
        ui.scope(|ui| {
            if !self.chorus.enabled { ui.disable(); }
            changes.extend(render_chorus_params(&mut self.chorus, ui));
        });
    }

    // --- Filter ---

    fn render_filter(&mut self, ui: &mut egui::Ui, changes: &mut Vec<EffectChange>) {
        ui.checkbox(&mut self.filter.enabled, "Enabled");
        ui.add_space(4.0);
        ui.scope(|ui| {
            if !self.filter.enabled { ui.disable(); }
            changes.extend(render_filter_params(&mut self.filter, ui));
        });
    }

}

// --- Standalone parameter render functions (reusable from effect chains) ---

pub fn render_lfo_params(state: &mut LfoState, ui: &mut egui::Ui) -> Vec<EffectChange> {
    let mut changes = Vec::new();

    let before_freq = state.frequency;
    ui.add(
        egui::Slider::new(&mut state.frequency, 0.01..=22050.0)
            .logarithmic(true)
            .text("Freq (Hz)"),
    );
    if (state.frequency - before_freq).abs() > 0.001 {
        changes.push(EffectChange {
            update: ParameterUpdate::LfoFrequency(state.frequency),
            description: format!("LFO frequency → {:.1} Hz", state.frequency),
        });
    }

    let before_amp = state.amplitude;
    ui.add(egui::Slider::new(&mut state.amplitude, 0.0..=1.0).text("Amplitude"));
    if (state.amplitude - before_amp).abs() > 0.001 {
        changes.push(EffectChange {
            update: ParameterUpdate::LfoAmplitude(state.amplitude),
            description: format!("LFO amplitude → {:.2}", state.amplitude),
        });
    }

    changes
}

pub fn render_tremolo_params(state: &mut TremoloState, ui: &mut egui::Ui) -> Vec<EffectChange> {
    let mut changes = Vec::new();

    let before = state.mod_freq;
    ui.add(egui::Slider::new(&mut state.mod_freq, 0.1..=20.0).text("Mod Freq (Hz)"));
    if (state.mod_freq - before).abs() > 0.01 {
        changes.push(EffectChange {
            update: ParameterUpdate::TremoloModFreq(state.mod_freq),
            description: format!("Tremolo mod freq → {:.1} Hz", state.mod_freq),
        });
    }

    let before = state.mod_depth;
    ui.add(egui::Slider::new(&mut state.mod_depth, 0.0..=1.0).text("Mod Depth"));
    if (state.mod_depth - before).abs() > 0.001 {
        changes.push(EffectChange {
            update: ParameterUpdate::TremoloModDepth(state.mod_depth),
            description: format!("Tremolo mod depth → {:.2}", state.mod_depth),
        });
    }

    changes
}

pub fn render_vibrato_params(state: &mut VibratoState, ui: &mut egui::Ui) -> Vec<EffectChange> {
    let mut changes = Vec::new();

    let before = state.avg_delay;
    ui.add(
        egui::Slider::new(&mut state.avg_delay, 0.001..=0.020)
            .text("Avg Delay (s)")
            .custom_formatter(|v, _| format!("{:.1} ms", v * 1000.0)),
    );
    if (state.avg_delay - before).abs() > 0.0001 {
        changes.push(EffectChange {
            update: ParameterUpdate::VibratoAvgDelay(state.avg_delay),
            description: format!("Vibrato avg delay → {:.1} ms", state.avg_delay * 1000.0),
        });
    }

    let before = state.mod_width;
    ui.add(
        egui::Slider::new(&mut state.mod_width, 0.001..=0.010)
            .text("Mod Width (s)")
            .custom_formatter(|v, _| format!("{:.1} ms", v * 1000.0)),
    );
    if (state.mod_width - before).abs() > 0.0001 {
        changes.push(EffectChange {
            update: ParameterUpdate::VibratoModWidth(state.mod_width),
            description: format!("Vibrato mod width → {:.1} ms", state.mod_width * 1000.0),
        });
    }

    let before = state.mod_freq;
    ui.add(egui::Slider::new(&mut state.mod_freq, 0.1..=20.0).text("Mod Freq (Hz)"));
    if (state.mod_freq - before).abs() > 0.01 {
        changes.push(EffectChange {
            update: ParameterUpdate::VibratoModFreq(state.mod_freq),
            description: format!("Vibrato mod freq → {:.1} Hz", state.mod_freq),
        });
    }

    changes
}

pub fn render_flanger_params(state: &mut FlangerState, ui: &mut egui::Ui) -> Vec<EffectChange> {
    let mut changes = Vec::new();

    let before = state.delay_ms;
    ui.add(egui::Slider::new(&mut state.delay_ms, 1.0..=10.0).text("Delay (ms)"));
    if (state.delay_ms - before).abs() > 0.01 {
        changes.push(EffectChange {
            update: ParameterUpdate::FlangerDelayMs(state.delay_ms),
            description: format!("Flanger delay → {:.1} ms", state.delay_ms),
        });
    }

    let before = state.depth_ms;
    ui.add(egui::Slider::new(&mut state.depth_ms, 0.1..=10.0).text("Depth (ms)"));
    if (state.depth_ms - before).abs() > 0.01 {
        changes.push(EffectChange {
            update: ParameterUpdate::FlangerDepthMs(state.depth_ms),
            description: format!("Flanger depth → {:.1} ms", state.depth_ms),
        });
    }

    let before = state.rate_hz;
    ui.add(egui::Slider::new(&mut state.rate_hz, 0.01..=10.0).text("Rate (Hz)"));
    if (state.rate_hz - before).abs() > 0.001 {
        changes.push(EffectChange {
            update: ParameterUpdate::FlangerRateHz(state.rate_hz),
            description: format!("Flanger rate → {:.2} Hz", state.rate_hz),
        });
    }

    let before = state.mix;
    ui.add(egui::Slider::new(&mut state.mix, 0.0..=1.0).text("Mix"));
    if (state.mix - before).abs() > 0.001 {
        changes.push(EffectChange {
            update: ParameterUpdate::FlangerMix(state.mix),
            description: format!("Flanger mix → {:.2}", state.mix),
        });
    }

    let before = state.feedback;
    ui.add(egui::Slider::new(&mut state.feedback, 0.0..=0.99).text("Feedback"));
    if (state.feedback - before).abs() > 0.001 {
        changes.push(EffectChange {
            update: ParameterUpdate::FlangerFeedback(state.feedback),
            description: format!("Flanger feedback → {:.2}", state.feedback),
        });
    }

    changes
}

pub fn render_delay_params(state: &mut DelayState, ui: &mut egui::Ui) -> Vec<EffectChange> {
    let mut changes = Vec::new();

    let before = state.mix;
    ui.add(egui::Slider::new(&mut state.mix, 0.0..=1.0).text("Mix"));
    if (state.mix - before).abs() > 0.001 {
        changes.push(EffectChange {
            update: ParameterUpdate::DelayMix(state.mix),
            description: format!("Delay mix → {:.2}", state.mix),
        });
    }

    let before = state.decay;
    ui.add(egui::Slider::new(&mut state.decay, 0.0..=1.0).text("Decay"));
    if (state.decay - before).abs() > 0.001 {
        changes.push(EffectChange {
            update: ParameterUpdate::DelayDecay(state.decay),
            description: format!("Delay decay → {:.2}", state.decay),
        });
    }

    let before = state.interval_ms;
    ui.add(egui::Slider::new(&mut state.interval_ms, 1.0..=1000.0).text("Interval (ms)"));
    if (state.interval_ms - before).abs() > 0.1 {
        changes.push(EffectChange {
            update: ParameterUpdate::DelayIntervalMs(state.interval_ms),
            description: format!("Delay interval → {:.1} ms", state.interval_ms),
        });
    }

    let before = state.duration_ms;
    ui.add(egui::Slider::new(&mut state.duration_ms, 1.0..=500.0).text("Duration (ms)"));
    if (state.duration_ms - before).abs() > 0.1 {
        changes.push(EffectChange {
            update: ParameterUpdate::DelayDurationMs(state.duration_ms),
            description: format!("Delay duration → {:.1} ms", state.duration_ms),
        });
    }

    let before = state.num_repeats;
    let mut repeats_f = state.num_repeats as f32;
    ui.add(
        egui::Slider::new(&mut repeats_f, 1.0..=16.0)
            .step_by(1.0)
            .text("Repeats"),
    );
    state.num_repeats = repeats_f as usize;
    if state.num_repeats != before {
        changes.push(EffectChange {
            update: ParameterUpdate::DelayNumRepeats(state.num_repeats),
            description: format!("Delay repeats → {}", state.num_repeats),
        });
    }

    changes
}

pub fn render_chorus_params(state: &mut ChorusState, ui: &mut egui::Ui) -> Vec<EffectChange> {
    let mut changes = Vec::new();

    // Voice count
    let before_count = state.chorus_count;
    let mut count_f = state.chorus_count as f32;
    ui.add(
        egui::Slider::new(&mut count_f, 1.0..=6.0)
            .step_by(1.0)
            .text("Voices"),
    );
    let new_count = count_f as usize;
    if new_count != before_count {
        state.resize_voices(new_count);
        changes.push(EffectChange {
            update: ParameterUpdate::ChorusCount(new_count),
            description: format!("Chorus voices → {}", new_count),
        });
    }

    // Dry gain
    let before = state.dry_gain;
    ui.add(egui::Slider::new(&mut state.dry_gain, 0.0..=1.0).text("Dry Gain"));
    if (state.dry_gain - before).abs() > 0.001 {
        changes.push(EffectChange {
            update: ParameterUpdate::ChorusDryGain(state.dry_gain),
            description: format!("Chorus dry gain → {:.2}", state.dry_gain),
        });
    }

    // Per-voice controls
    for v in 0..state.chorus_count {
        ui.horizontal(|ui| {
            ui.label(format!("V{}:", v + 1));

            let before_gain = state.voice_gains[v];
            ui.add(
                egui::Slider::new(&mut state.voice_gains[v], 0.0..=1.0)
                    .text("Gain"),
            );
            if (state.voice_gains[v] - before_gain).abs() > 0.001 {
                changes.push(EffectChange {
                    update: ParameterUpdate::ChorusVoiceGain {
                        voice: v,
                        gain: state.voice_gains[v],
                    },
                    description: format!(
                        "Chorus voice {} gain → {:.2}",
                        v + 1,
                        state.voice_gains[v]
                    ),
                });
            }

            let before_delay = state.voice_delays[v];
            ui.add(
                egui::Slider::new(&mut state.voice_delays[v], 0.001..=0.100)
                    .text("Delay (s)")
                    .custom_formatter(|val, _| format!("{:.1} ms", val * 1000.0)),
            );
            if (state.voice_delays[v] - before_delay).abs() > 0.0001 {
                changes.push(EffectChange {
                    update: ParameterUpdate::ChorusVoiceDelay {
                        voice: v,
                        delay: state.voice_delays[v],
                    },
                    description: format!(
                        "Chorus voice {} delay → {:.1} ms",
                        v + 1,
                        state.voice_delays[v] * 1000.0
                    ),
                });
            }
        });
    }

    changes
}

pub fn render_filter_params(state: &mut FilterState, ui: &mut egui::Ui) -> Vec<EffectChange> {
    let mut changes = Vec::new();

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
            let selected = state.kind == *kind;
            if ui.selectable_label(selected, *label).clicked() && !selected {
                state.kind = *kind;
                changes.push(EffectChange {
                    update: ParameterUpdate::FilterType(*kind),
                    description: format!("Filter type → {}", label),
                });
            }
        }
    });

    let is_band = matches!(state.kind, FilterKind::BandPass | FilterKind::Notch);
    let freq_label = if is_band { "Center Freq (Hz)" } else { "Cutoff (Hz)" };

    let before = state.cutoff;
    ui.add(
        egui::Slider::new(&mut state.cutoff, 20.0..=20000.0)
            .logarithmic(true)
            .text(freq_label),
    );
    if (state.cutoff - before).abs() > 0.1 {
        changes.push(EffectChange {
            update: ParameterUpdate::FilterCutoff(state.cutoff),
            description: format!("Filter {} → {:.1} Hz", freq_label.to_lowercase(), state.cutoff),
        });
    }

    if is_band {
        let before = state.bandwidth;
        ui.add(
            egui::Slider::new(&mut state.bandwidth, 10.0..=10000.0)
                .logarithmic(true)
                .text("Bandwidth (Hz)"),
        );
        if (state.bandwidth - before).abs() > 0.1 {
            changes.push(EffectChange {
                update: ParameterUpdate::FilterBandwidth(state.bandwidth),
                description: format!("Filter bandwidth → {:.1} Hz", state.bandwidth),
            });
        }
    }

    let before = state.resonance;
    ui.add(egui::Slider::new(&mut state.resonance, 0.0..=20.0).text("Resonance (Q)"));
    if (state.resonance - before).abs() > 0.01 {
        changes.push(EffectChange {
            update: ParameterUpdate::FilterResonance(state.resonance),
            description: format!("Filter resonance → {:.2}", state.resonance),
        });
    }

    let before = state.mix;
    ui.add(egui::Slider::new(&mut state.mix, 0.0..=1.0).text("Mix"));
    if (state.mix - before).abs() > 0.001 {
        changes.push(EffectChange {
            update: ParameterUpdate::FilterMix(state.mix),
            description: format!("Filter mix → {:.2}", state.mix),
        });
    }

    changes
}
