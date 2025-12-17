use rusqlite::{Connection, params};
use crate::db::models::active::Active;
use crate::error::Result;

/// Get the current active state
pub fn get(conn: &Connection) -> Result<Active> {
    let mut stmt = conn.prepare(
        "SELECT id, semester_id, course_id, lecture_id, activated_at
         FROM active
         WHERE id = 1"
    )?;

    let active = stmt.query_row([], |row| {
        Ok(Active {
            id: row.get(0)?,
            semester_id: row.get(1)?,
            course_id: row.get(2)?,
            lecture_id: row.get(3)?,
            activated_at: row.get(4)?,
        })
    })?;

    Ok(active)
}

/// Set the active semester
pub fn set_active_semester(conn: &Connection, semester_id: i64) -> Result<()> {
    let now = chrono::Local::now().naive_local();

    conn.execute(
        "UPDATE active
         SET semester_id = ?1, activated_at = ?2
         WHERE id = 1",
        params![semester_id, now],
    )?;

    Ok(())
}

/// Set the active course (and its semester)
pub fn set_active_course(conn: &Connection, course_id: i64, semester_id: i64) -> Result<()> {
    let now = chrono::Local::now().naive_local();

    conn.execute(
        "UPDATE active
         SET semester_id = ?1, course_id = ?2, lecture_id = NULL, activated_at = ?3
         WHERE id = 1",
        params![semester_id, course_id, now],
    )?;

    Ok(())
}

/// Clear the active course (but keep semester active)
pub fn clear_active_course(conn: &Connection) -> Result<()> {
    conn.execute(
        "UPDATE active
         SET course_id = NULL, lecture_id = NULL
         WHERE id = 1",
        [],
    )?;

    Ok(())
}

/// Clear everything
pub fn clear_all(conn: &Connection) -> Result<()> {
    conn.execute(
        "UPDATE active
         SET semester_id = NULL, course_id = NULL, lecture_id = NULL, activated_at = NULL
         WHERE id = 1",
        [],
    )?;

    Ok(())
}

/// Set the active lecture for the current active course
pub fn set_active_lecture(conn: &Connection, lecture_id: i64) -> Result<()> {
    conn.execute(
        "UPDATE active
         SET lecture_id = ?1
         WHERE id = 1",
        [lecture_id],
    )?;

    Ok(())
}
