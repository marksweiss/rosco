use crate::tui::ui::widgets::StepCell;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Pattern storage and management for the TUI sequencer
#[derive(Debug, Clone)]
pub struct PatternManager {
    patterns: HashMap<String, Pattern>,
    next_pattern_id: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub id: String,
    pub name: String,
    pub steps: Vec<StepCell>,
    pub length: usize,
    pub created: chrono::DateTime<chrono::Utc>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternBank {
    pub patterns: HashMap<String, Pattern>,
    pub version: String,
    pub created: chrono::DateTime<chrono::Utc>,
}

impl PatternManager {
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
            next_pattern_id: 1,
        }
    }
    
    /// Store a new pattern
    pub fn store_pattern(&mut self, name: String, steps: Vec<StepCell>, description: Option<String>) -> String {
        let pattern_id = format!("pattern_{:04}", self.next_pattern_id);
        self.next_pattern_id += 1;
        
        let length = steps.len();
        let pattern = Pattern {
            id: pattern_id.clone(),
            name,
            steps,
            length,
            created: chrono::Utc::now(),
            description,
        };
        
        self.patterns.insert(pattern_id.clone(), pattern);
        pattern_id
    }
    
    /// Retrieve a pattern by ID
    pub fn get_pattern(&self, pattern_id: &str) -> Option<&Pattern> {
        self.patterns.get(pattern_id)
    }
    
    /// Get pattern steps for pasting
    pub fn get_pattern_steps(&self, pattern_id: &str) -> Option<Vec<StepCell>> {
        self.patterns.get(pattern_id).map(|p| p.steps.clone())
    }
    
    /// List all patterns
    pub fn list_patterns(&self) -> Vec<&Pattern> {
        self.patterns.values().collect()
    }
    
    /// Delete a pattern
    pub fn delete_pattern(&mut self, pattern_id: &str) -> bool {
        self.patterns.remove(pattern_id).is_some()
    }
    
    /// Rename a pattern
    pub fn rename_pattern(&mut self, pattern_id: &str, new_name: String) -> bool {
        if let Some(pattern) = self.patterns.get_mut(pattern_id) {
            pattern.name = new_name;
            true
        } else {
            false
        }
    }
    
    /// Update pattern description
    pub fn update_description(&mut self, pattern_id: &str, description: Option<String>) -> bool {
        if let Some(pattern) = self.patterns.get_mut(pattern_id) {
            pattern.description = description;
            true
        } else {
            false
        }
    }
    
    /// Search patterns by name
    pub fn search_patterns(&self, query: &str) -> Vec<&Pattern> {
        self.patterns
            .values()
            .filter(|p| p.name.to_lowercase().contains(&query.to_lowercase()))
            .collect()
    }
    
    /// Get patterns sorted by creation date (newest first)
    pub fn get_recent_patterns(&self, limit: usize) -> Vec<&Pattern> {
        let mut patterns: Vec<&Pattern> = self.patterns.values().collect();
        patterns.sort_by(|a, b| b.created.cmp(&a.created));
        patterns.truncate(limit);
        patterns
    }
    
    /// Export patterns to a pattern bank
    pub fn export_bank(&self) -> PatternBank {
        PatternBank {
            patterns: self.patterns.clone(),
            version: "1.0".to_string(),
            created: chrono::Utc::now(),
        }
    }
    
    /// Import patterns from a pattern bank
    pub fn import_bank(&mut self, bank: PatternBank) -> Result<usize, String> {
        let mut imported_count = 0;
        
        for (id, pattern) in bank.patterns {
            // Check for ID conflicts and rename if necessary
            let final_id = if self.patterns.contains_key(&id) {
                format!("{}_{}", id, chrono::Utc::now().timestamp())
            } else {
                id
            };
            
            self.patterns.insert(final_id, pattern);
            imported_count += 1;
        }
        
        Ok(imported_count)
    }
    
    /// Clear all patterns
    pub fn clear_all(&mut self) {
        self.patterns.clear();
        self.next_pattern_id = 1;
    }
    
    /// Get pattern count
    pub fn count(&self) -> usize {
        self.patterns.len()
    }
    
    /// Check if a pattern exists
    pub fn has_pattern(&self, pattern_id: &str) -> bool {
        self.patterns.contains_key(pattern_id)
    }
}

impl Default for PatternManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Pattern utility functions
impl PatternManager {
    /// Create a basic kick pattern
    pub fn create_kick_pattern(&mut self) -> String {
        let mut steps = vec![StepCell::default(); 16];
        
        // Set kick hits on steps 1, 5, 9, 13 (classic four-on-the-floor)
        for &step_idx in &[0, 4, 8, 12] {
            steps[step_idx].enabled = true;
            steps[step_idx].velocity = 127;
        }
        
        self.store_pattern(
            "Four on Floor Kick".to_string(),
            steps,
            Some("Classic four-on-the-floor kick pattern".to_string()),
        )
    }
    
    /// Create a basic snare pattern
    pub fn create_snare_pattern(&mut self) -> String {
        let mut steps = vec![StepCell::default(); 16];
        
        // Set snare hits on steps 5, 13 (backbeat)
        for &step_idx in &[4, 12] {
            steps[step_idx].enabled = true;
            steps[step_idx].velocity = 120;
        }
        
        self.store_pattern(
            "Backbeat Snare".to_string(),
            steps,
            Some("Classic backbeat snare pattern".to_string()),
        )
    }
    
    /// Create a basic hi-hat pattern
    pub fn create_hihat_pattern(&mut self) -> String {
        let mut steps = vec![StepCell::default(); 16];
        
        // Set hi-hat hits on every other step
        for step_idx in (1..16).step_by(2) {
            steps[step_idx].enabled = true;
            steps[step_idx].velocity = 80;
        }
        
        self.store_pattern(
            "Eighth Note Hi-Hat".to_string(),
            steps,
            Some("Eighth note hi-hat pattern".to_string()),
        )
    }
    
    /// Create a basic bass pattern
    pub fn create_bass_pattern(&mut self) -> String {
        let mut steps = vec![StepCell::default(); 16];
        
        // Set bass hits with syncopation
        for &step_idx in &[0, 3, 6, 10, 14] {
            steps[step_idx].enabled = true;
            steps[step_idx].velocity = 100;
        }
        
        self.store_pattern(
            "Syncopated Bass".to_string(),
            steps,
            Some("Syncopated bass line pattern".to_string()),
        )
    }
    
    /// Initialize with default patterns
    pub fn init_with_defaults(&mut self) {
        self.create_kick_pattern();
        self.create_snare_pattern();
        self.create_hihat_pattern();
        self.create_bass_pattern();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pattern_storage() {
        let mut manager = PatternManager::new();
        let steps = vec![StepCell::default(); 16];
        
        let pattern_id = manager.store_pattern(
            "Test Pattern".to_string(),
            steps.clone(),
            Some("Test description".to_string()),
        );
        
        assert!(manager.has_pattern(&pattern_id));
        assert_eq!(manager.count(), 1);
        
        let retrieved_pattern = manager.get_pattern(&pattern_id).unwrap();
        assert_eq!(retrieved_pattern.name, "Test Pattern");
        assert_eq!(retrieved_pattern.steps.len(), 16);
    }
    
    #[test]
    fn test_pattern_search() {
        let mut manager = PatternManager::new();
        manager.init_with_defaults();
        
        let kick_patterns = manager.search_patterns("kick");
        assert_eq!(kick_patterns.len(), 1);
        assert!(kick_patterns[0].name.to_lowercase().contains("kick"));
    }
    
    #[test]
    fn test_pattern_bank_export_import() {
        let mut manager1 = PatternManager::new();
        manager1.init_with_defaults();
        
        let bank = manager1.export_bank();
        
        let mut manager2 = PatternManager::new();
        let imported_count = manager2.import_bank(bank).unwrap();
        
        assert_eq!(imported_count, manager1.count());
        assert_eq!(manager2.count(), manager1.count());
    }
}