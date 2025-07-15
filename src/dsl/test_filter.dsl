# Test script demonstrating filter usage in DSL
# This script creates a sequence with a low-pass filter applied to oscillator notes

FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
# Envelope for smooth attack and release
a 0.1,0.8 d 0.3,0.6 s 0.8,0.4 r 1.0,0.0
# Low-pass filter to soften the sound
filter cutoff_frequency 800.0 resonance 0.2 mix 0.7
# Add some delay for space
delay mix 0.3 decay 0.5 interval_ms 100.0 duration_ms 50.0 num_repeats 2 num_predelay_samples 10 num_concurrent_delays 1

# Notes with filter applied
osc:sine:440.0:0.6:0
osc:square:880.0:0.4:4
osc:triangle:220.0:0.5:8
osc:sine:660.0:0.3:12 