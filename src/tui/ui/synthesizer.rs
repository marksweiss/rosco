use crate::tui::ui::widgets::{LinearSlider, LogSlider, WaveformSelector, FilterTypeSelector};
use crate::tui::ui::widgets::selector::FilterType;
use crate::tui::audio_bridge::ParameterUpdate;
use crate::audio_gen::Waveform;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Widget},
};

#[derive(Debug)]
pub struct SynthesizerPanel {
    pub oscillator: OscillatorControls,
    pub filter: FilterControls,
    pub envelope: EnvelopeControls,
    pub effects: EffectsControls,
    pub current_section: SynthesizerSubSection,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SynthesizerSubSection {
    Oscillator(OscillatorSubSection),
    Filter(FilterSubSection),
    Envelope(EnvelopeSubSection),
    Effects(EffectsSubSection),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EffectsSubSection {
    // Delay
    DelayTime,
    DelayFeedback,
    DelayMix,
    // Flanger
    FlangerRate,
    FlangerDepth,
    FlangerMix,
    // LFO
    LfoRate,
    LfoDepth,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OscillatorSubSection {
    Waveform,
    Volume,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FilterSubSection {
    Type,
    Frequency, // Cutoff for LP/HP, Center for BP/Notch
    Bandwidth, // Only for BP/Notch
    Resonance,
    Mix,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnvelopeSubSection {
    AttackTime,
    AttackLevel,
    DecayTime,
    DecayLevel,
    SustainTime,
    SustainLevel,
    ReleaseTime,
    ReleaseLevel,
}

#[derive(Debug)]
pub struct OscillatorControls {
    pub waveform_selector: WaveformSelector,
    pub volume_slider: LinearSlider,
    pub sub_focus: OscillatorSubSection,
}

#[derive(Debug)]
pub struct FilterControls {
    pub filter_type: FilterTypeSelector,
    pub frequency_slider: LogSlider, // Cutoff for LP/HP, Center for BP/Notch
    pub bandwidth_slider: LogSlider, // Only for BP/Notch
    pub resonance_slider: LinearSlider,
    pub mix_slider: LinearSlider,
    pub sub_focus: FilterSubSection,
}

#[derive(Debug)]
pub struct EnvelopeControls {
    pub attack_time_slider: LinearSlider,
    pub attack_level_slider: LinearSlider,
    pub decay_time_slider: LinearSlider,
    pub decay_level_slider: LinearSlider,
    pub sustain_time_slider: LinearSlider,
    pub sustain_level_slider: LinearSlider,
    pub release_time_slider: LinearSlider,
    pub release_level_slider: LinearSlider,
    pub sub_focus: EnvelopeSubSection,
}

#[derive(Debug)]
pub struct EffectsControls {
    // Delay effect controls
    pub delay_time_slider: LinearSlider,     // 0.0-1.0s
    pub delay_feedback_slider: LinearSlider, // 0.0-1.0
    pub delay_mix_slider: LinearSlider,      // 0.0-1.0

    // Flanger effect controls
    pub flanger_rate_slider: LinearSlider,   // 0.1-10.0 Hz
    pub flanger_depth_slider: LinearSlider,  // 0.0-1.0
    pub flanger_mix_slider: LinearSlider,    // 0.0-1.0

    // LFO controls
    pub lfo_rate_slider: LinearSlider,       // 0.1-20.0 Hz
    pub lfo_depth_slider: LinearSlider,      // 0.0-1.0

    pub sub_focus: EffectsSubSection,
}

impl Default for SynthesizerPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl SynthesizerPanel {
    pub fn new() -> Self {
        Self {
            oscillator: OscillatorControls::new(),
            filter: FilterControls::new(),
            envelope: EnvelopeControls::new(),
            effects: EffectsControls::new(),
            current_section: SynthesizerSubSection::Oscillator(OscillatorSubSection::Waveform),
        }
    }
    
    pub fn handle_input(&mut self, key: KeyEvent) -> Vec<ParameterUpdate> {
        let mut updates = Vec::new();
        
        match self.current_section {
            SynthesizerSubSection::Oscillator(osc_section) => {
                match key.code {
                    KeyCode::Up | KeyCode::Down => {
                        let new_section = match osc_section {
                            OscillatorSubSection::Waveform => OscillatorSubSection::Volume,
                            OscillatorSubSection::Volume => OscillatorSubSection::Waveform,
                        };
                        self.current_section = SynthesizerSubSection::Oscillator(new_section);
                        // Keep the individual control's sub_focus in sync
                        self.oscillator.sub_focus = new_section;
                    }
                    KeyCode::Left | KeyCode::Right => {
                        if let Some(update) = self.handle_oscillator_adjustment(osc_section, key.code) {
                            updates.push(update);
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(update) = self.handle_oscillator_activation(osc_section) {
                            updates.push(update);
                        }
                    }
                    _ => {}
                }
            }
            SynthesizerSubSection::Filter(filter_section) => {
                match key.code {
                    KeyCode::Up | KeyCode::Down => {
                        let new_section = self.get_next_filter_section(filter_section);
                        self.current_section = SynthesizerSubSection::Filter(new_section);
                        // Keep the individual control's sub_focus in sync
                        self.filter.sub_focus = new_section;
                    }
                    KeyCode::Left | KeyCode::Right => {
                        if let Some(update) = self.handle_filter_adjustment(filter_section, key.code) {
                            updates.push(update);
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(update) = self.handle_filter_activation(filter_section) {
                            updates.push(update);
                        }
                    }
                    _ => {}
                }
            }
            SynthesizerSubSection::Envelope(env_section) => {
                match key.code {
                    KeyCode::Up | KeyCode::Down => {
                        let new_section = self.get_next_envelope_section(env_section);
                        self.current_section = SynthesizerSubSection::Envelope(new_section);
                        // Keep the individual control's sub_focus in sync
                        self.envelope.sub_focus = new_section;
                    }
                    KeyCode::Left | KeyCode::Right => {
                        if let Some(update) = self.handle_envelope_adjustment(env_section, key.code) {
                            updates.push(update);
                        }
                    }
                    _ => {}
                }
            }
            SynthesizerSubSection::Effects(fx_section) => {
                match key.code {
                    KeyCode::Up | KeyCode::Down => {
                        let new_section = self.get_next_effects_section(fx_section);
                        self.current_section = SynthesizerSubSection::Effects(new_section);
                        self.effects.sub_focus = new_section;
                    }
                    KeyCode::Left | KeyCode::Right => {
                        if let Some(update) = self.handle_effects_adjustment(fx_section, key.code) {
                            updates.push(update);
                        }
                    }
                    _ => {}
                }
            }
        }
        
        updates
    }
    
    fn get_next_filter_section(&self, current: FilterSubSection) -> FilterSubSection {
        let filter_type = self.filter.filter_type.selected_filter();
        let has_bandwidth = matches!(filter_type, FilterType::BandPass | FilterType::Notch);
        
        match current {
            FilterSubSection::Type => FilterSubSection::Frequency,
            FilterSubSection::Frequency => {
                if has_bandwidth {
                    FilterSubSection::Bandwidth
                } else {
                    FilterSubSection::Resonance
                }
            }
            FilterSubSection::Bandwidth => FilterSubSection::Resonance,
            FilterSubSection::Resonance => FilterSubSection::Mix,
            FilterSubSection::Mix => FilterSubSection::Type,
        }
    }
    
    pub fn handle_fine_adjustment(&mut self, increase: bool) -> Option<ParameterUpdate> {
        match self.current_section {
            SynthesizerSubSection::Oscillator(OscillatorSubSection::Volume) => {
                let delta = if increase { 0.01 } else { -0.01 };
                self.oscillator.volume_slider.adjust(delta);
                Some(ParameterUpdate::OscillatorVolume(
                    self.oscillator.volume_slider.value
                ))
            }
            SynthesizerSubSection::Filter(FilterSubSection::Frequency) => {
                let _delta = if increase { 0.05 } else { -0.05 };
                self.filter.frequency_slider.adjust_log(if increase { 1.05 } else { 0.95 });
                Some(self.get_filter_frequency_update())
            }
            SynthesizerSubSection::Filter(FilterSubSection::Bandwidth) => {
                let _delta = if increase { 0.05 } else { -0.05 };
                self.filter.bandwidth_slider.adjust_log(if increase { 1.05 } else { 0.95 });
                Some(self.get_filter_bandwidth_update())
            }
            SynthesizerSubSection::Filter(FilterSubSection::Resonance) => {
                let delta = if increase { 0.01 } else { -0.01 };
                self.filter.resonance_slider.adjust(delta);
                Some(ParameterUpdate::FilterResonance(
                    self.filter.resonance_slider.value
                ))
            }
            SynthesizerSubSection::Filter(FilterSubSection::Mix) => {
                let delta = if increase { 0.01 } else { -0.01 };
                self.filter.mix_slider.adjust(delta);
                Some(ParameterUpdate::FilterMix(
                    self.filter.mix_slider.value
                ))
            }
            SynthesizerSubSection::Envelope(env_section) => {
                let delta = if increase { 0.005 } else { -0.005 };
                match env_section {
                    EnvelopeSubSection::AttackTime => {
                        self.envelope.attack_time_slider.adjust(delta);
                        Some(ParameterUpdate::EnvelopeAttackTime(
                            self.envelope.attack_time_slider.value
                        ))
                    }
                    EnvelopeSubSection::AttackLevel => {
                        self.envelope.attack_level_slider.adjust(delta);
                        Some(ParameterUpdate::EnvelopeAttackLevel(
                            self.envelope.attack_level_slider.value
                        ))
                    }
                    EnvelopeSubSection::DecayTime => {
                        self.envelope.decay_time_slider.adjust(delta);
                        Some(ParameterUpdate::EnvelopeDecayTime(
                            self.envelope.decay_time_slider.value
                        ))
                    }
                    EnvelopeSubSection::DecayLevel => {
                        self.envelope.decay_level_slider.adjust(delta);
                        Some(ParameterUpdate::EnvelopeDecayLevel(
                            self.envelope.decay_level_slider.value
                        ))
                    }
                    EnvelopeSubSection::SustainTime => {
                        self.envelope.sustain_time_slider.adjust(delta);
                        Some(ParameterUpdate::EnvelopeSustainTime(
                            self.envelope.sustain_time_slider.value
                        ))
                    }
                    EnvelopeSubSection::SustainLevel => {
                        self.envelope.sustain_level_slider.adjust(delta);
                        Some(ParameterUpdate::EnvelopeSustainLevel(
                            self.envelope.sustain_level_slider.value
                        ))
                    }
                    EnvelopeSubSection::ReleaseTime => {
                        self.envelope.release_time_slider.adjust(delta);
                        Some(ParameterUpdate::EnvelopeReleaseTime(
                            self.envelope.release_time_slider.value
                        ))
                    }
                    EnvelopeSubSection::ReleaseLevel => {
                        self.envelope.release_level_slider.adjust(delta);
                        Some(ParameterUpdate::EnvelopeReleaseLevel(
                            self.envelope.release_level_slider.value
                        ))
                    }
                }
            }
            _ => None
        }
    }
    
    fn get_filter_frequency_update(&self) -> ParameterUpdate {
        let filter_type = self.filter.filter_type.selected_filter();
        match filter_type {
            FilterType::LowPass | FilterType::HighPass => {
                ParameterUpdate::FilterCutoff(self.filter.frequency_slider.value)
            }
            FilterType::BandPass | FilterType::Notch => {
                ParameterUpdate::FilterCenterFrequency(self.filter.frequency_slider.value)
            }
        }
    }
    
    fn get_filter_bandwidth_update(&self) -> ParameterUpdate {
        ParameterUpdate::FilterBandwidth(self.filter.bandwidth_slider.value)
    }
    
    fn get_next_envelope_section(&self, current: EnvelopeSubSection) -> EnvelopeSubSection {
        match current {
            EnvelopeSubSection::AttackTime => EnvelopeSubSection::AttackLevel,
            EnvelopeSubSection::AttackLevel => EnvelopeSubSection::DecayTime,
            EnvelopeSubSection::DecayTime => EnvelopeSubSection::DecayLevel,
            EnvelopeSubSection::DecayLevel => EnvelopeSubSection::SustainTime,
            EnvelopeSubSection::SustainTime => EnvelopeSubSection::SustainLevel,
            EnvelopeSubSection::SustainLevel => EnvelopeSubSection::ReleaseTime,
            EnvelopeSubSection::ReleaseTime => EnvelopeSubSection::ReleaseLevel,
            EnvelopeSubSection::ReleaseLevel => EnvelopeSubSection::AttackTime,
        }
    }
    
    fn handle_envelope_adjustment(&mut self, env_section: EnvelopeSubSection, key_code: KeyCode) -> Option<ParameterUpdate> {
        let delta = match key_code {
            KeyCode::Left => -0.01,
            KeyCode::Right => 0.01,
            _ => return None,
        };
        
        match env_section {
            EnvelopeSubSection::AttackTime => {
                self.envelope.attack_time_slider.adjust(delta);
                Some(ParameterUpdate::EnvelopeAttackTime(
                    self.envelope.attack_time_slider.value
                ))
            }
            EnvelopeSubSection::AttackLevel => {
                self.envelope.attack_level_slider.adjust(delta);
                Some(ParameterUpdate::EnvelopeAttackLevel(
                    self.envelope.attack_level_slider.value
                ))
            }
            EnvelopeSubSection::DecayTime => {
                self.envelope.decay_time_slider.adjust(delta);
                Some(ParameterUpdate::EnvelopeDecayTime(
                    self.envelope.decay_time_slider.value
                ))
            }
            EnvelopeSubSection::DecayLevel => {
                self.envelope.decay_level_slider.adjust(delta);
                Some(ParameterUpdate::EnvelopeDecayLevel(
                    self.envelope.decay_level_slider.value
                ))
            }
            EnvelopeSubSection::SustainTime => {
                self.envelope.sustain_time_slider.adjust(delta);
                Some(ParameterUpdate::EnvelopeSustainTime(
                    self.envelope.sustain_time_slider.value
                ))
            }
            EnvelopeSubSection::SustainLevel => {
                self.envelope.sustain_level_slider.adjust(delta);
                Some(ParameterUpdate::EnvelopeSustainLevel(
                    self.envelope.sustain_level_slider.value
                ))
            }
            EnvelopeSubSection::ReleaseTime => {
                self.envelope.release_time_slider.adjust(delta);
                Some(ParameterUpdate::EnvelopeReleaseTime(
                    self.envelope.release_time_slider.value
                ))
            }
            EnvelopeSubSection::ReleaseLevel => {
                self.envelope.release_level_slider.adjust(delta);
                Some(ParameterUpdate::EnvelopeReleaseLevel(
                    self.envelope.release_level_slider.value
                ))
            }
        }
    }

    fn get_next_effects_section(&self, current: EffectsSubSection) -> EffectsSubSection {
        match current {
            EffectsSubSection::DelayTime => EffectsSubSection::DelayFeedback,
            EffectsSubSection::DelayFeedback => EffectsSubSection::DelayMix,
            EffectsSubSection::DelayMix => EffectsSubSection::FlangerRate,
            EffectsSubSection::FlangerRate => EffectsSubSection::FlangerDepth,
            EffectsSubSection::FlangerDepth => EffectsSubSection::FlangerMix,
            EffectsSubSection::FlangerMix => EffectsSubSection::LfoRate,
            EffectsSubSection::LfoRate => EffectsSubSection::LfoDepth,
            EffectsSubSection::LfoDepth => EffectsSubSection::DelayTime,
        }
    }

    fn handle_effects_adjustment(&mut self, fx_section: EffectsSubSection, key_code: KeyCode) -> Option<ParameterUpdate> {
        let delta = match key_code {
            KeyCode::Left => -0.05,
            KeyCode::Right => 0.05,
            _ => return None,
        };

        match fx_section {
            EffectsSubSection::DelayTime => {
                self.effects.delay_time_slider.adjust(delta);
                Some(ParameterUpdate::DelayTime(self.effects.delay_time_slider.value))
            }
            EffectsSubSection::DelayFeedback => {
                self.effects.delay_feedback_slider.adjust(delta);
                Some(ParameterUpdate::DelayFeedback(self.effects.delay_feedback_slider.value))
            }
            EffectsSubSection::DelayMix => {
                self.effects.delay_mix_slider.adjust(delta);
                Some(ParameterUpdate::DelayMix(self.effects.delay_mix_slider.value))
            }
            EffectsSubSection::FlangerRate => {
                self.effects.flanger_rate_slider.adjust(delta);
                Some(ParameterUpdate::FlangerRate(self.effects.flanger_rate_slider.value))
            }
            EffectsSubSection::FlangerDepth => {
                self.effects.flanger_depth_slider.adjust(delta);
                Some(ParameterUpdate::FlangerDepth(self.effects.flanger_depth_slider.value))
            }
            EffectsSubSection::FlangerMix => {
                self.effects.flanger_mix_slider.adjust(delta);
                Some(ParameterUpdate::FlangerMix(self.effects.flanger_mix_slider.value))
            }
            EffectsSubSection::LfoRate => {
                self.effects.lfo_rate_slider.adjust(delta);
                Some(ParameterUpdate::LfoRate(self.effects.lfo_rate_slider.value))
            }
            EffectsSubSection::LfoDepth => {
                self.effects.lfo_depth_slider.adjust(delta);
                Some(ParameterUpdate::LfoDepth(self.effects.lfo_depth_slider.value))
            }
        }
    }

    fn handle_oscillator_adjustment(&mut self, osc_section: OscillatorSubSection, key_code: KeyCode) -> Option<ParameterUpdate> {
        match osc_section {
            OscillatorSubSection::Waveform => {
                match key_code {
                    KeyCode::Left => self.oscillator.waveform_selector.previous(),
                    KeyCode::Right => self.oscillator.waveform_selector.next(),
                    _ => {}
                }
                Some(ParameterUpdate::OscillatorWaveform(
                    self.oscillator.waveform_selector.selected_waveform()
                ))
            }
            OscillatorSubSection::Volume => {
                match key_code {
                    KeyCode::Left => {
                        self.oscillator.volume_slider.adjust(-0.05);
                        Some(ParameterUpdate::OscillatorVolume(
                            self.oscillator.volume_slider.value
                        ))
                    }
                    KeyCode::Right => {
                        self.oscillator.volume_slider.adjust(0.05);
                        Some(ParameterUpdate::OscillatorVolume(
                            self.oscillator.volume_slider.value
                        ))
                    }
                    _ => None
                }
            }
        }
    }
    
    fn handle_filter_adjustment(&mut self, filter_section: FilterSubSection, key_code: KeyCode) -> Option<ParameterUpdate> {
        match filter_section {
            FilterSubSection::Type => {
                match key_code {
                    KeyCode::Left => self.filter.filter_type.previous(),
                    KeyCode::Right => self.filter.filter_type.next(),
                    _ => {}
                }
                Some(ParameterUpdate::FilterType(
                    self.filter.filter_type.selected_filter().clone()
                ))
            }
            FilterSubSection::Frequency => {
                match key_code {
                    KeyCode::Left => {
                        self.filter.frequency_slider.adjust_log(0.95);
                        Some(self.get_filter_frequency_update())
                    }
                    KeyCode::Right => {
                        self.filter.frequency_slider.adjust_log(1.05);
                        Some(self.get_filter_frequency_update())
                    }
                    _ => None
                }
            }
            FilterSubSection::Bandwidth => {
                match key_code {
                    KeyCode::Left => {
                        self.filter.bandwidth_slider.adjust_log(0.95);
                        Some(self.get_filter_bandwidth_update())
                    }
                    KeyCode::Right => {
                        self.filter.bandwidth_slider.adjust_log(1.05);
                        Some(self.get_filter_bandwidth_update())
                    }
                    _ => None
                }
            }
            FilterSubSection::Resonance => {
                match key_code {
                    KeyCode::Left => {
                        self.filter.resonance_slider.adjust(-0.05);
                        Some(ParameterUpdate::FilterResonance(
                            self.filter.resonance_slider.value
                        ))
                    }
                    KeyCode::Right => {
                        self.filter.resonance_slider.adjust(0.05);
                        Some(ParameterUpdate::FilterResonance(
                            self.filter.resonance_slider.value
                        ))
                    }
                    _ => None
                }
            }
            FilterSubSection::Mix => {
                match key_code {
                    KeyCode::Left => {
                        self.filter.mix_slider.adjust(-0.05);
                        Some(ParameterUpdate::FilterMix(
                            self.filter.mix_slider.value
                        ))
                    }
                    KeyCode::Right => {
                        self.filter.mix_slider.adjust(0.05);
                        Some(ParameterUpdate::FilterMix(
                            self.filter.mix_slider.value
                        ))
                    }
                    _ => None
                }
            }
        }
    }
    
    fn handle_oscillator_activation(&mut self, osc_section: OscillatorSubSection) -> Option<ParameterUpdate> {
        match osc_section {
            OscillatorSubSection::Waveform => {
                self.oscillator.waveform_selector.toggle_expanded();
                Some(ParameterUpdate::OscillatorWaveform(
                    self.oscillator.waveform_selector.selected_waveform()
                ))
            }
            _ => None
        }
    }
    
    fn handle_filter_activation(&mut self, filter_section: FilterSubSection) -> Option<ParameterUpdate> {
        match filter_section {
            FilterSubSection::Type => {
                self.filter.filter_type.toggle_expanded();
                Some(ParameterUpdate::FilterType(
                    self.filter.filter_type.selected_filter().clone()
                ))
            }
            _ => None
        }
    }
    
    pub fn get_waveform(&self) -> Waveform {
        self.oscillator.waveform_selector.selected_waveform()
    }
    
    pub fn get_volume(&self) -> f32 {
        self.oscillator.volume_slider.value
    }
    
    pub fn get_filter_type(&self) -> &FilterType {
        self.filter.filter_type.selected_filter()
    }
    
    pub fn get_filter_frequency(&self) -> f32 {
        self.filter.frequency_slider.value
    }
    
    pub fn get_filter_bandwidth(&self) -> f32 {
        self.filter.bandwidth_slider.value
    }
    
    pub fn get_filter_resonance(&self) -> f32 {
        self.filter.resonance_slider.value
    }
    
    pub fn get_filter_mix(&self) -> f32 {
        self.filter.mix_slider.value
    }
    
    // Envelope getter methods
    pub fn get_envelope_attack_time(&self) -> f32 {
        self.envelope.attack_time_slider.value
    }
    
    pub fn get_envelope_attack_level(&self) -> f32 {
        self.envelope.attack_level_slider.value
    }
    
    pub fn get_envelope_decay_time(&self) -> f32 {
        self.envelope.decay_time_slider.value
    }
    
    pub fn get_envelope_decay_level(&self) -> f32 {
        self.envelope.decay_level_slider.value
    }
    
    pub fn get_envelope_sustain_time(&self) -> f32 {
        self.envelope.sustain_time_slider.value
    }
    
    pub fn get_envelope_sustain_level(&self) -> f32 {
        self.envelope.sustain_level_slider.value
    }
    
    pub fn get_envelope_release_time(&self) -> f32 {
        self.envelope.release_time_slider.value
    }
    
    pub fn get_envelope_release_level(&self) -> f32 {
        self.envelope.release_level_slider.value
    }
}

impl Default for OscillatorControls {
    fn default() -> Self {
        Self::new()
    }
}

impl OscillatorControls {
    pub fn new() -> Self {
        Self {
            waveform_selector: WaveformSelector::new(),
            volume_slider: LinearSlider::new("Vol", 0.75, 0.0, 1.0, 10),
            sub_focus: OscillatorSubSection::Waveform,
        }
    }
    
    pub fn render(&self, area: Rect, buf: &mut Buffer, focused: bool, current_section: OscillatorSubSection) {
        let title = if focused { "OSCILLATOR [FOCUSED]" } else { "OSCILLATOR" };
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL);
        
        let inner = block.inner(area);
        block.render(area, buf);
        
        // Split oscillator area vertically
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Waveform
                Constraint::Length(2), // Volume
            ])
            .split(inner);
        
        // Render waveform selector
        let mut waveform_selector = self.waveform_selector.clone();
        waveform_selector.focused = focused && current_section == OscillatorSubSection::Waveform;
        waveform_selector.render(chunks[0], buf);
        
        // Render volume slider
        let mut vol_slider = self.volume_slider.clone();
        vol_slider.focused = focused && current_section == OscillatorSubSection::Volume;
        vol_slider.render(chunks[1], buf);
    }
}

impl Default for FilterControls {
    fn default() -> Self {
        Self::new()
    }
}

impl FilterControls {
    pub fn new() -> Self {
        Self {
            filter_type: FilterTypeSelector::new(),
            frequency_slider: LogSlider::new("Freq", 1000.0, 20.0, 20000.0, 8),
            bandwidth_slider: LogSlider::new("BW", 200.0, 10.0, 5000.0, 8),
            resonance_slider: LinearSlider::new("Res", 0.3, 0.0, 1.0, 8),
            mix_slider: LinearSlider::new("Mix", 0.8, 0.0, 1.0, 8),
            sub_focus: FilterSubSection::Type,
        }
    }
    
    pub fn render(&self, area: Rect, buf: &mut Buffer, focused: bool, current_section: FilterSubSection) {
        let title = if focused { "FILTER [FOCUSED]" } else { "FILTER" };
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL);
        
        let inner = block.inner(area);
        block.render(area, buf);
        
        // Determine which controls to show based on filter type
        let filter_type = self.filter_type.selected_filter();
        let has_bandwidth = matches!(filter_type, FilterType::BandPass | FilterType::Notch);
        
        // Create constraints based on filter type
        let constraints = if has_bandwidth {
            vec![
                Constraint::Length(2), // Type
                Constraint::Length(2), // Frequency (Center for BP/Notch)
                Constraint::Length(2), // Bandwidth
                Constraint::Length(2), // Resonance
                Constraint::Length(2), // Mix
            ]
        } else {
            vec![
                Constraint::Length(2), // Type
                Constraint::Length(2), // Frequency (Cutoff for LP/HP)
                Constraint::Length(2), // Resonance
                Constraint::Length(2), // Mix
            ]
        };
        
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(inner);
        
        // Render filter type selector
        let mut filter_type_widget = self.filter_type.clone();
        filter_type_widget.focused = focused && current_section == FilterSubSection::Type;
        filter_type_widget.render(chunks[0], buf);
        
        // Render frequency control (label changes based on filter type)
        let freq_label = if has_bandwidth { "Center" } else { "Cutoff" };
        let mut freq_slider = self.frequency_slider.clone();
        freq_slider.label = freq_label.to_string();
        freq_slider.focused = focused && current_section == FilterSubSection::Frequency;
        freq_slider.render(chunks[1], buf);
        
        if has_bandwidth {
            // Render bandwidth slider (only for BP/Notch)
            let mut bw_slider = self.bandwidth_slider.clone();
            bw_slider.focused = focused && current_section == FilterSubSection::Bandwidth;
            bw_slider.render(chunks[2], buf);
            
            // Render resonance slider
            let mut resonance_slider = self.resonance_slider.clone();
            resonance_slider.focused = focused && current_section == FilterSubSection::Resonance;
            resonance_slider.render(chunks[3], buf);
            
            // Render mix slider
            let mut mix_slider = self.mix_slider.clone();
            mix_slider.focused = focused && current_section == FilterSubSection::Mix;
            mix_slider.render(chunks[4], buf);
        } else {
            // Render resonance slider (for LP/HP)
            let mut resonance_slider = self.resonance_slider.clone();
            resonance_slider.focused = focused && current_section == FilterSubSection::Resonance;
            resonance_slider.render(chunks[2], buf);
            
            // Render mix slider
            let mut mix_slider = self.mix_slider.clone();
            mix_slider.focused = focused && current_section == FilterSubSection::Mix;
            mix_slider.render(chunks[3], buf);
        }
    }
}

impl Default for EnvelopeControls {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvelopeControls {
    pub fn new() -> Self {
        Self {
            attack_time_slider: LinearSlider::new("A.Time", 0.02, 0.001, 1.0, 8),
            attack_level_slider: LinearSlider::new("A.Level", 1.0, 0.0, 1.0, 8),
            decay_time_slider: LinearSlider::new("D.Time", 0.51, 0.001, 1.0, 8),
            decay_level_slider: LinearSlider::new("D.Level", 1.0, 0.0, 1.0, 8),
            sustain_time_slider: LinearSlider::new("S.Time", 0.98, 0.001, 1.0, 8),
            sustain_level_slider: LinearSlider::new("S.Level", 1.0, 0.0, 1.0, 8),
            release_time_slider: LinearSlider::new("R.Time", 1.0, 0.001, 1.0, 8),
            release_level_slider: LinearSlider::new("R.Level", 0.0, 0.0, 1.0, 8),
            sub_focus: EnvelopeSubSection::AttackTime,
        }
    }
    
    pub fn render(&self, area: Rect, buf: &mut Buffer, focused: bool, current_section: EnvelopeSubSection) {
        let title = if focused { "ENVELOPE [FOCUSED]" } else { "ENVELOPE" };
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL);
        
        let inner = block.inner(area);
        block.render(area, buf);
        
        // Split envelope area vertically for all controls
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Attack Time
                Constraint::Length(2), // Attack Level
                Constraint::Length(2), // Decay Time
                Constraint::Length(2), // Decay Level
                Constraint::Length(2), // Sustain Time
                Constraint::Length(2), // Sustain Level
                Constraint::Length(2), // Release Time
                Constraint::Length(2), // Release Level
            ])
            .split(inner);
        
        // Render attack time slider
        let mut attack_time_slider = self.attack_time_slider.clone();
        attack_time_slider.focused = focused && current_section == EnvelopeSubSection::AttackTime;
        attack_time_slider.render(chunks[0], buf);
        
        // Render attack level slider
        let mut attack_level_slider = self.attack_level_slider.clone();
        attack_level_slider.focused = focused && current_section == EnvelopeSubSection::AttackLevel;
        attack_level_slider.render(chunks[1], buf);
        
        // Render decay time slider
        let mut decay_time_slider = self.decay_time_slider.clone();
        decay_time_slider.focused = focused && current_section == EnvelopeSubSection::DecayTime;
        decay_time_slider.render(chunks[2], buf);
        
        // Render decay level slider
        let mut decay_level_slider = self.decay_level_slider.clone();
        decay_level_slider.focused = focused && current_section == EnvelopeSubSection::DecayLevel;
        decay_level_slider.render(chunks[3], buf);
        
        // Render sustain time slider
        let mut sustain_time_slider = self.sustain_time_slider.clone();
        sustain_time_slider.focused = focused && current_section == EnvelopeSubSection::SustainTime;
        sustain_time_slider.render(chunks[4], buf);
        
        // Render sustain level slider
        let mut sustain_level_slider = self.sustain_level_slider.clone();
        sustain_level_slider.focused = focused && current_section == EnvelopeSubSection::SustainLevel;
        sustain_level_slider.render(chunks[5], buf);
        
        // Render release time slider
        let mut release_time_slider = self.release_time_slider.clone();
        release_time_slider.focused = focused && current_section == EnvelopeSubSection::ReleaseTime;
        release_time_slider.render(chunks[6], buf);
        
        // Render release level slider
        let mut release_level_slider = self.release_level_slider.clone();
        release_level_slider.focused = focused && current_section == EnvelopeSubSection::ReleaseLevel;
        release_level_slider.render(chunks[7], buf);
    }
}

impl Default for EffectsControls {
    fn default() -> Self {
        Self::new()
    }
}

impl EffectsControls {
    pub fn new() -> Self {
        Self {
            // Delay controls
            delay_time_slider: LinearSlider::new("D.Time", 0.25, 0.0, 1.0, 8),
            delay_feedback_slider: LinearSlider::new("D.Fdbk", 0.3, 0.0, 0.95, 8),
            delay_mix_slider: LinearSlider::new("D.Mix", 0.3, 0.0, 1.0, 8),

            // Flanger controls
            flanger_rate_slider: LinearSlider::new("F.Rate", 0.5, 0.1, 10.0, 8),
            flanger_depth_slider: LinearSlider::new("F.Dpth", 0.5, 0.0, 1.0, 8),
            flanger_mix_slider: LinearSlider::new("F.Mix", 0.5, 0.0, 1.0, 8),

            // LFO controls
            lfo_rate_slider: LinearSlider::new("L.Rate", 1.0, 0.1, 20.0, 8),
            lfo_depth_slider: LinearSlider::new("L.Dpth", 0.5, 0.0, 1.0, 8),

            sub_focus: EffectsSubSection::DelayTime,
        }
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, focused: bool, current_section: EffectsSubSection) {
        use ratatui::style::{Color, Style};

        let title = if focused { "EFFECTS [FOCUSED]" } else { "EFFECTS" };
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL);

        let inner = block.inner(area);
        block.render(area, buf);

        // Split effects area vertically for all controls
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Delay header
                Constraint::Length(1), // Delay Time
                Constraint::Length(1), // Delay Feedback
                Constraint::Length(1), // Delay Mix
                Constraint::Length(1), // Flanger header
                Constraint::Length(1), // Flanger Rate
                Constraint::Length(1), // Flanger Depth
                Constraint::Length(1), // Flanger Mix
                Constraint::Length(1), // LFO header
                Constraint::Length(1), // LFO Rate
                Constraint::Length(1), // LFO Depth
            ])
            .split(inner);

        // Render section headers and sliders
        let header_style = Style::default().fg(Color::Yellow);
        buf.set_string(chunks[0].x, chunks[0].y, "─DELAY─", header_style);

        let mut delay_time = self.delay_time_slider.clone();
        delay_time.focused = focused && current_section == EffectsSubSection::DelayTime;
        delay_time.render(chunks[1], buf);

        let mut delay_feedback = self.delay_feedback_slider.clone();
        delay_feedback.focused = focused && current_section == EffectsSubSection::DelayFeedback;
        delay_feedback.render(chunks[2], buf);

        let mut delay_mix = self.delay_mix_slider.clone();
        delay_mix.focused = focused && current_section == EffectsSubSection::DelayMix;
        delay_mix.render(chunks[3], buf);

        buf.set_string(chunks[4].x, chunks[4].y, "─FLANGER─", header_style);

        let mut flanger_rate = self.flanger_rate_slider.clone();
        flanger_rate.focused = focused && current_section == EffectsSubSection::FlangerRate;
        flanger_rate.render(chunks[5], buf);

        let mut flanger_depth = self.flanger_depth_slider.clone();
        flanger_depth.focused = focused && current_section == EffectsSubSection::FlangerDepth;
        flanger_depth.render(chunks[6], buf);

        let mut flanger_mix = self.flanger_mix_slider.clone();
        flanger_mix.focused = focused && current_section == EffectsSubSection::FlangerMix;
        flanger_mix.render(chunks[7], buf);

        buf.set_string(chunks[8].x, chunks[8].y, "─LFO─", header_style);

        let mut lfo_rate = self.lfo_rate_slider.clone();
        lfo_rate.focused = focused && current_section == EffectsSubSection::LfoRate;
        lfo_rate.render(chunks[9], buf);

        let mut lfo_depth = self.lfo_depth_slider.clone();
        lfo_depth.focused = focused && current_section == EffectsSubSection::LfoDepth;
        lfo_depth.render(chunks[10], buf);
    }
}