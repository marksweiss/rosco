use eframe::egui;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusPanel {
    Oscillator,
    Envelope,
    Effects,
    Sequencer,
    Transport,
}

impl FocusPanel {
    pub fn next(self) -> Self {
        match self {
            Self::Oscillator => Self::Envelope,
            Self::Envelope => Self::Effects,
            Self::Effects => Self::Sequencer,
            Self::Sequencer => Self::Transport,
            Self::Transport => Self::Oscillator,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Oscillator => Self::Transport,
            Self::Envelope => Self::Oscillator,
            Self::Effects => Self::Envelope,
            Self::Sequencer => Self::Effects,
            Self::Transport => Self::Sequencer,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Oscillator => "Oscillator",
            Self::Envelope => "Envelope",
            Self::Effects => "Effects",
            Self::Sequencer => "Sequencer",
            Self::Transport => "Transport",
        }
    }
}

#[derive(Clone, Debug)]
pub enum ShortcutAction {
    PlayPause,
    Stop,
    NextPanel,
    PrevPanel,
    Undo,
    Redo,
    SaveSession,
    LoadFile,
}

/// Check for keyboard shortcuts and return the action if one was triggered.
pub fn process_shortcuts(ctx: &egui::Context) -> Option<ShortcutAction> {
    let mut action = None;

    ctx.input(|input| {
        let ctrl = input.modifiers.command; // Cmd on macOS, Ctrl on others
        let shift = input.modifiers.shift;

        // Space -> Play/Pause (only when no text widget is focused)
        if input.key_pressed(egui::Key::Space) && !ctrl && !has_text_focus(ctx) {
            action = Some(ShortcutAction::PlayPause);
        }

        // Escape -> Stop
        if input.key_pressed(egui::Key::Escape) {
            action = Some(ShortcutAction::Stop);
        }

        // Tab / Shift+Tab -> Next/Prev panel
        if input.key_pressed(egui::Key::Tab) && !ctrl {
            if shift {
                action = Some(ShortcutAction::PrevPanel);
            } else {
                action = Some(ShortcutAction::NextPanel);
            }
        }

        // Ctrl+Z / Ctrl+Shift+Z -> Undo/Redo
        if ctrl && input.key_pressed(egui::Key::Z) {
            if shift {
                action = Some(ShortcutAction::Redo);
            } else {
                action = Some(ShortcutAction::Undo);
            }
        }

        // Ctrl+S -> Save session
        if ctrl && input.key_pressed(egui::Key::S) {
            action = Some(ShortcutAction::SaveSession);
        }

        // Ctrl+O -> Load file
        if ctrl && input.key_pressed(egui::Key::O) {
            action = Some(ShortcutAction::LoadFile);
        }
    });

    action
}

fn has_text_focus(ctx: &egui::Context) -> bool {
    ctx.memory(|mem| mem.focused().is_some())
}

// --- Minimal undo/redo system ---

use crate::tui::audio_bridge::ParameterUpdate;

#[derive(Clone, Debug)]
struct UndoEntry {
    forward: ParameterUpdate,
    backward: ParameterUpdate,
    description: String,
}

pub struct UndoStack {
    entries: Vec<UndoEntry>,
    position: usize, // points to next entry to undo (entries[position-1])
}

const MAX_UNDO: usize = 100;

impl UndoStack {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            position: 0,
        }
    }

    /// Record a parameter change. `forward` is the new value, `backward` is the old value.
    pub fn record(&mut self, forward: ParameterUpdate, backward: ParameterUpdate, description: String) {
        // Discard any redo history beyond current position
        self.entries.truncate(self.position);
        self.entries.push(UndoEntry {
            forward,
            backward,
            description,
        });
        self.position = self.entries.len();

        // Cap at MAX_UNDO
        if self.entries.len() > MAX_UNDO {
            let excess = self.entries.len() - MAX_UNDO;
            self.entries.drain(0..excess);
            self.position = self.entries.len();
        }
    }

    /// Undo: returns the backward ParameterUpdate and a description.
    pub fn undo(&mut self) -> Option<(ParameterUpdate, String)> {
        if self.position == 0 {
            return None;
        }
        self.position -= 1;
        let entry = &self.entries[self.position];
        Some((entry.backward.clone(), format!("Undo: {}", entry.description)))
    }

    /// Redo: returns the forward ParameterUpdate and a description.
    pub fn redo(&mut self) -> Option<(ParameterUpdate, String)> {
        if self.position >= self.entries.len() {
            return None;
        }
        let entry = &self.entries[self.position];
        self.position += 1;
        Some((entry.forward.clone(), format!("Redo: {}", entry.description)))
    }

    pub fn can_undo(&self) -> bool {
        self.position > 0
    }

    pub fn can_redo(&self) -> bool {
        self.position < self.entries.len()
    }
}
