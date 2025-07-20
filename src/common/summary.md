# Common Utilities Module

## Purpose
Contains shared utilities, constants, and helper functions used across the entire Rosco codebase. This module provides foundational components that support other modules.

## Key Components
- **constants.rs**: System-wide constants and configuration values
- **float_utils.rs**: Floating-point arithmetic utilities and helper functions
- **pair.rs**: Pair data structure and related utilities

## Architecture
The common module serves as a central repository for shared functionality, preventing code duplication across modules. It contains low-level utilities that don't belong to any specific domain module.

## Dependencies
- Minimal external dependencies to avoid circular imports
- Used by most other modules in the system

## Usage Patterns
- Import constants for consistent configuration across modules
- Use float utilities for audio processing calculations
- Leverage pair structures for stereo audio handling and coordinate systems