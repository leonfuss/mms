use anyhow::Result;
use mms_core::sync;
use colored::Colorize;

pub fn handle_status() -> Result<()> {
    let conn = mms_core::db::connection::get()?;

    println!("{}", "System Status".bold().underline());
    println!();

    // Get active state
    let active = mms_core::db::queries::active::get(&conn)?;

    // Show active semester
    if let Some(semester_id) = active.semester_id {
        let semester = mms_core::db::queries::semester::get_by_id(&conn, semester_id)?;
        println!("{} {}", "Active Semester:".bold(), semester.to_string().green());
    } else {
        println!("{} {}", "Active Semester:".bold(), "None".yellow());
    }

    // Show active course
    if let Some(course_id) = active.course_id {
        let course = mms_core::db::queries::course::get_by_id(&conn, course_id)?;
        println!("{} {}", "Active Course:  ".bold(), course.name.green());
    } else {
        println!("{} {}", "Active Course:  ".bold(), "None".yellow());
    }

    // Show symlink status
    let (cs_target, cc_target) = mms_core::symlink::check_symlinks()?;
    println!();
    println!("{}", "Symlinks:".bold());
    if let Some(target) = cs_target {
        println!("  ~/cs -> {}", target.display().to_string().green());
    } else {
        println!("  ~/cs -> {}", "Not set".yellow());
    }
    if let Some(target) = cc_target {
        println!("  ~/cc -> {}", target.display().to_string().green());
    } else {
        println!("  ~/cc -> {}", "Not set".yellow());
    }

    println!();
    println!("{}", "Sync Status".bold().underline());
    println!();

    let status = sync::check_status()?;

    if status.is_synced() {
        println!("{}", "✓ Everything is in sync!".green().bold());
        println!();
        println!("Semesters synced: {}", status.synced_semesters.len());
        return Ok(());
    }

    // Show synced semesters
    if !status.synced_semesters.is_empty() {
        println!("{}", "✓ Synced Semesters:".green().bold());
        for semester in &status.synced_semesters {
            println!("  {} [{}]", semester.to_string(), semester.folder_name().dimmed());
        }
        println!();
    }

    // Show semesters in DB but missing folders
    if !status.semesters_in_db_only.is_empty() {
        println!("{}", "⚠ Semesters in database without folders:".yellow().bold());
        for semester in &status.semesters_in_db_only {
            let expected_path = mms_core::config::Config::load()?
                .general
                .university_base_path
                .join(semester.folder_name());
            println!(
                "  {} [{}] - Missing: {}",
                semester.to_string(),
                semester.folder_name().dimmed(),
                expected_path.display().to_string().red()
            );
        }
        println!();
    }

    // Show folders on disk not in DB
    if !status.semesters_on_disk_only.is_empty() {
        println!("{}", "⚠ Folders on disk not in database:".yellow().bold());
        for disk_sem in &status.semesters_on_disk_only {
            if disk_sem.parsed_type.is_some() && disk_sem.parsed_number.is_some() {
                println!(
                    "  {} - {}",
                    disk_sem.folder_name.bold(),
                    disk_sem.path.display().to_string().dimmed()
                );
            } else {
                println!(
                    "  {} - {} {}",
                    disk_sem.folder_name.bold(),
                    disk_sem.path.display().to_string().dimmed(),
                    "(invalid format)".red()
                );
            }
        }
        println!();
    }

    // Show summary
    println!("{}", "Summary:".bold().underline());
    println!("  Synced:                    {}", status.synced_semesters.len());
    println!("  In DB only (need folders): {}", status.semesters_in_db_only.len());
    println!("  On disk only (not in DB):  {}", status.semesters_on_disk_only.len());
    println!();

    if !status.semesters_in_db_only.is_empty() {
        println!("{}", "Run 'mms sync' to create missing folders.".cyan());
    }

    Ok(())
}

pub fn handle_sync(dry_run: bool) -> Result<()> {
    if dry_run {
        println!("{}", "Dry-run mode: No changes will be made".yellow().bold());
        println!();
    }

    let actions = sync::sync_to_filesystem(dry_run)?;

    if actions.is_empty() {
        println!("{}", "✓ Nothing to sync!".green().bold());
        return Ok(());
    }

    println!("{}", "Sync actions:".bold());
    for action in &actions {
        if dry_run {
            println!("  [DRY-RUN] {}", action.dimmed());
        } else {
            println!("  ✓ {}", action);
        }
    }
    println!();

    if dry_run {
        println!("{}", "Run 'mms sync' without --dry-run to apply changes.".cyan());
    } else {
        println!("{}", "✓ Sync completed!".green().bold());
    }

    Ok(())
}
