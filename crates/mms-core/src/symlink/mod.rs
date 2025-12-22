use crate::config::Config;
use crate::error::Result;
use std::os::unix::fs as unix_fs;
use std::path::PathBuf;

/// Create or update the cs (current semester) symlink
pub fn update_semester_symlink(semester_folder_name: &str) -> Result<()> {
    let config = Config::load()?;
    let symlink_path = get_semester_symlink_path(&config);

    // Target path
    let target_path = config
        .general
        .university_base_path
        .join(semester_folder_name);

    // Remove existing symlink if it exists
    if symlink_path.exists() || symlink_path.is_symlink() {
        std::fs::remove_file(&symlink_path)?;
    }

    // Create new symlink
    unix_fs::symlink(&target_path, &symlink_path)?;

    Ok(())
}

/// Create or update the cc (current course) symlink
pub fn update_course_symlink(semester_folder_name: &str, course_folder_name: &str) -> Result<()> {
    let config = Config::load()?;
    let symlink_path = get_course_symlink_path(&config);

    // Target path
    let target_path = config
        .general
        .university_base_path
        .join(semester_folder_name)
        .join(course_folder_name);

    // Remove existing symlink if it exists
    if symlink_path.exists() || symlink_path.is_symlink() {
        std::fs::remove_file(&symlink_path)?;
    }

    // Create new symlink
    unix_fs::symlink(&target_path, &symlink_path)?;

    Ok(())
}

/// Remove the cs (current semester) symlink
pub fn remove_semester_symlink() -> Result<()> {
    let config = Config::load()?;
    let symlink_path = get_semester_symlink_path(&config);

    if symlink_path.exists() || symlink_path.is_symlink() {
        std::fs::remove_file(&symlink_path)?;
    }

    Ok(())
}

/// Remove the cc (current course) symlink
pub fn remove_course_symlink() -> Result<()> {
    let config = Config::load()?;
    let symlink_path = get_course_symlink_path(&config);

    if symlink_path.exists() || symlink_path.is_symlink() {
        std::fs::remove_file(&symlink_path)?;
    }

    Ok(())
}

/// Get the path where the semester symlink should be created
fn get_semester_symlink_path(config: &Config) -> PathBuf {
    config.general.symlink_path.join("cs").into()
}

/// Get the path where the course symlink should be created
fn get_course_symlink_path(config: &Config) -> PathBuf {
    config.general.symlink_path.join("cc").into()
}

/// Check if symlinks are correctly set up
pub fn check_symlinks() -> Result<(Option<PathBuf>, Option<PathBuf>)> {
    let config = Config::load()?;

    let cs_path = get_semester_symlink_path(&config);
    let cc_path = get_course_symlink_path(&config);

    let cs_target = if cs_path.is_symlink() {
        Some(std::fs::read_link(&cs_path)?)
    } else {
        None
    };

    let cc_target = if cc_path.is_symlink() {
        Some(std::fs::read_link(&cc_path)?)
    } else {
        None
    };

    Ok((cs_target, cc_target))
}
