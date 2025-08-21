use crate::tui::audio_engine::AudioState;
use crate::note::scales::WesternPitch;
use std::sync::atomic::Ordering;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_frequency_storage() {
        let audio_state = AudioState::default();
        
        // Test setting different frequencies for different steps in same track
        let track_idx = 0;
        let step_0_idx = track_idx * 16 + 0; // First step in track 0
        let step_1_idx = track_idx * 16 + 1; // Second step in track 0
        
        // Set different frequencies for two steps in the same track
        let freq_c4 = WesternPitch::C.get_frequency(4); // C4 = 261.63 Hz
        let freq_a4 = WesternPitch::A.get_frequency(4); // A4 = 440.00 Hz
        
        println!("C4 frequency: {:.2} Hz, A4 frequency: {:.2} Hz", freq_c4, freq_a4);
        
        audio_state.step_frequencies[step_0_idx].store(freq_c4, Ordering::Relaxed);
        audio_state.step_frequencies[step_1_idx].store(freq_a4, Ordering::Relaxed);
        
        // Verify that each step maintains its own frequency
        let stored_freq_c4 = audio_state.step_frequencies[step_0_idx].load(Ordering::Relaxed);
        let stored_freq_a4 = audio_state.step_frequencies[step_1_idx].load(Ordering::Relaxed);
        
        assert!((stored_freq_c4 - freq_c4).abs() < 0.01, "C4 frequency not stored correctly: expected {:.2}, got {:.2}", freq_c4, stored_freq_c4);
        assert!((stored_freq_a4 - freq_a4).abs() < 0.01, "A4 frequency not stored correctly: expected {:.2}, got {:.2}", freq_a4, stored_freq_a4);
        assert!((stored_freq_c4 - stored_freq_a4).abs() > 50.0, "Frequencies should be different: C4={:.2}, A4={:.2}", stored_freq_c4, stored_freq_a4);
    }
    
    #[test]
    fn test_multiple_tracks_different_frequencies() {
        let audio_state = AudioState::default();
        
        // Test that different tracks can have different frequencies for the same step position
        let step_pos = 5; // Step position 5
        let track0_step5_idx = 0 * 16 + step_pos; // Track 0, Step 5
        let track1_step5_idx = 1 * 16 + step_pos; // Track 1, Step 5
        
        let freq_e4 = WesternPitch::E.get_frequency(4); // E4 = 329.63 Hz
        let freq_g5 = WesternPitch::G.get_frequency(5); // G5 = 783.99 Hz
        
        println!("E4 frequency: {:.2} Hz, G5 frequency: {:.2} Hz", freq_e4, freq_g5);
        
        audio_state.step_frequencies[track0_step5_idx].store(freq_e4, Ordering::Relaxed);
        audio_state.step_frequencies[track1_step5_idx].store(freq_g5, Ordering::Relaxed);
        
        let stored_e4 = audio_state.step_frequencies[track0_step5_idx].load(Ordering::Relaxed);
        let stored_g5 = audio_state.step_frequencies[track1_step5_idx].load(Ordering::Relaxed);
        
        assert!((stored_e4 - freq_e4).abs() < 0.01, "E4 frequency not stored correctly: expected {:.2}, got {:.2}", freq_e4, stored_e4);
        assert!((stored_g5 - freq_g5).abs() < 0.01, "G5 frequency not stored correctly: expected {:.2}, got {:.2}", freq_g5, stored_g5);
        assert!((stored_e4 - stored_g5).abs() > 200.0, "Different tracks should have different frequencies: E4={:.2}, G5={:.2}", stored_e4, stored_g5);
    }
}