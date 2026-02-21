use std::collections::VecDeque;

use crate::note::note_trait::BuilderWrapper;

pub struct NotePool<NoteType> {
    available: VecDeque<NoteType>,
    #[allow(dead_code)]
    capacity: usize,
}

impl<NoteType> NotePool<NoteType> {
    
    pub fn new<NoteBuilderType: BuilderWrapper<NoteType>>(capacity: usize) -> Self {
        let mut available = VecDeque::with_capacity(capacity);
        // Pre-allocate PlaybackNotes
        for _ in 0..capacity {
            available.push_back(NoteBuilderType::new());
        }
        Self { available, capacity }
    }
    
    pub fn acquire(&mut self) -> Option<NoteType> {
        self.available.pop_front()
    }
    
    #[allow(dead_code)]
    pub fn release(&mut self, note: NoteType) {
        if self.available.len() < self.capacity {
            self.available.push_back(note);
        }
    }
}