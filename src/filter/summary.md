# Filter Module

## Purpose
Implements IIR (Infinite Impulse Response) digital filters for audio processing. This module provides various filter types for frequency-domain audio manipulation and tone shaping.

## Key Components
- **low_pass_filter.rs**: Low-pass filter implementation (allows low frequencies, attenuates high)
- **high_pass_filter.rs**: High-pass filter implementation (allows high frequencies, attenuates low)
- **band_pass_filter.rs**: Band-pass filter implementation (allows specific frequency range)
- **notch_filter.rs**: Notch filter implementation (removes specific frequency range)
- **example.rs**: Usage examples and demonstrations
- **test_filter.rs**: Filter testing utilities
- **README.md**: Detailed filter documentation

## Architecture
All filters use Direct Form II IIR structure with:
- Builder pattern construction via derive_builder
- Real-time processing capabilities
- Mix control (dry/wet blend) for effect intensity
- Resonance control (Q factor) for filter sharpness
- Thread-safe history management for stateful processing

## Dependencies
- Uses derive_builder for builder pattern construction
- Integrates with track module for per-track filtering
- Works with common module for shared utilities

## Usage Patterns
- Filters are constructed using builder pattern for flexible configuration
- Each filter type has specific use cases for frequency manipulation
- Mix control allows blending filtered and unfiltered audio
- Resonance parameter controls filter sharpness and character
- Multiple filters can be applied per track for complex frequency shaping