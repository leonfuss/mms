use crate::db::models::semester::{Semester, SemesterType};
use crate::error::Result;
use rusqlite::{Connection, params};

/// Insert a new semester into the database
pub fn insert(conn: &Connection, semester: &Semester) -> Result<i64> {
    conn.execute(
        "INSERT INTO semesters (type, number, is_current, default_location)
         VALUES (?1, ?2, ?3, ?4)",
        params![
            semester.type_.to_str(),
            semester.number,
            semester.is_current,
            semester.default_location,
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

/// Get a semester by ID
pub fn get_by_id(conn: &Connection, id: i64) -> Result<Semester> {
    let mut stmt = conn.prepare(
        "SELECT id, type, number, is_current, default_location, created_at
         FROM semesters
         WHERE id = ?1",
    )?;

    let semester = stmt.query_row([id], |row| {
        let type_str: String = row.get(1)?;
        Ok(Semester {
            id: Some(row.get(0)?),
            type_: SemesterType::from_str(&type_str).unwrap(),
            number: row.get(2)?,
            is_current: row.get(3)?,
            default_location: row.get(4)?,
            created_at: row.get(5)?,
        })
    })?;

    Ok(semester)
}

/// List all semesters
pub fn list(conn: &Connection) -> Result<Vec<Semester>> {
    let mut stmt = conn.prepare(
        "SELECT id, type, number, is_current, default_location, created_at
         FROM semesters
         ORDER BY type DESC, number DESC",
    )?;

    let semesters = stmt
        .query_map([], |row| {
            let type_str: String = row.get(1)?;
            Ok(Semester {
                id: Some(row.get(0)?),
                type_: SemesterType::from_str(&type_str).unwrap(),
                number: row.get(2)?,
                is_current: row.get(3)?,
                default_location: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(semesters)
}

/// Get the current semester
pub fn get_current(conn: &Connection) -> Result<Option<Semester>> {
    let mut stmt = conn.prepare(
        "SELECT id, type, number, is_current, default_location, created_at
         FROM semesters
         WHERE is_current = 1
         LIMIT 1",
    )?;

    let result = stmt.query_row([], |row| {
        let type_str: String = row.get(1)?;
        Ok(Semester {
            id: Some(row.get(0)?),
            type_: SemesterType::from_str(&type_str).unwrap(),
            number: row.get(2)?,
            is_current: row.get(3)?,
            default_location: row.get(4)?,
            created_at: row.get(5)?,
        })
    });

    match result {
        Ok(semester) => Ok(Some(semester)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Set a semester as current (and unset all others)
pub fn set_current(conn: &Connection, id: i64) -> Result<()> {
    // First verify the semester exists
    get_by_id(conn, id)?;

    // Unset all semesters
    conn.execute("UPDATE semesters SET is_current = 0", [])?;

    // Set the specified semester as current
    conn.execute("UPDATE semesters SET is_current = 1 WHERE id = ?1", [id])?;

    Ok(())
}

/// Update a semester
pub fn update(conn: &Connection, semester: &Semester) -> Result<()> {
    let id = semester.id.ok_or_else(|| {
        crate::error::MmsError::Other("Cannot update semester without id".to_string())
    })?;

    conn.execute(
        "UPDATE semesters
         SET type = ?1, number = ?2, is_current = ?3, default_location = ?4
         WHERE id = ?5",
        params![
            semester.type_.to_str(),
            semester.number,
            semester.is_current,
            semester.default_location,
            id,
        ],
    )?;

    Ok(())
}

/// Delete a semester
pub fn delete(conn: &Connection, id: i64) -> Result<()> {
    let rows_affected = conn.execute("DELETE FROM semesters WHERE id = ?1", [id])?;

    if rows_affected == 0 {
        return Err(crate::error::MmsError::SemesterNotFound(id));
    }

    Ok(())
}
