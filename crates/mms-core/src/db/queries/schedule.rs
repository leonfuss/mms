use rusqlite::{Connection, params};
use crate::db::models::schedule::{CourseSchedule, ScheduleType};
use crate::error::{MmsError, Result};

/// Insert a new course schedule
pub fn insert(conn: &Connection, schedule: &CourseSchedule) -> Result<i64> {
    conn.execute(
        "INSERT INTO course_schedules (course_id, schedule_type, day_of_week, start_time, end_time, start_date, end_date, room, location)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            schedule.course_id,
            schedule.schedule_type.to_str(),
            schedule.day_of_week,
            schedule.start_time,
            schedule.end_time,
            schedule.start_date,
            schedule.end_date,
            schedule.room,
            schedule.location,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Get a schedule by ID
pub fn get_by_id(conn: &Connection, id: i64) -> Result<CourseSchedule> {
    let mut stmt = conn.prepare(
        "SELECT id, course_id, schedule_type, day_of_week, start_time, end_time, start_date, end_date, room, location
         FROM course_schedules
         WHERE id = ?1"
    )?;

    let schedule = stmt.query_row(params![id], |row| {
        let schedule_type_str: String = row.get(2)?;
        Ok(CourseSchedule {
            id: Some(row.get(0)?),
            course_id: row.get(1)?,
            schedule_type: ScheduleType::from_str(&schedule_type_str)
                .ok_or(rusqlite::Error::InvalidQuery)?,
            day_of_week: row.get(3)?,
            start_time: row.get(4)?,
            end_time: row.get(5)?,
            start_date: row.get(6)?,
            end_date: row.get(7)?,
            room: row.get(8)?,
            location: row.get(9)?,
        })
    })?;

    Ok(schedule)
}

/// List all schedules for a course
pub fn list_by_course(conn: &Connection, course_id: i64) -> Result<Vec<CourseSchedule>> {
    let mut stmt = conn.prepare(
        "SELECT id, course_id, schedule_type, day_of_week, start_time, end_time, start_date, end_date, room, location
         FROM course_schedules
         WHERE course_id = ?1
         ORDER BY day_of_week, start_time"
    )?;

    let schedules = stmt.query_map(params![course_id], |row| {
        let schedule_type_str: String = row.get(2)?;
        Ok(CourseSchedule {
            id: Some(row.get(0)?),
            course_id: row.get(1)?,
            schedule_type: ScheduleType::from_str(&schedule_type_str)
                .ok_or(rusqlite::Error::InvalidQuery)?,
            day_of_week: row.get(3)?,
            start_time: row.get(4)?,
            end_time: row.get(5)?,
            start_date: row.get(6)?,
            end_date: row.get(7)?,
            room: row.get(8)?,
            location: row.get(9)?,
        })
    })?;

    let mut result = Vec::new();
    for schedule in schedules {
        result.push(schedule?);
    }

    Ok(result)
}

/// Update a schedule
pub fn update(conn: &Connection, schedule: &CourseSchedule) -> Result<()> {
    let id = schedule.id.ok_or(MmsError::Other("Schedule ID is required for update".to_string()))?;

    conn.execute(
        "UPDATE course_schedules
         SET course_id = ?1, schedule_type = ?2, day_of_week = ?3, start_time = ?4, end_time = ?5,
             start_date = ?6, end_date = ?7, room = ?8, location = ?9
         WHERE id = ?10",
        params![
            schedule.course_id,
            schedule.schedule_type.to_str(),
            schedule.day_of_week,
            schedule.start_time,
            schedule.end_time,
            schedule.start_date,
            schedule.end_date,
            schedule.room,
            schedule.location,
            id,
        ],
    )?;

    Ok(())
}

/// Delete a schedule
pub fn delete(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM course_schedules WHERE id = ?1", params![id])?;
    Ok(())
}
