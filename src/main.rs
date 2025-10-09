mod config;
mod db;
mod error;

use config::Config;
use error::Result;

fn main() -> Result<()> {
    println!("MMS - My Study Management System");
    println!("Data structures initialized successfully!");

    // Test config loading
    let config = Config::load()?;
    println!("\nConfig loaded:");
    println!("  Default location: {}", config.general.default_location);
    println!("  Symlink path: {}", config.general.symlink_path.display());
    println!("  Check interval: {} minutes", config.service.schedule_check_interval_minutes);
    println!("  Categories defined: {}", config.categories.len());

    if !Config::exists() {
        println!("\nNote: No config file found. Using defaults.");
        println!("Run 'mms config init' to create a config file.");
    }

    Ok(())
}
