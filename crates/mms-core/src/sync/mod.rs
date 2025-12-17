use crate::config::Config;
use crate::db::connection;
use crate::db::models::semester::{Semester, SemesterType};
use crate::db::queries;
use crate::error::Result;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

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
pub fn scan_disk_semesters(config: &Config) -> Result<Vec<DiskSemester>> {
    let base_path = &config.general.university_base_path;

    if !base_path.exists() {
        return Ok(Vec::new());
    }

    let mut disk_semesters = Vec::new();

    for entry in std::fs::read_dir(base_path)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let folder_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string());

        if let Some(folder_name) = folder_name {
            let (parsed_type, parsed_number) = parse_semester_folder(&folder_name);

            disk_semesters.push(DiskSemester {
                folder_name,
                path,
                parsed_type,
                parsed_number,
            });
        }
    }

    Ok(disk_semesters)
}

/// Parse semester folder name like "m01" -> (Master, 1) or "b03" -> (Bachelor, 3)
fn parse_semester_folder(folder_name: &str) -> (Option<SemesterType>, Option<i32>) {
    if folder_name.len() < 3 {
        return (None, None);
    }

    let type_char = folder_name
        .chars()
        .next()
        .unwrap()
        .to_lowercase()
        .next()
        .unwrap();
    let number_str = &folder_name[1..];

    let semester_type = match type_char {
        'm' => Some(SemesterType::Master),
        'b' => Some(SemesterType::Bachelor),
        _ => None,
    };

    let number = number_str.parse::<i32>().ok();

    (semester_type, number)
}

/// Check sync status between database and filesystem
pub fn check_status() -> Result<SyncStatus> {
    let config = Config::load()?;
    let conn = connection::get()?;

    // Get all semesters from database
    let db_semesters = queries::semester::list(&conn)?;

    // Scan filesystem
    let disk_semesters = scan_disk_semesters(&config)?;

    // Build lookup maps
    let mut db_map: HashMap<String, Semester> = db_semesters
        .iter()
        .map(|s| (s.folder_name(), s.clone()))
        .collect();

    let mut disk_set: HashSet<String> = disk_semesters
        .iter()
        .filter(|ds| ds.parsed_type.is_some() && ds.parsed_number.is_some())
        .map(|ds| ds.folder_name.clone())
        .collect();

    // Find semesters in both
    let mut synced_semesters = Vec::new();
    for (folder_name, semester) in &db_map {
        if disk_set.contains(folder_name) {
            synced_semesters.push(semester.clone());
            disk_set.remove(folder_name);
        }
    }

    // Remove synced semesters from db_map
    for semester in &synced_semesters {
        db_map.remove(&semester.folder_name());
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
pub fn sync_to_filesystem(dry_run: bool) -> Result<Vec<String>> {
    let config = Config::load()?;
    let status = check_status()?;
    let mut actions = Vec::new();

    for semester in &status.semesters_in_db_only {
        let semester_path = config
            .general
            .university_base_path
            .join(semester.folder_name());
        let action = format!("Create folder: {}", semester_path.display());
        actions.push(action);

        if !dry_run {
            std::fs::create_dir_all(&semester_path)?;
        }
    }

    Ok(actions)
}
