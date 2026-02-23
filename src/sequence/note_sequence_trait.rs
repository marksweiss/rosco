use crate::note::playback_note::PlaybackNote;

#[allow(dead_code)]
pub(crate) trait AppendNote {
    fn append_note(&mut self, note: PlaybackNote);
}

pub(crate) trait AppendNotes {
    fn append_notes(&mut self, notes: &[PlaybackNote]);
}

pub(crate) trait BuilderWrapper<SequenceType> {
    fn new() -> SequenceType;
}

pub(crate) trait NextNotes {
    fn next_notes(&mut self) -> Vec<PlaybackNote>;
}

pub(crate) trait SetCurPosition {
    fn set_cur_position(&mut self, position: f32);
}

#[allow(dead_code)]
pub(crate) trait IterMutWrapper {
    fn iter_mut(&mut self) -> std::slice::IterMut<'_, Vec<PlaybackNote>>;
}
