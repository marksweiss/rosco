# Agent Rules

This file contains rules and guidelines for agents working on the Rosco codebase.

## Module Directory Rules

### Rule 1: Check Module Summary Before Working
Before working on any files within a src module directory, always check for and read a `summary.md` file in that directory first. This file may contain important context about the module's purpose, architecture, and implementation details that are crucial for making appropriate changes.

**Process:**
1. When tasked with modifying files in `src/[module_name]/`, first look for `src/[module_name]/summary.md`
2. If the summary file exists, read it completely before proceeding with any modifications
3. Use the information from the summary to inform your approach to the task
4. If no summary file exists, proceed with normal analysis of the module structure

**Examples:**
- Before modifying `src/filter/low_pass.rs`, check for `src/filter/summary.md`
- Before working on `src/audio_gen/oscillator.rs`, check for `src/audio_gen/summary.md`
- Before updating `src/dsl/parser.rs`, check for `src/dsl/summary.md`

## General Working Rules

### Rule 2: Follow Existing Patterns
Always examine existing code patterns, naming conventions, and architectural decisions within a module before making changes.

### Rule 3: Respect Module Boundaries
Understand each module's responsibilities and avoid creating inappropriate dependencies between modules.

### Rule 4: Test After Changes
Run relevant tests after making changes to ensure functionality is preserved:
- `cargo test` for general testing
- `cargo test [module_name]::tests` for module-specific tests

## Automated Systems

### Pre-commit Hook: Summary File Updates
A pre-commit hook automatically updates `summary.md` files when source files in their respective modules change:

**Functionality:**
- Monitors staged files during commits for changes in `src/[module_name]/` directories
- Automatically updates the corresponding `src/[module_name]/summary.md` file with:
  - Current timestamp
  - Note indicating automatic update due to source changes
- Re-stages the updated summary file as part of the commit

**Location:** `.git/hooks/pre-commit`

**Behavior:**
- Only triggers when source files (not summary.md itself) are modified
- Preserves original summary content while adding update metadata
- Provides console feedback about which modules were updated

This ensures summary files stay current with code changes without manual intervention.