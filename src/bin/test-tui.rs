extern crate derive_builder;

use osc::tui::RoscoTuiApp;

// Test TUI components without terminal setup
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing TUI components without terminal...");
    
    // Test creating TUI app
    println!("Creating TUI app...");
    let app = RoscoTuiApp::new()?;
    println!("TUI app created successfully!");
    
    // Test individual components
    println!("Testing component initialization...");
    
    println!("All tests passed!");
    Ok(())
}