use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;

use crate::config::Config;
use crate::db::connection_seaorm;
use crate::db::queries;
use crate::error::{MmsError, Result};
use crate::service::scheduler::ScheduleEngine;
use crate::symlink;
use sea_orm::DatabaseConnection;

/// Daemon that runs in the background and automatically switches courses
pub struct Daemon {
    check_interval: Duration,
    pid_file: PathBuf,
}

impl Daemon {
    /// Create a new daemon instance
    pub fn new() -> Result<Self> {
        let config = Config::load()?;

        // Get schedule config or return error
        let schedule = config.schedule.as_ref().ok_or(MmsError::ScheduleNotSet)?;

        let check_interval = Duration::from_secs(schedule.check_interval_minutes * 60);
        let pid_file = Self::get_pid_file_path()?;

        Ok(Self {
            check_interval,
            pid_file,
        })
    }

    /// Start the daemon loop
    pub async fn run(&self) -> Result<()> {
        // Check if another instance is already running
        if self.is_running()? {
            return Err(MmsError::Other(
                "Daemon is already running. Use 'mms service stop' to stop it first.".to_string(),
            ));
        }

        // Write PID file
        self.write_pid_file()?;

        println!("Starting MMS daemon...");
        println!(
            "Check interval: {} minutes",
            self.check_interval.as_secs() / 60
        );
        println!("PID file: {}", self.pid_file.display());

        // Set up signal handlers for graceful shutdown
        let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
        let r = running.clone();

        ctrlc::set_handler(move || {
            println!("\nReceived shutdown signal. Stopping daemon...");
            r.store(false, std::sync::atomic::Ordering::SeqCst);
        })
        .map_err(|e| MmsError::Other(format!("Failed to set signal handler: {}", e)))?;

        // Main daemon loop
        while running.load(std::sync::atomic::Ordering::SeqCst) {
            if let Err(e) = self.check_and_update().await {
                eprintln!("Error checking schedule: {}", e);
                // Continue running despite errors
            }

            // Sleep for the configured interval
            sleep(self.check_interval).await;
        }

        // Cleanup on exit
        self.cleanup()?;
        println!("Daemon stopped.");

        Ok(())
    }

    /// Check schedule and update active course if needed
    async fn check_and_update(&self) -> Result<()> {
        eprintln!("[DEBUG] Starting check_and_update...");

        let conn = connection_seaorm::get_connection().await.map_err(|e| {
            eprintln!("[DEBUG] Failed to get connection: {}", e);
            MmsError::Database(e)
        })?;

        eprintln!("[DEBUG] Got database connection");

        // Get current active course
        let active = queries::active::get(&conn).await.map_err(|e| {
            eprintln!("[DEBUG] Failed to get active state: {}", e);
            e
        })?;
        let current_course_id = active.course_id;

        eprintln!("[DEBUG] Current course_id: {:?}", current_course_id);

        // Determine what course should be active now
        let should_be_active = ScheduleEngine::determine_active_course_now(&conn)
            .await
            .map_err(|e| {
                eprintln!("[DEBUG] Failed to determine active course: {}", e);
                e
            })?;

        // Debug logging
        eprintln!(
            "[DEBUG] Current: {:?}, Should be: {:?}",
            current_course_id, should_be_active
        );

        // Check if we need to switch
        if current_course_id != should_be_active {
            eprintln!("[DEBUG] Switching course...");
            self.switch_course(&conn, current_course_id, should_be_active)
                .await?;
        }

        Ok(())
    }

    /// Switch to a different active course
    async fn switch_course(
        &self,
        conn: &DatabaseConnection,
        from: Option<i64>,
        to: Option<i64>,
    ) -> Result<()> {
        match (from, to) {
            (Some(old_id), Some(new_id)) => {
                let old_course = queries::course::get_by_id(conn, old_id).await?;
                let new_course = queries::course::get_by_id(conn, new_id).await?;
                let semester = queries::semester::get_by_id(conn, new_course.semester_id).await?;

                // Update active course
                queries::active::set_active_course(conn, new_id, new_course.semester_id).await?;

                // Update symlinks
                symlink::update_semester_symlink(&semester.directory_path)?;
                symlink::update_course_symlink(&semester.directory_path, &new_course.short_name)?;

                println!(
                    "[{}] Switched course: {} -> {}",
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                    old_course.name,
                    new_course.name
                );
            }
            (Some(old_id), None) => {
                let old_course = queries::course::get_by_id(conn, old_id).await?;

                // Clear active course
                queries::active::clear_active_course(conn).await?;

                // Remove course symlink but keep semester symlink
                let _ = symlink::remove_course_symlink();

                println!(
                    "[{}] No active course (was: {})",
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                    old_course.name
                );
            }
            (None, Some(new_id)) => {
                let new_course = queries::course::get_by_id(conn, new_id).await?;
                let semester = queries::semester::get_by_id(conn, new_course.semester_id).await?;

                // Set active course
                queries::active::set_active_course(conn, new_id, new_course.semester_id).await?;

                // Update symlinks
                symlink::update_semester_symlink(&semester.directory_path)?;
                symlink::update_course_symlink(&semester.directory_path, &new_course.short_name)?;

                println!(
                    "[{}] Course started: {}",
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                    new_course.name
                );
            }
            (None, None) => {
                // No change, nothing to do
            }
        }

        Ok(())
    }

    /// Check if daemon is already running
    pub fn is_running(&self) -> Result<bool> {
        if !self.pid_file.exists() {
            return Ok(false);
        }

        let pid_str = fs::read_to_string(&self.pid_file)?;
        let pid: u32 = pid_str
            .trim()
            .parse()
            .map_err(|_| MmsError::Other("Invalid PID file".to_string()))?;

        // Check if process with this PID exists
        Ok(Self::process_exists(pid))
    }

    /// Stop the daemon
    pub fn stop(&self) -> Result<()> {
        if !self.is_running()? {
            return Err(MmsError::Other("Daemon is not running".to_string()));
        }

        let pid_str = fs::read_to_string(&self.pid_file)?;
        let pid: u32 = pid_str
            .trim()
            .parse()
            .map_err(|_| MmsError::Other("Invalid PID file".to_string()))?;

        // Send SIGTERM to the process
        Self::kill_process(pid)?;

        // Wait for process to exit
        for _ in 0..50 {
            if !Self::process_exists(pid) {
                // Remove PID file
                fs::remove_file(&self.pid_file)?;
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(100));
        }

        Err(MmsError::Other("Failed to stop daemon".to_string()))
    }

    /// Get daemon status
    pub fn status(&self) -> Result<DaemonStatus> {
        if !self.is_running()? {
            return Ok(DaemonStatus::Stopped);
        }

        let pid_str = fs::read_to_string(&self.pid_file)?;
        let pid: u32 = pid_str
            .trim()
            .parse()
            .map_err(|_| MmsError::Other("Invalid PID file".to_string()))?;

        Ok(DaemonStatus::Running { pid })
    }

    /// Write PID file
    fn write_pid_file(&self) -> Result<()> {
        let pid = std::process::id();

        // Create parent directory if it doesn't exist
        if let Some(parent) = self.pid_file.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&self.pid_file, pid.to_string())?;
        Ok(())
    }

    /// Cleanup on exit
    fn cleanup(&self) -> Result<()> {
        if self.pid_file.exists() {
            fs::remove_file(&self.pid_file)?;
        }
        Ok(())
    }

    /// Get PID file path
    fn get_pid_file_path() -> Result<PathBuf> {
        let data_dir = dirs::data_local_dir()
            .ok_or_else(|| MmsError::Other("Could not determine data directory".to_string()))?;
        Ok(data_dir.join("mms").join("daemon.pid"))
    }

    /// Check if a process with given PID exists
    #[cfg(unix)]
    fn process_exists(pid: u32) -> bool {
        let output = std::process::Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .output();

        match output {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    /// Kill a process by PID
    #[cfg(unix)]
    fn kill_process(pid: u32) -> Result<()> {
        std::process::Command::new("kill")
            .arg("-TERM")
            .arg(pid.to_string())
            .output()
            .map_err(|e| MmsError::Other(format!("Failed to send signal: {}", e)))?;
        Ok(())
    }

    #[cfg(not(unix))]
    fn process_exists(_pid: u32) -> bool {
        // On Windows, always return false for now
        false
    }

    #[cfg(not(unix))]
    fn kill_process(_pid: u32) -> Result<()> {
        Err(MmsError::Other("Windows is not yet supported".to_string()))
    }
}

#[derive(Debug)]
pub enum DaemonStatus {
    Running { pid: u32 },
    Stopped,
}
