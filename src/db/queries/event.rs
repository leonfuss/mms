use rusqlite::{Connection, params};
use chrono::NaiveDate;
use crate::db::models::event::{CourseEvent, EventType};
use crate::db::models::schedule::ScheduleType;
use crate::error::{MmsError, Result};

/// Insert a new course event
pub fn insert(conn: &Connection, event: &CourseEvent) -> Result<i64> {
    conn.execute(
        "INSERT INTO course_events (course_id, course_schedule_id, schedule_type, event_type, date, start_time, end_time, room, location, description)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            event.course_id,
            event.course_schedule_id,
            event.schedule_type.to_str(),
            event.event_type.to_str(),
            event.date,
            event.start_time,
            event.end_time,
            event.room,
            event.location,
            event.description,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Get an event by ID
pub fn get_by_id(conn: &Connection, id: i64) -> Result<CourseEvent> {
    let mut stmt = conn.prepare(
        "SELECT id, course_id, course_schedule_id, schedule_type, event_type, date, start_time, end_time, room, location, description
         FROM course_events
         WHERE id = ?1"
    )?;

    let event = stmt.query_row(params![id], |row| {
        let schedule_type_str: String = row.get(3)?;
        let event_type_str: String = row.get(4)?;
        Ok(CourseEvent {
            id: Some(row.get(0)?),
            course_id: row.get(1)?,
            course_schedule_id: row.get(2)?,
            schedule_type: ScheduleType::from_str(&schedule_type_str)
                .ok_or(rusqlite::Error::InvalidQuery)?,
            event_type: EventType::from_str(&event_type_str)
                .ok_or(rusqlite::Error::InvalidQuery)?,
            date: row.get(5)?,
            start_time: row.get(6)?,
            end_time: row.get(7)?,
            room: row.get(8)?,
            location: row.get(9)?,
            description: row.get(10)?,
        })
    })?;

    Ok(event)
}

/// List all events for a course
pub fn list_by_course(conn: &Connection, course_id: i64) -> Result<Vec<CourseEvent>> {
    let mut stmt = conn.prepare(
        "SELECT id, course_id, course_schedule_id, schedule_type, event_type, date, start_time, end_time, room, location, description
         FROM course_events
         WHERE course_id = ?1
         ORDER BY date, start_time"
    )?;

    let events = stmt.query_map(params![course_id], |row| {
        let schedule_type_str: String = row.get(3)?;
        let event_type_str: String = row.get(4)?;
        Ok(CourseEvent {
            id: Some(row.get(0)?),
            course_id: row.get(1)?,
            course_schedule_id: row.get(2)?,
            schedule_type: ScheduleType::from_str(&schedule_type_str)
                .ok_or(rusqlite::Error::InvalidQuery)?,
            event_type: EventType::from_str(&event_type_str)
                .ok_or(rusqlite::Error::InvalidQuery)?,
            date: row.get(5)?,
            start_time: row.get(6)?,
            end_time: row.get(7)?,
            room: row.get(8)?,
            location: row.get(9)?,
            description: row.get(10)?,
        })
    })?;

    let mut result = Vec::new();
    for event in events {
        result.push(event?);
    }

    Ok(result)
}

/// Get events for a specific date and course (used for checking cancellations/overrides)
pub fn get_by_course_and_date(conn: &Connection, course_id: i64, date: NaiveDate) -> Result<Vec<CourseEvent>> {
    let mut stmt = conn.prepare(
        "SELECT id, course_id, course_schedule_id, schedule_type, event_type, date, start_time, end_time, room, location, description
         FROM course_events
         WHERE course_id = ?1 AND date = ?2"
    )?;

    let events = stmt.query_map(params![course_id, date], |row| {
        let schedule_type_str: String = row.get(3)?;
        let event_type_str: String = row.get(4)?;
        Ok(CourseEvent {
            id: Some(row.get(0)?),
            course_id: row.get(1)?,
            course_schedule_id: row.get(2)?,
            schedule_type: ScheduleType::from_str(&schedule_type_str)
                .ok_or(rusqlite::Error::InvalidQuery)?,
            event_type: EventType::from_str(&event_type_str)
                .ok_or(rusqlite::Error::InvalidQuery)?,
            date: row.get(5)?,
            start_time: row.get(6)?,
            end_time: row.get(7)?,
            room: row.get(8)?,
            location: row.get(9)?,
            description: row.get(10)?,
        })
    })?;

    let mut result = Vec::new();
    for event in events {
        result.push(event?);
    }

    Ok(result)
}

/// Update an event
pub fn update(conn: &Connection, event: &CourseEvent) -> Result<()> {
    let id = event.id.ok_or(MmsError::Other("Event ID is required for update".to_string()))?;

    conn.execute(
        "UPDATE course_events
         SET course_id = ?1, course_schedule_id = ?2, schedule_type = ?3, event_type = ?4, date = ?5,
             start_time = ?6, end_time = ?7, room = ?8, location = ?9, description = ?10
         WHERE id = ?11",
        params![
            event.course_id,
            event.course_schedule_id,
            event.schedule_type.to_str(),
            event.event_type.to_str(),
            event.date,
            event.start_time,
            event.end_time,
            event.room,
            event.location,
            event.description,
            id,
        ],
    )?;

    Ok(())
}

/// Delete an event
pub fn delete(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM course_events WHERE id = ?1", params![id])?;
    Ok(())
}
