use crate::cli::args::ServiceAction;
use crate::error::{MmsError, Result};
use crate::service::{Daemon, DaemonStatus};
use colored::Colorize;
use std::fs;
use std::path::PathBuf;

pub fn handle(action: ServiceAction) -> Result<()> {
    match action {
        ServiceAction::Install => handle_install(),
        ServiceAction::Uninstall => handle_uninstall(),
        ServiceAction::Start => handle_start(),
        ServiceAction::Stop => handle_stop(),
        ServiceAction::Status => handle_status(),
        ServiceAction::Run => handle_run(),
    }
}

fn handle_install() -> Result<()> {
    #[cfg(not(target_os = "macos"))]
    {
        return Err(MmsError::Other(
            "Auto-start installation is only supported on macOS".to_string()
        ));
    }

    #[cfg(target_os = "macos")]
    {
        println!("{}", "Installing MMS background service...".bold());
        println!();

        // Get paths
        let plist_template = include_str!("../../../resources/com.mms.daemon.plist");
        let home_dir = dirs::home_dir()
            .ok_or_else(|| MmsError::Other("Could not determine home directory".to_string()))?;
        let launch_agents_dir = home_dir.join("Library/LaunchAgents");
        let plist_path = launch_agents_dir.join("com.mms.daemon.plist");
        let binary_path = std::env::current_exe()?;
        let log_dir = dirs::data_local_dir()
            .ok_or_else(|| MmsError::Other("Could not determine data directory".to_string()))?
            .join("mms");

        // Create log directory
        fs::create_dir_all(&log_dir)?;

        // Create LaunchAgents directory if it doesn't exist
        fs::create_dir_all(&launch_agents_dir)?;

        // Replace placeholders in template
        let plist_content = plist_template
            .replace("{{BINARY_PATH}}", &binary_path.to_string_lossy())
            .replace("{{LOG_PATH}}", &log_dir.to_string_lossy())
            .replace("{{HOME}}", &home_dir.to_string_lossy());

        // Write plist file
        fs::write(&plist_path, plist_content)?;

        println!("{}", "✓ Plist file created".green());
        println!("  Location: {}", plist_path.display().to_string().dimmed());
        println!();

        // Load the service
        let output = std::process::Command::new("launchctl")
            .arg("load")
            .arg(&plist_path)
            .output()?;

        if output.status.success() {
            println!("{}", "✓ Service installed and loaded!".green());
            println!();
            println!("The MMS daemon will now start automatically on login.");
            println!("Logs: {}", log_dir.join("mms-daemon.log").display().to_string().dimmed());
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("{}", "✗ Failed to load service".red());
            println!("Error: {}", stderr);
        }

        Ok(())
    }
}

fn handle_uninstall() -> Result<()> {
    #[cfg(not(target_os = "macos"))]
    {
        return Err(MmsError::Other(
            "Auto-start uninstallation is only supported on macOS".to_string()
        ));
    }

    #[cfg(target_os = "macos")]
    {
        println!("{}", "Uninstalling MMS background service...".bold());
        println!();

        let home_dir = dirs::home_dir()
            .ok_or_else(|| MmsError::Other("Could not determine home directory".to_string()))?;
        let plist_path = home_dir.join("Library/LaunchAgents/com.mms.daemon.plist");

        if !plist_path.exists() {
            println!("{}", "Service is not installed.".yellow());
            return Ok(());
        }

        // Unload the service
        let output = std::process::Command::new("launchctl")
            .arg("unload")
            .arg(&plist_path)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("{}", "Warning: Failed to unload service".yellow());
            println!("Error: {}", stderr);
            println!();
        }

        // Remove plist file
        fs::remove_file(&plist_path)?;

        println!("{}", "✓ Service uninstalled!".green());
        println!();
        println!("The MMS daemon will no longer start automatically on login.");

        Ok(())
    }
}

fn handle_start() -> Result<()> {
    let daemon = Daemon::new()?;

    // Check if already running
    match daemon.status()? {
        DaemonStatus::Running { pid } => {
            println!("{}", "Service is already running!".yellow());
            println!("  PID: {}", pid);
            return Ok(());
        }
        DaemonStatus::Stopped => {}
    }

    // Fork and run in background
    #[cfg(unix)]
    {
        use std::process::{Command, Stdio};

        let exe = std::env::current_exe()?;

        // Spawn the daemon in background using 'mms service run'
        Command::new(exe)
            .arg("service")
            .arg("run")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        // Give it a moment to start
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Check if it started successfully
        match daemon.status()? {
            DaemonStatus::Running { pid } => {
                println!("{}", "✓ Service started successfully!".green());
                println!("  PID: {}", pid);
            }
            DaemonStatus::Stopped => {
                println!("{}", "✗ Failed to start service".red());
            }
        }
    }

    #[cfg(not(unix))]
    {
        println!("{}", "Starting service in background is not supported on this platform.".yellow());
        println!("Use 'mms service run' to run in foreground.");
    }

    Ok(())
}

fn handle_stop() -> Result<()> {
    let daemon = Daemon::new()?;

    match daemon.status()? {
        DaemonStatus::Stopped => {
            println!("{}", "Service is not running.".yellow());
            return Ok(());
        }
        DaemonStatus::Running { pid } => {
            println!("Stopping service (PID: {})...", pid);
            daemon.stop()?;
            println!("{}", "✓ Service stopped successfully!".green());
        }
    }

    Ok(())
}

fn handle_status() -> Result<()> {
    let daemon = Daemon::new()?;

    match daemon.status()? {
        DaemonStatus::Running { pid } => {
            println!("{}", "Service Status:".bold().underline());
            println!();
            println!("  Status: {}", "Running".green().bold());
            println!("  PID:    {}", pid);
        }
        DaemonStatus::Stopped => {
            println!("{}", "Service Status:".bold().underline());
            println!();
            println!("  Status: {}", "Stopped".red().bold());
            println!();
            println!("Use 'mms service start' to start the service.");
        }
    }

    Ok(())
}

fn handle_run() -> Result<()> {
    println!("{}", "Starting MMS service...".bold());
    println!();

    let daemon = Daemon::new()?;
    daemon.run()?;

    Ok(())
}
