use mms_core::config::Config;
use anyhow::Result;
use colored::Colorize;

pub fn handle_init() -> Result<()> {
    if Config::exists() {
        println!("{}", "Config file already exists.".yellow());
        println!("Location: {}", Config::default_config_path()?.display());
        return Ok(());
    }

    let config = Config::init_default_config()?;
    config.save()?;

    println!("{}", "✓ Config file created successfully!".green());
    println!("Location: {}", Config::default_config_path()?.display());
    println!("\nDefault categories added:");
    for (name, cat) in &config.categories {
        println!(
            "  {} - {} ECTS{}",
            name.bold(),
            cat.required_ects,
            if cat.counts_towards_average {
                " (counts towards average)"
            } else {
                ""
            }
        );
    }
    println!("\nEdit the config file to customize settings.");

    Ok(())
}

pub fn handle_show() -> Result<()> {
    let config = Config::load()?;

    println!("{}", "Configuration:".bold().underline());
    println!();

    println!("{}", "General:".bold());
    println!("  University base path: {}", config.general.university_base_path.display());
    println!("  Default location:     {}", config.general.default_location);
    println!("  Symlink path:         {}", config.general.symlink_path.display());
    println!();

    println!("{}", "Service:".bold());
    println!("  Schedule check interval: {} minutes", config.service.schedule_check_interval_minutes);
    println!("  Auto-commit on lecture end: {}", config.service.auto_commit_on_lecture_end);
    println!("  Auto-clear todos: {}", config.service.auto_clear_todos_on_next_lecture);
    println!();

    println!("{}", "Git:".bold());
    println!("  Author name:  {}", config.git.author_name);
    println!("  Author email: {}", config.git.author_email);
    println!();

    if !config.categories.is_empty() {
        println!("{}", "Categories:".bold());
        for (name, cat) in &config.categories {
            println!(
                "  {} - {} ECTS{}",
                name,
                cat.required_ects,
                if cat.counts_towards_average {
                    " (counts towards average)".dimmed()
                } else {
                    " (does not count)".dimmed()
                }
            );
        }
        println!();
    }

    println!("Config file: {}", Config::default_config_path()?.display().to_string().dimmed());

    Ok(())
}

pub fn handle_edit() -> Result<()> {
    let config_path = Config::default_config_path()?;

    if !config_path.exists() {
        println!("{}", "Config file does not exist. Creating...".yellow());
        handle_init()?;
    }

    // Try to open with default editor
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());

    println!("Opening config file with {}...", editor);

    std::process::Command::new(editor)
        .arg(&config_path)
        .status()?;

    println!("{}", "✓ Config file updated.".green());

    Ok(())
}
