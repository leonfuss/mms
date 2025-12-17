mod cli;

use clap::Parser;
use cli::commands;
use cli::{Cli, Commands};
use colored::Colorize;
use anyhow::Result;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Config { action } => match action {
            cli::args::ConfigAction::Init => commands::config::handle_init(),
            cli::args::ConfigAction::Show => commands::config::handle_show(),
            cli::args::ConfigAction::Edit => commands::config::handle_edit(),
        },
        Commands::Semester { action } => commands::semester::handle(action),
        Commands::Course { action } => commands::course::handle(action),
        Commands::Schedule { action } => commands::schedule::handle(action),
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
        Commands::Service { action } => commands::service::handle(action),
        Commands::Status => commands::status::handle_status(),
        Commands::Sync { dry_run } => commands::status::handle_sync(dry_run),
        Commands::Today => commands::today::handle(),
    }
}
