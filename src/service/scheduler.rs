use chrono::{Datelike, Local, NaiveDate, NaiveTime};
use rusqlite::Connection;

use crate::db::models::event::EventType;
use crate::db::models::schedule::CourseSchedule;
use crate::db::queries;
use crate::error::Result;

/// ScheduleEngine determines which course should be active at any given time
/// based on recurring schedules, one-time events, cancellations, and holidays.
pub struct ScheduleEngine;

impl ScheduleEngine {
    /// Determine which course should be active at the current moment
    pub fn determine_active_course_now(conn: &Connection) -> Result<Option<i64>> {
        let now = Local::now();
        let current_date = now.date_naive();
        let current_time = now.time();

        Self::determine_active_course(conn, current_date, current_time)
    }

    /// Determine which course should be active at a specific date and time
    ///
    /// Resolution priority (highest to lowest):
    /// 1. Cancelled events → return None for this course
    /// 2. Override events → use override course
    /// 3. One-time events → return event's course
    /// 4. Recurring schedules (filtered by holidays) → return scheduled course
    /// 5. No match → return None
    pub fn determine_active_course(
        conn: &Connection,
        date: NaiveDate,
        time: NaiveTime,
    ) -> Result<Option<i64>> {
        // Get current semester
        let semester = match queries::semester::get_current(conn)? {
            Some(s) => s,
            None => return Ok(None), // No active semester
        };

        // Get all courses for this semester
        let courses = queries::course::list_by_semester(conn, semester.id.unwrap())?;

        // Check each course in priority order
        for course in courses {
            let course_id = course.id.unwrap();

            // Priority 1: Check if this course is cancelled at this time
            if Self::is_cancelled(conn, course_id, date, time)? {
                continue; // Skip this course
            }

            // Priority 2: Check for override events
            if let Some(override_course_id) = Self::get_override_course(conn, course_id, date, time)? {
                // Override might point to a different course
                return Ok(Some(override_course_id));
            }

            // Priority 3: Check for one-time events
            if Self::is_one_time_event_active(conn, course_id, date, time)? {
                return Ok(Some(course_id));
            }

            // Priority 4: Check recurring schedules (if not a holiday)
            if !Self::is_holiday(conn, semester.id.unwrap(), course_id, date)? {
                if Self::is_recurring_schedule_active(conn, course_id, date, time)? {
                    return Ok(Some(course_id));
                }
            }
        }

        // No active course found
        Ok(None)
    }

    /// Check if a course is cancelled at a specific date and time
    fn is_cancelled(
        conn: &Connection,
        course_id: i64,
        date: NaiveDate,
        time: NaiveTime,
    ) -> Result<bool> {
        let events = queries::event::get_by_course_and_date(conn, course_id, date)?;

        for event in events {
            if event.event_type == EventType::Cancelled {
                // If cancelled event has times, check if current time matches
                if let (Some(start), Some(end)) = (event.start_time, event.end_time) {
                    if Self::is_time_in_range(time, start, end) {
                        return Ok(true);
                    }
                } else {
                    // No time specified = entire day cancelled
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Check for override events and return the override course ID
    fn get_override_course(
        conn: &Connection,
        course_id: i64,
        date: NaiveDate,
        time: NaiveTime,
    ) -> Result<Option<i64>> {
        let events = queries::event::get_by_course_and_date(conn, course_id, date)?;

        for event in events {
            if event.event_type == EventType::Override {
                // If override has times, check if current time matches
                if let (Some(start), Some(end)) = (event.start_time, event.end_time) {
                    if Self::is_time_in_range(time, start, end) {
                        return Ok(Some(event.course_id));
                    }
                } else {
                    // No time specified = entire day override
                    return Ok(Some(event.course_id));
                }
            }
        }

        Ok(None)
    }

    /// Check if a one-time event is active at this date and time
    fn is_one_time_event_active(
        conn: &Connection,
        course_id: i64,
        date: NaiveDate,
        time: NaiveTime,
    ) -> Result<bool> {
        let events = queries::event::get_by_course_and_date(conn, course_id, date)?;

        for event in events {
            if event.event_type == EventType::OneTime {
                if let (Some(start), Some(end)) = (event.start_time, event.end_time) {
                    if Self::is_time_in_range(time, start, end) {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    /// Check if a recurring schedule is active at this date and time
    fn is_recurring_schedule_active(
        conn: &Connection,
        course_id: i64,
        date: NaiveDate,
        time: NaiveTime,
    ) -> Result<bool> {
        let schedules = queries::schedule::list_by_course(conn, course_id)?;

        for schedule in schedules {
            if Self::is_schedule_active_at(&schedule, date, time)? {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Check if a specific schedule is active at the given date and time
    fn is_schedule_active_at(
        schedule: &CourseSchedule,
        date: NaiveDate,
        time: NaiveTime,
    ) -> Result<bool> {
        // Check if date is within schedule range
        if !Self::is_date_in_range(date, schedule.start_date, schedule.end_date) {
            return Ok(false);
        }

        // Check if day of week matches
        // chrono uses 0=Monday, 6=Sunday
        // Our DB also uses 0=Monday, 6=Sunday (based on day_of_week check)
        if date.weekday().num_days_from_monday() != schedule.day_of_week {
            return Ok(false);
        }

        // Check if time is within schedule time range
        if !Self::is_time_in_range(time, schedule.start_time, schedule.end_time) {
            return Ok(false);
        }

        Ok(true)
    }

    /// Check if the date is a holiday (unless the course has an exception)
    fn is_holiday(
        conn: &Connection,
        semester_id: i64,
        course_id: i64,
        date: NaiveDate,
    ) -> Result<bool> {
        // Query holidays for this semester
        let mut stmt = conn.prepare(
            "SELECT id, start_date, end_date FROM holidays
             WHERE semester_id = ?1"
        )?;

        let holidays: Vec<(i64, String, String)> = stmt
            .query_map([semester_id], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        for (holiday_id, start_str, end_str) in holidays {
            let start_date = NaiveDate::parse_from_str(&start_str, "%Y-%m-%d")?;
            let end_date = NaiveDate::parse_from_str(&end_str, "%Y-%m-%d")?;

            if Self::is_date_in_range(date, start_date, end_date) {
                // Check if this course has an exception for this holiday
                let exception_count: i64 = conn.query_row(
                    "SELECT COUNT(*) FROM holiday_exceptions
                     WHERE holiday_id = ?1 AND course_id = ?2",
                    [holiday_id, course_id],
                    |row| row.get(0),
                )?;

                if exception_count == 0 {
                    // No exception = this is a holiday for this course
                    return Ok(true);
                }
                // Has exception = course continues during this holiday
            }
        }

        Ok(false)
    }

    /// Helper: Check if time is within range [start, end)
    fn is_time_in_range(time: NaiveTime, start: NaiveTime, end: NaiveTime) -> bool {
        time >= start && time < end
    }

    /// Helper: Check if date is within range [start, end]
    fn is_date_in_range(date: NaiveDate, start: NaiveDate, end: NaiveDate) -> bool {
        date >= start && date <= end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_time_in_range() {
        let start = NaiveTime::from_hms_opt(14, 0, 0).unwrap();
        let end = NaiveTime::from_hms_opt(16, 0, 0).unwrap();

        // Before range
        let before = NaiveTime::from_hms_opt(13, 59, 59).unwrap();
        assert!(!ScheduleEngine::is_time_in_range(before, start, end));

        // Start of range
        let at_start = NaiveTime::from_hms_opt(14, 0, 0).unwrap();
        assert!(ScheduleEngine::is_time_in_range(at_start, start, end));

        // Middle of range
        let middle = NaiveTime::from_hms_opt(15, 0, 0).unwrap();
        assert!(ScheduleEngine::is_time_in_range(middle, start, end));

        // End of range (exclusive)
        let at_end = NaiveTime::from_hms_opt(16, 0, 0).unwrap();
        assert!(!ScheduleEngine::is_time_in_range(at_end, start, end));

        // After range
        let after = NaiveTime::from_hms_opt(16, 0, 1).unwrap();
        assert!(!ScheduleEngine::is_time_in_range(after, start, end));
    }

    #[test]
    fn test_is_date_in_range() {
        let start = NaiveDate::from_ymd_opt(2024, 10, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        // Before range
        let before = NaiveDate::from_ymd_opt(2024, 9, 30).unwrap();
        assert!(!ScheduleEngine::is_date_in_range(before, start, end));

        // Start of range
        let at_start = NaiveDate::from_ymd_opt(2024, 10, 1).unwrap();
        assert!(ScheduleEngine::is_date_in_range(at_start, start, end));

        // Middle of range
        let middle = NaiveDate::from_ymd_opt(2024, 11, 15).unwrap();
        assert!(ScheduleEngine::is_date_in_range(middle, start, end));

        // End of range (inclusive)
        let at_end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        assert!(ScheduleEngine::is_date_in_range(at_end, start, end));

        // After range
        let after = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        assert!(!ScheduleEngine::is_date_in_range(after, start, end));
    }
}
