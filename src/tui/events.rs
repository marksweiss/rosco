use crate::tui::TuiError;
use crossterm::event::{self, Event, KeyEvent};
use std::time::Duration;
use tokio::sync::mpsc;

pub struct EventHandler {
    // Future: could add more sophisticated event handling here
}

impl EventHandler {
    pub fn new() -> Self {
        Self {}
    }
    
    pub async fn next_event(&self, timeout: Duration) -> Result<Option<Event>, TuiError> {
        if event::poll(timeout)? {
            Ok(Some(event::read()?))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone)]
pub enum TuiEvent {
    Key(KeyEvent),
    Tick,
    Quit,
    AudioFeedback(crate::tui::audio_bridge::AudioFeedback),
}

pub struct EventLoop {
    event_sender: mpsc::UnboundedSender<TuiEvent>,
    event_receiver: mpsc::UnboundedReceiver<TuiEvent>,
}

impl EventLoop {
    pub fn new() -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        Self {
            event_sender,
            event_receiver,
        }
    }
    
    pub fn sender(&self) -> mpsc::UnboundedSender<TuiEvent> {
        self.event_sender.clone()
    }
    
    pub async fn next(&mut self) -> Option<TuiEvent> {
        self.event_receiver.recv().await
    }
    
    pub async fn start_input_handler(&self) -> Result<(), TuiError> {
        let sender = self.event_sender.clone();
        
        tokio::spawn(async move {
            loop {
                if let Ok(true) = event::poll(Duration::from_millis(16)) {
                    if let Ok(event) = event::read() {
                        match event {
                            Event::Key(key) => {
                                if sender.send(TuiEvent::Key(key)).is_err() {
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                
                // Send tick events for regular updates
                if sender.send(TuiEvent::Tick).is_err() {
                    break;
                }
                
                tokio::time::sleep(Duration::from_millis(16)).await;
            }
        });
        
        Ok(())
    }
}