use crate::filter::low_pass_filter::default_low_pass_filter;
use crate::filter::high_pass_filter::default_high_pass_filter;
use crate::filter::band_pass_filter::default_band_pass_filter;
use crate::filter::notch_filter::default_notch_filter;
use crate::tui::ui::widgets::selector::FilterType;
use crate::track::track_effects::TrackEffects;

#[derive(Debug, Clone)]
pub struct FilterManager {
    pub filter_type: FilterType,
    pub frequency: f32,
    pub bandwidth: f32,
    pub resonance: f32,
    pub mix: f32,
}

impl Default for FilterManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FilterManager {
    pub fn new() -> Self {
        Self {
            filter_type: FilterType::LowPass,
            frequency: 1000.0,
            bandwidth: 200.0,
            resonance: 0.3,
            mix: 0.8,
        }
    }
    
    pub fn update_filter_type(&mut self, filter_type: FilterType) {
        self.filter_type = filter_type;
    }
    
    pub fn update_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
    }
    
    pub fn update_bandwidth(&mut self, bandwidth: f32) {
        self.bandwidth = bandwidth;
    }
    
    pub fn update_resonance(&mut self, resonance: f32) {
        self.resonance = resonance;
    }
    
    pub fn update_mix(&mut self, mix: f32) {
        self.mix = mix;
    }
    
    pub(crate) fn apply_to_track_effects(&self, track_effects: &mut TrackEffects) {
        // Clear existing filters
        track_effects.low_pass_filters.clear();
        track_effects.high_pass_filters.clear();
        track_effects.band_pass_filters.clear();
        track_effects.notch_filters.clear();
        
        // Add the current filter
        match self.filter_type {
            FilterType::LowPass => {
                let mut filter = default_low_pass_filter();
                filter.cutoff_frequency = self.frequency;
                filter.resonance = self.resonance;
                filter.mix = self.mix;
                filter.update_coefficients();
                track_effects.low_pass_filters.push(filter);
            }
            FilterType::HighPass => {
                let mut filter = default_high_pass_filter();
                filter.cutoff_frequency = self.frequency;
                filter.resonance = self.resonance;
                filter.mix = self.mix;
                filter.update_coefficients();
                track_effects.high_pass_filters.push(filter);
            }
            FilterType::BandPass => {
                let mut filter = default_band_pass_filter();
                filter.center_frequency = self.frequency;
                filter.bandwidth = self.bandwidth;
                filter.resonance = self.resonance;
                filter.mix = self.mix;
                filter.update_coefficients();
                track_effects.band_pass_filters.push(filter);
            }
            FilterType::Notch => {
                let mut filter = default_notch_filter();
                filter.center_frequency = self.frequency;
                filter.bandwidth = self.bandwidth;
                filter.resonance = self.resonance;
                filter.mix = self.mix;
                filter.update_coefficients();
                track_effects.notch_filters.push(filter);
            }
        }
    }
}
