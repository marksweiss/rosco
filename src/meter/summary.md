# Meter Module

## Purpose
Handles timing, duration, and meter-related functionality for musical compositions. This module provides the temporal foundation for organizing musical events and maintaining rhythmic structure.

## Key Components
- **meter.rs**: Core meter and timing logic
- **durations.rs**: Duration calculations and time-based utilities

## Architecture
The meter module establishes the temporal framework for musical compositions, providing:
- Time signature management
- Beat and measure calculations
- Duration conversions between musical and sample time
- Tempo and timing utilities

## Dependencies
- Works closely with sequence module for note timing
- Integrates with composition module for overall timing structure
- Uses common module for shared calculations

## Usage Patterns
- Provides timing foundation for note sequences and compositions
- Handles conversion between musical time (beats, measures) and audio time (samples)
- Supports various time signatures and tempo changes
- Essential for synchronizing musical events across tracks