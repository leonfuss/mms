mod cli;

use clap::Parser;
use cli::commands;
use cli::{Cli, Commands};
use colored::Colorize;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Startup: Ensure configuration exists and is valid
    // This will prompt the user if config is missing or invalid
    let _config = cli::setup::ensure_config()?;

    // 2. Parse arguments
    let cli = Cli::parse();

    // 3. Execute commands
    match cli.command {
        Commands::Config { action } => match action {
            cli::args::ConfigAction::Init => commands::config::handle_init(),
            cli::args::ConfigAction::Show => commands::config::handle_show(),
            cli::args::ConfigAction::Edit => commands::config::handle_edit(),
        },
        Commands::Service { action } => commands::service::handle(action).await,
        
        // TODO: Refactor other commands to async/SeaORM
        Commands::Semester { action: _ } => {
            println!("{}", "Command 'semester' is currently being refactored.".yellow());
            Ok(())
        }
        Commands::Course { action: _ } => {
            println!("{}", "Command 'course' is currently being refactored.".yellow());
            Ok(())
        }
        Commands::Schedule { action: _ } => {
            println!("{}", "Command 'schedule' is currently being refactored.".yellow());
            Ok(())
        }
        Commands::Todo { action } => {
            println!("{}", "TODO: Todo commands not yet implemented".yellow());
            println!("Action: {:?}", action);
            Ok(())
        }
        Commands::Lecture { action } => {
            println!("{}", "TODO: Lecture commands not yet implemented".yellow());
            println!("Action: {:?}", action);
            Ok(())
        }
        Commands::Exam { action } => {
            println!("{}", "TODO: Exam commands not yet implemented".yellow());
            println!("Action: {:?}", action);
            Ok(())
        }
        Commands::Holiday { action } => {
            println!("{}", "TODO: Holiday commands not yet implemented".yellow());
            println!("Action: {:?}", action);
            Ok(())
        }
        Commands::Stats { action } => {
            println!("{}", "TODO: Stats commands not yet implemented".yellow());
            println!("Action: {:?}", action);
            Ok(())
        }
        
        Commands::Status => {
             // commands::status::handle_status()
             println!("{}", "Command 'status' is currently being refactored.".yellow());
             Ok(())
        },
        Commands::Sync { dry_run: _ } => {
             // commands::status::handle_sync(dry_run)
             println!("{}", "Command 'sync' is currently being refactored.".yellow());
             Ok(())
        },
        Commands::Today => {
             // commands::today::handle()
             println!("{}", "Command 'today' is currently being refactored.".yellow());
             Ok(())
        },
    }
}