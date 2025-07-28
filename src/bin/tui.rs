extern crate derive_builder;

use osc::tui::RoscoTuiApp;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Rosco TUI application...");
    
    println!("Creating TUI app instance...");
    let mut app = RoscoTuiApp::new()?;
    println!("TUI app created successfully");
    
    println!("Starting TUI run loop...");
    app.run().await?;
    println!("TUI run completed");
    
    Ok(())
}