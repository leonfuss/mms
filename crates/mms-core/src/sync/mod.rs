use crate::config::Config;
use crate::db::connection_seaorm; // Changed
use crate::db::entities::semesters::Model as Semester; // Use SeaORM model
use crate::db::queries;
use crate::error::{MmsError, Result}; // Import MmsError
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)] // Redefined locally or move to a shared types module
pub enum SemesterType {
    Bachelor,
    Master,
}

#[derive(Debug, Clone)]
pub struct SyncStatus {
    pub semesters_in_db_only: Vec<Semester>,
    pub semesters_on_disk_only: Vec<DiskSemester>,
    pub synced_semesters: Vec<Semester>,
}

#[derive(Debug, Clone)]
pub struct DiskSemester {
    pub folder_name: String,
    pub path: PathBuf,
    pub parsed_type: Option<SemesterType>,
    pub parsed_number: Option<i32>,
}

impl SyncStatus {
    pub fn is_synced(&self) -> bool {
        self.semesters_in_db_only.is_empty() && self.semesters_on_disk_only.is_empty()
    }
}

/// Scan the filesystem for semester folders
///
/// This function iterates over the directory entries in the configured university base path.
/// It filters for directories, attempts to parse their names as semester folders,
/// and returns a list of `DiskSemester` structs.
///
/// # Preconditions
/// - `config.university_base_path()` must return a valid path (validated on config load).
///
/// # Postconditions
/// - Returns a `Result` containing a `Vec<DiskSemester>`.
/// - If the base path does not exist, returns an empty vector.
/// - Only directories are included.
pub fn scan_disk_semesters(config: &Config) -> Result<Vec<DiskSemester>> {
    let base_path = &config.university_base_path;

    if !base_path.exists() {
        return Ok(Vec::new());
    }

    let disk_semesters = std::fs::read_dir(base_path)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();

            if !path.is_dir() {
                return None;
            }

            let folder_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())?;

            let (parsed_type, parsed_number) = parse_semester_folder(&folder_name);

            Some(DiskSemester {
                folder_name,
                path,
                parsed_type,
                parsed_number,
            })
        })
        .collect();

    Ok(disk_semesters)
}

/// Parse semester folder name like "m01" -> (Master, 1) or "b03" -> (Bachelor, 3)
fn parse_semester_folder(folder_name: &str) -> (Option<SemesterType>, Option<i32>) {
    // Contract: folder_name must have at least 2 chars to possibly be valid (e.g. "b1")
    if folder_name.len() < 2 {
        return (None, None);
    }

    let mut chars = folder_name.chars();
    let type_char = chars.next().unwrap_or_default().to_ascii_lowercase();
    let number_str = chars.as_str();

    let semester_type = match type_char {
        'm' => Some(SemesterType::Master),
        'b' => Some(SemesterType::Bachelor),
        _ => None,
    };

    let number = number_str.parse::<i32>().ok();

    (semester_type, number)
}

/// Check sync status between database and filesystem
pub async fn check_status() -> Result<SyncStatus> {
    // Async
    let config = Config::load()?;
    let conn = connection_seaorm::get_connection()
        .await
        .map_err(MmsError::Database)?; // Async connection

    // Get all semesters from database
    let db_semesters = queries::semester::list(&conn).await?; // Async query

    // Scan filesystem
    let disk_semesters = scan_disk_semesters(&config)?;

    // Build lookup maps
    let mut db_map: HashMap<String, Semester> = db_semesters
        .iter()
        .map(|s| (s.directory_path.clone(), s.clone())) // Use directory_path as folder name
        .collect();

    let mut disk_set: HashSet<String> = disk_semesters
        .iter()
        .filter(|ds| ds.parsed_type.is_some() && ds.parsed_number.is_some())
        .map(|ds| ds.folder_name.clone())
        .collect();

    // Find semesters in both
    let mut synced_semesters = Vec::new();
    // Iterating over keys of db_map is tricky while modifying it.
    // Let's iterate over a clone of keys or just check disk_set against db_map keys.

    let db_folders: Vec<String> = db_map.keys().cloned().collect();
    for folder_name in db_folders {
        if disk_set.contains(&folder_name) {
            if let Some(semester) = db_map.get(&folder_name) {
                synced_semesters.push(semester.clone());
            }
            // We want to remove it from disk_set (to find disk-only ones)
            // and remove from db_map (to find db-only ones).
            disk_set.remove(&folder_name);
            db_map.remove(&folder_name);
        }
    }

    // What's left in db_map is in DB only
    let semesters_in_db_only: Vec<Semester> = db_map.into_values().collect();

    // What's left in disk_set is on disk only
    let semesters_on_disk_only: Vec<DiskSemester> = disk_semesters
        .into_iter()
        .filter(|ds| disk_set.contains(&ds.folder_name))
        .collect();

    Ok(SyncStatus {
        semesters_in_db_only,
        semesters_on_disk_only,
        synced_semesters,
    })
}

/// Sync filesystem with database (create missing folders)
pub async fn sync_to_filesystem(dry_run: bool) -> Result<Vec<String>> {
    // Async
    let config = Config::load()?;
    let status = check_status().await?; // Async await
    let mut actions = Vec::new();

    for semester in &status.semesters_in_db_only {
        let semester_path = config.university_base_path.join(&semester.directory_path); // Use directory_path
        let action = format!("Create folder: {}", semester_path.display());
        actions.push(action);

        if !dry_run {
            std::fs::create_dir_all(&semester_path)?;
        }
    }

    Ok(actions)
}
