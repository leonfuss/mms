use mms_core::config::Config;
use mms_core::paths;
use anyhow::Result;
use colored::Colorize;

pub fn handle_init() -> Result<()> {
    if Config::exists() {
        println!("{}", "Config file already exists.".yellow());
        println!("Location: {}", paths::default_config_path()?.display());
        return Ok(())
    }

    println!("{}", "Config file does not exist. Let's create it.".bold());
    println!();

    // Run setup wizard to interactively create config
    let config = crate::cli::setup::ensure_config()?;

    println!();
    println!("Location: {}", paths::default_config_path()?.display());

    Ok(())
}

pub fn handle_show() -> Result<()> {
    let config = Config::load()?;

    println!("{}", "Configuration:".bold().underline());
    println!();

    println!("{}", "General:".bold());
    println!("  University base path: {}", config.general.university_base_path.display());
    println!("  Student Name:         {}", config.general.student_name);
    println!("  Student ID:           {}", config.general.student_id);
    println!("  Default location:     {}", config.general.default_location);
    println!("  Symlink path:         {}", config.general.symlink_path.display());
    println!();

    println!("{}", "Schedule:".bold());
    println!("  Auto-switch:          {}", config.schedule.auto_switch);
    println!("  Switch window:        {} minutes", config.schedule.switch_window_minutes);
    println!("  Notify:               {}", config.schedule.notify);
    println!("  Check interval:       {} minutes", config.schedule.check_interval_minutes);
    println!();

    println!("{}", "Notes:".bold());
    println!("  Auto watch:           {}", config.notes.auto_watch);
    println!("  Auto open PDF:        {}", config.notes.auto_open_pdf);
    println!();

    println!("Config file: {}", paths::default_config_path()?.display().to_string().dimmed());

    Ok(())
}

pub fn handle_edit() -> Result<()> {
    let config_path = paths::default_config_path()?;

    if !config_path.exists() {
        println!("{}", "Config file does not exist. Creating...".yellow());
        handle_init()?;
    }

    // Try to open with default editor
    let config = Config::load()?;
    let editor = std::env::var("EDITOR").unwrap_or(config.general.default_editor);

    println!("Opening config file with {}...", editor);

    std::process::Command::new(editor)
        .arg(&config_path)
        .status()?;

    println!("{}", "✓ Config file updated.".green());

    Ok(())
}