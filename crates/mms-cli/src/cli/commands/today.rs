use mms_core::db::connection;
use mms_core::db::models::event::EventType;
use mms_core::db::models::schedule::{CourseSchedule, ScheduleType};
use mms_core::db::queries;
use anyhow::Result;
use chrono::{Datelike, Local, NaiveTime};
use colored::Colorize;

#[derive(Debug)]
enum TodayEvent {
    Regular {
        course_name: String,
        schedule: CourseSchedule,
        start_time: NaiveTime,
    },
    Cancelled {
        course_name: String,
        schedule: CourseSchedule,
        start_time: NaiveTime,
        reason: Option<String>,
    },
    Modified {
        course_name: String,
        schedule: CourseSchedule,
        start_time: NaiveTime,
        end_time: NaiveTime,
        room: Option<String>,
        location: Option<String>,
    },
    Special {
        course_name: String,
        schedule_type: ScheduleType,
        start_time: NaiveTime,
        end_time: NaiveTime,
        room: Option<String>,
        location: Option<String>,
        description: Option<String>,
    },
}

impl TodayEvent {
    fn start_time(&self) -> NaiveTime {
        match self {
            TodayEvent::Regular { start_time, .. } => *start_time,
            TodayEvent::Cancelled { start_time, .. } => *start_time,
            TodayEvent::Modified { start_time, .. } => *start_time,
            TodayEvent::Special { start_time, .. } => *start_time,
        }
    }
}

pub fn handle() -> Result<()> {
    let conn = connection::get()?;
    let today = Local::now().date_naive();
    let today_weekday = today.weekday().num_days_from_monday();

    println!("{}", format!("Today's Schedule - {}", today.format("%A, %d.%m.%Y")).bold().underline());
    println!();

    // Get all courses
    let courses = queries::course::list(&conn)?;

    if courses.is_empty() {
        println!("{}", "No courses found.".yellow());
        return Ok(());
    }

    let mut all_events = Vec::new();

    for course in courses {
        let course_id = course.id.unwrap();
        let course_name = course.name.clone();

        // Get recurring schedules for this course
        let schedules = queries::schedule::list_by_course(&conn, course_id)?;

        // Get all events for this course
        let events = queries::event::list_by_course(&conn, course_id)?;

        // Find schedules that occur today
        for schedule in schedules {
            // Check if schedule is active on this date and matches the day of week
            if schedule.day_of_week == today_weekday
                && today >= schedule.start_date
                && today <= schedule.end_date {

                // Check if this occurrence is cancelled
                let cancel_event = events.iter().find(|e| {
                    e.course_schedule_id == schedule.id
                        && e.date == today
                        && e.event_type == EventType::Cancelled
                });

                // Check if this occurrence is overridden
                let override_event = events.iter().find(|e| {
                    e.course_schedule_id == schedule.id
                        && e.date == today
                        && e.event_type == EventType::Override
                });

                if let Some(cancel) = cancel_event {
                    all_events.push(TodayEvent::Cancelled {
                        course_name: course_name.clone(),
                        schedule: schedule.clone(),
                        start_time: schedule.start_time,
                        reason: cancel.description.clone(),
                    });
                } else if let Some(override_ev) = override_event {
                    let start = override_ev.start_time.unwrap_or(schedule.start_time);
                    let end = override_ev.end_time.unwrap_or(schedule.end_time);
                    let room = override_ev.room.clone().or(schedule.room.clone());
                    let location = override_ev.location.clone().or(schedule.location.clone());

                    all_events.push(TodayEvent::Modified {
                        course_name: course_name.clone(),
                        schedule: schedule.clone(),
                        start_time: start,
                        end_time: end,
                        room,
                        location,
                    });
                } else {
                    all_events.push(TodayEvent::Regular {
                        course_name: course_name.clone(),
                        schedule: schedule.clone(),
                        start_time: schedule.start_time,
                    });
                }
            }
        }

        // Find one-time events for today
        for event in events.iter().filter(|e| e.date == today && e.event_type == EventType::OneTime) {
            if let (Some(start), Some(end)) = (event.start_time, event.end_time) {
                all_events.push(TodayEvent::Special {
                    course_name: course_name.clone(),
                    schedule_type: event.schedule_type,
                    start_time: start,
                    end_time: end,
                    room: event.room.clone(),
                    location: event.location.clone(),
                    description: event.description.clone(),
                });
            }
        }
    }

    if all_events.is_empty() {
        println!("{}", "No events scheduled for today.".yellow());
        println!("Enjoy your free day! 🎉");
        return Ok(());
    }

    // Sort all events by start time
    all_events.sort_by_key(|e| e.start_time());

    // Get current time to determine which events have passed
    let now = Local::now().time();

    // Display all events in order
    for event in all_events {
        // Determine if event has ended (comparing end time with current time)
        let end_time = match &event {
            TodayEvent::Regular { schedule, .. } => schedule.end_time,
            TodayEvent::Cancelled { schedule, .. } => schedule.end_time,
            TodayEvent::Modified { end_time, .. } => *end_time,
            TodayEvent::Special { end_time, .. } => *end_time,
        };
        let has_passed = end_time < now;

        match event {
            TodayEvent::Regular { course_name, schedule, start_time } => {
                if has_passed {
                    println!(
                        "{} {} {} - {} {}",
                        "→".dimmed(),
                        format!("{} - {}", start_time.format("%H:%M"), schedule.end_time.format("%H:%M")).dimmed(),
                        course_name.dimmed(),
                        format_schedule_type(schedule.schedule_type).dimmed(),
                        "(done)".dimmed()
                    );
                } else {
                    println!(
                        "{} {} {} - {}",
                        "→".green(),
                        format!("{} - {}", start_time.format("%H:%M"), schedule.end_time.format("%H:%M")).bold(),
                        course_name.cyan(),
                        format_schedule_type(schedule.schedule_type)
                    );
                    if let Some(room) = &schedule.room {
                        println!("  Room: {}", room);
                    }
                    if let Some(location) = &schedule.location {
                        println!("  Location: {}", location);
                    }
                }
            }
            TodayEvent::Cancelled { course_name, schedule, start_time, reason } => {
                if has_passed {
                    println!(
                        "{} {} {} - {} {}",
                        "✗".dimmed(),
                        format!("{} - {}", start_time.format("%H:%M"), schedule.end_time.format("%H:%M")).dimmed(),
                        course_name.dimmed(),
                        format_schedule_type(schedule.schedule_type).dimmed(),
                        "[CANCELLED]".dimmed()
                    );
                } else {
                    println!(
                        "{} {} {} - {} {}",
                        "✗".red(),
                        format!("{} - {}", start_time.format("%H:%M"), schedule.end_time.format("%H:%M")).bold(),
                        course_name.cyan(),
                        format_schedule_type(schedule.schedule_type),
                        "[CANCELLED]".red().bold()
                    );
                    if let Some(r) = reason {
                        println!("  Reason: {}", r.dimmed());
                    }
                }
            }
            TodayEvent::Modified { course_name, schedule, start_time, end_time, room, location } => {
                if has_passed {
                    println!(
                        "{} {} {} - {} {}",
                        "⚠".dimmed(),
                        format!("{} - {}", start_time.format("%H:%M"), end_time.format("%H:%M")).dimmed(),
                        course_name.dimmed(),
                        format_schedule_type(schedule.schedule_type).dimmed(),
                        "[MODIFIED]".dimmed()
                    );
                } else {
                    println!(
                        "{} {} {} - {} {}",
                        "⚠".yellow(),
                        format!("{} - {}", start_time.format("%H:%M"), end_time.format("%H:%M")).bold(),
                        course_name.cyan(),
                        format_schedule_type(schedule.schedule_type),
                        "[MODIFIED]".yellow().bold()
                    );
                    if let Some(r) = room {
                        println!("  Room: {}", r);
                    }
                    if let Some(l) = location {
                        println!("  Location: {}", l);
                    }
                }
            }
            TodayEvent::Special { course_name, schedule_type, start_time, end_time, room, location, description } => {
                if has_passed {
                    println!(
                        "{} {} {} - {} {} {}",
                        "★".dimmed(),
                        format!("{} - {}", start_time.format("%H:%M"), end_time.format("%H:%M")).dimmed(),
                        course_name.dimmed(),
                        format_schedule_type(schedule_type).dimmed(),
                        "[SPECIAL]".dimmed(),
                        "(done)".dimmed()
                    );
                } else {
                    println!(
                        "{} {} {} - {} {}",
                        "★".cyan(),
                        format!("{} - {}", start_time.format("%H:%M"), end_time.format("%H:%M")).bold(),
                        course_name.cyan(),
                        format_schedule_type(schedule_type),
                        "[SPECIAL]".cyan().bold()
                    );
                    if let Some(desc) = description {
                        println!("  Note: {}", desc);
                    }
                    if let Some(r) = room {
                        println!("  Room: {}", r);
                    }
                    if let Some(l) = location {
                        println!("  Location: {}", l);
                    }
                }
            }
        }
    }

    Ok(())
}

fn format_schedule_type(schedule_type: ScheduleType) -> String {
    match schedule_type {
        ScheduleType::Lecture => "Lecture".to_string(),
        ScheduleType::Tutorium => "Tutorium".to_string(),
        ScheduleType::Exercise => "Exercise".to_string(),
    }
}
