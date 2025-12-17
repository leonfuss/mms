use rusqlite::{Connection, params};
use crate::db::models::course::Course;
use crate::error::Result;

/// Insert a new course into the database
pub fn insert(conn: &Connection, course: &Course) -> Result<i64> {
    conn.execute(
        "INSERT INTO courses (semester_id, name, short_name, category, ects, lecturer, learning_platform_url, location, grade, counts_towards_average)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            course.semester_id,
            course.name,
            course.short_name,
            course.category,
            course.ects,
            course.lecturer,
            course.learning_platform_url,
            course.location,
            course.grade,
            course.counts_towards_average,
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

/// Get a course by ID
pub fn get_by_id(conn: &Connection, id: i64) -> Result<Course> {
    let mut stmt = conn.prepare(
        "SELECT id, semester_id, name, short_name, category, ects, lecturer, learning_platform_url, location, grade, counts_towards_average, created_at
         FROM courses
         WHERE id = ?1"
    )?;

    let course = stmt.query_row([id], |row| {
        Ok(Course {
            id: Some(row.get(0)?),
            semester_id: row.get(1)?,
            name: row.get(2)?,
            short_name: row.get(3)?,
            category: row.get(4)?,
            ects: row.get(5)?,
            lecturer: row.get(6)?,
            learning_platform_url: row.get(7)?,
            location: row.get(8)?,
            grade: row.get(9)?,
            counts_towards_average: row.get(10)?,
            created_at: row.get(11)?,
        })
    })?;

    Ok(course)
}

/// List all courses
pub fn list(conn: &Connection) -> Result<Vec<Course>> {
    let mut stmt = conn.prepare(
        "SELECT id, semester_id, name, short_name, category, ects, lecturer, learning_platform_url, location, grade, counts_towards_average, created_at
         FROM courses
         ORDER BY semester_id DESC, name ASC"
    )?;

    let courses = stmt
        .query_map([], |row| {
            Ok(Course {
                id: Some(row.get(0)?),
                semester_id: row.get(1)?,
                name: row.get(2)?,
                short_name: row.get(3)?,
                category: row.get(4)?,
                ects: row.get(5)?,
                lecturer: row.get(6)?,
                learning_platform_url: row.get(7)?,
                location: row.get(8)?,
                grade: row.get(9)?,
                counts_towards_average: row.get(10)?,
                created_at: row.get(11)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(courses)
}

/// List courses for a specific semester
pub fn list_by_semester(conn: &Connection, semester_id: i64) -> Result<Vec<Course>> {
    let mut stmt = conn.prepare(
        "SELECT id, semester_id, name, short_name, category, ects, lecturer, learning_platform_url, location, grade, counts_towards_average, created_at
         FROM courses
         WHERE semester_id = ?1
         ORDER BY name ASC"
    )?;

    let courses = stmt
        .query_map([semester_id], |row| {
            Ok(Course {
                id: Some(row.get(0)?),
                semester_id: row.get(1)?,
                name: row.get(2)?,
                short_name: row.get(3)?,
                category: row.get(4)?,
                ects: row.get(5)?,
                lecturer: row.get(6)?,
                learning_platform_url: row.get(7)?,
                location: row.get(8)?,
                grade: row.get(9)?,
                counts_towards_average: row.get(10)?,
                created_at: row.get(11)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(courses)
}

/// Update a course
pub fn update(conn: &Connection, course: &Course) -> Result<()> {
    let id = course.id.ok_or_else(|| {
        crate::error::MmsError::Other("Cannot update course without id".to_string())
    })?;

    conn.execute(
        "UPDATE courses
         SET semester_id = ?1, name = ?2, short_name = ?3, category = ?4, ects = ?5, lecturer = ?6, learning_platform_url = ?7, location = ?8, grade = ?9, counts_towards_average = ?10
         WHERE id = ?11",
        params![
            course.semester_id,
            course.name,
            course.short_name,
            course.category,
            course.ects,
            course.lecturer,
            course.learning_platform_url,
            course.location,
            course.grade,
            course.counts_towards_average,
            id,
        ],
    )?;

    Ok(())
}

/// Delete a course
pub fn delete(conn: &Connection, id: i64) -> Result<()> {
    let rows_affected = conn.execute("DELETE FROM courses WHERE id = ?1", [id])?;

    if rows_affected == 0 {
        return Err(crate::error::MmsError::CourseNotFound(id));
    }

    Ok(())
}
