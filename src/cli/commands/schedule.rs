use crate::cli::args::ScheduleAction;
use crate::cli::course_resolver::CourseResolver;
use crate::cli::prompt_helpers::*;
use crate::db::connection;
use crate::db::models::event::{CourseEvent, EventType};
use crate::db::models::schedule::{CourseSchedule, ScheduleType};
use crate::db::queries;
use crate::error::{MmsError, Result};
use chrono::NaiveTime;
use colored::Colorize;
use dialoguer::{Confirm, Input};

pub fn handle(action: ScheduleAction) -> Result<()> {
    match action {
        ScheduleAction::Add {
            course,
            day,
            date,
            start,
            end,
            start_date,
            end_date,
            recurring,
            schedule_type,
            room,
            location,
            description,
        } => handle_add(recurring, course, day, date, start, end, start_date, end_date, schedule_type, room, location, description),
        ScheduleAction::Cancel {
            schedule_id,
            date,
            reason,
        } => handle_cancel(schedule_id, date, reason),
        ScheduleAction::Override {
            schedule_id,
            date,
            room,
            time,
        } => handle_override(schedule_id, date, room, time),
        ScheduleAction::List { course } => handle_list(course),
        ScheduleAction::Edit { id, event } => {
            if event {
                handle_edit_event(id)
            } else {
                handle_edit_schedule(id)
            }
        }
        ScheduleAction::Delete { id, event } => {
            if event {
                handle_delete_event(id)
            } else {
                handle_delete_schedule(id)
            }
        }
    }
}

fn parse_day_of_week(day: &str) -> Result<u32> {
    match day.to_lowercase().as_str() {
        "monday" => Ok(0),
        "tuesday" => Ok(1),
        "wednesday" => Ok(2),
        "thursday" => Ok(3),
        "friday" => Ok(4),
        "saturday" => Ok(5),
        "sunday" => Ok(6),
        _ => Err(MmsError::Parse(format!("Invalid day of week: {}", day))),
    }
}

fn day_of_week_to_string(day: u32) -> &'static str {
    match day {
        0 => "Monday",
        1 => "Tuesday",
        2 => "Wednesday",
        3 => "Thursday",
        4 => "Friday",
        5 => "Saturday",
        6 => "Sunday",
        _ => "Unknown",
    }
}

/// Parse date in dd.mm.yyyy format (European format)
/// Future: Can be extended to use locale-aware parsing with icu4x
fn parse_date(date_str: &str) -> Result<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(date_str, "%d.%m.%Y")
        .map_err(|_| MmsError::Parse(format!(
            "Invalid date: {}. Use dd.mm.yyyy format (e.g., 24.12.2024)",
            date_str
        )))
}

fn handle_add(
    mut recurring: bool,
    course_input: Option<String>,
    day: Option<String>,
    date: Option<String>,
    start: Option<String>,
    end: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    schedule_type: Option<String>,
    room: Option<String>,
    location: Option<String>,
    description: Option<String>,
) -> Result<()> {
    let conn = connection::get()?;

    // Resolve course
    let (course_id, course) = CourseResolver::resolve(course_input)?;

    // If no arguments provided at all, ask if recurring
    if !recurring && day.is_none() && date.is_none() {
        recurring = Confirm::new()
            .with_prompt("Is this a recurring weekly schedule?")
            .default(true)
            .interact()?;
    }

    if recurring {
        handle_add_recurring_internal(conn, course_id, course, day, start, end, start_date, end_date, schedule_type, room, location)
    } else {
        handle_add_event_internal(conn, course_id, course, date, start, end, schedule_type, room, location, description)
    }
}

fn handle_add_recurring_internal(
    conn: rusqlite::Connection,
    course_id: i64,
    course: crate::db::models::course::Course,
    day: Option<String>,
    start: Option<String>,
    end: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    schedule_type: Option<String>,
    room: Option<String>,
    location: Option<String>,
) -> Result<()> {
    println!("{}", format!("Add Recurring Schedule for: {}", course.name).bold().underline());
    println!();

    // Get all inputs (prompt if not provided)
    let day = get_or_prompt(day, prompt_day_of_week)?;
    let start = get_or_prompt(start, || prompt_time("Start time"))?;
    let end = get_or_prompt(end, || prompt_time("End time"))?;
    let start_date = get_or_prompt(start_date, || prompt_date("Start date"))?;
    let end_date = get_or_prompt(end_date, || prompt_date("End date"))?;
    let schedule_type = get_or_prompt(schedule_type, prompt_schedule_type)?;
    let room = get_or_prompt_optional(room, prompt_room)?;
    let location = get_or_prompt_optional(location, prompt_location_override)?;

    // Parse inputs
    let day_of_week = parse_day_of_week(&day)?;
    let start_time = NaiveTime::parse_from_str(&start, "%H:%M")
        .map_err(|_| MmsError::Parse(format!("Invalid start time: {}. Use HH:MM format", start)))?;
    let end_time = NaiveTime::parse_from_str(&end, "%H:%M")
        .map_err(|_| MmsError::Parse(format!("Invalid end time: {}. Use HH:MM format", end)))?;
    let start_date_parsed = parse_date(&start_date)?;
    let end_date_parsed = parse_date(&end_date)?;
    let sched_type = ScheduleType::from_str(&schedule_type)
        .ok_or(MmsError::InvalidScheduleType(schedule_type.clone()))?;

    // Create schedule
    let mut schedule = CourseSchedule::new(
        course_id,
        sched_type,
        day_of_week,
        start_time,
        end_time,
        start_date_parsed,
        end_date_parsed,
    );

    if let Some(r) = room {
        schedule = schedule.with_room(r);
    }
    if let Some(l) = location {
        schedule = schedule.with_location(l);
    }

    let id = queries::schedule::insert(&conn, &schedule)?;

    println!("{}", "✓ Recurring schedule created!".green());
    println!("  ID:       {}", id);
    println!("  Course:   {}", course.name.bold());
    println!("  Type:     {}", schedule_type);
    println!("  Day:      {}", day_of_week_to_string(day_of_week));
    println!("  Time:     {} - {}", start_time.format("%H:%M"), end_time.format("%H:%M"));
    println!("  Period:   {} to {}", start_date_parsed.format("%d.%m.%Y"), end_date_parsed.format("%d.%m.%Y"));
    if let Some(r) = &schedule.room {
        println!("  Room:     {}", r);
    }
    if let Some(l) = &schedule.location {
        println!("  Location: {}", l);
    }

    Ok(())
}

fn handle_add_event_internal(
    conn: rusqlite::Connection,
    course_id: i64,
    course: crate::db::models::course::Course,
    date: Option<String>,
    start: Option<String>,
    end: Option<String>,
    schedule_type: Option<String>,
    room: Option<String>,
    location: Option<String>,
    description: Option<String>,
) -> Result<()> {
    println!("{}", format!("Add One-Time Event for: {}", course.name).bold().underline());
    println!();

    // Get all inputs (prompt if not provided)
    let date = get_or_prompt(date, || prompt_date("Date"))?;
    let start = get_or_prompt(start, || prompt_time("Start time"))?;
    let end = get_or_prompt(end, || prompt_time("End time"))?;
    let schedule_type = get_or_prompt(schedule_type, prompt_schedule_type)?;
    let room = get_or_prompt_optional(room, prompt_room)?;
    let location = get_or_prompt_optional(location, prompt_location_override)?;
    let description = get_or_prompt_optional(description, prompt_description)?;

    // Parse inputs
    let date_parsed = parse_date(&date)?;
    let start_time = NaiveTime::parse_from_str(&start, "%H:%M")
        .map_err(|_| MmsError::Parse(format!("Invalid start time: {}. Use HH:MM format", start)))?;
    let end_time = NaiveTime::parse_from_str(&end, "%H:%M")
        .map_err(|_| MmsError::Parse(format!("Invalid end time: {}. Use HH:MM format", end)))?;
    let sched_type = ScheduleType::from_str(&schedule_type)
        .ok_or(MmsError::InvalidScheduleType(schedule_type.clone()))?;

    // Create one-time event
    let mut event = CourseEvent::new_one_time(
        course_id,
        sched_type,
        date_parsed,
        start_time,
        end_time,
    );

    if let Some(r) = room {
        event = event.with_room(r);
    }
    if let Some(l) = location {
        event = event.with_location(l);
    }
    if let Some(d) = description {
        event = event.with_description(d);
    }

    let id = queries::event::insert(&conn, &event)?;

    println!("{}", "✓ One-time event created!".green());
    println!("  ID:       {}", id);
    println!("  Course:   {}", course.name.bold());
    println!("  Type:     {}", schedule_type);
    println!("  Date:     {}", date_parsed.format("%d.%m.%Y"));
    println!("  Time:     {} - {}", start_time.format("%H:%M"), end_time.format("%H:%M"));
    if let Some(r) = &event.room {
        println!("  Room:     {}", r);
    }
    if let Some(l) = &event.location {
        println!("  Location: {}", l);
    }
    if let Some(d) = &event.description {
        println!("  Note:     {}", d);
    }

    Ok(())
}

fn handle_cancel(schedule_id: i64, date: String, reason: Option<String>) -> Result<()> {
    let conn = connection::get()?;

    // Verify schedule exists
    let schedule = queries::schedule::get_by_id(&conn, schedule_id)?;
    let course = queries::course::get_by_id(&conn, schedule.course_id)?;

    // Parse date
    let date_parsed = parse_date(&date)?;

    // Check if date is within schedule range
    if date_parsed < schedule.start_date || date_parsed > schedule.end_date {
        return Err(MmsError::Other(format!(
            "Date {} is outside schedule range ({} to {})",
            date_parsed, schedule.start_date, schedule.end_date
        )));
    }

    // Create cancellation event
    let mut event = CourseEvent::new_cancelled(schedule_id, schedule.course_id, date_parsed);
    if let Some(r) = reason {
        event = event.with_description(r);
    }

    let id = queries::event::insert(&conn, &event)?;

    println!("{}", "✓ Schedule cancelled for this date!".green());
    println!("  ID:       {}", id);
    println!("  Course:   {}", course.name.bold());
    println!("  Date:     {}", date_parsed.format("%d.%m.%Y"));
    println!("  Day:      {}", day_of_week_to_string(schedule.day_of_week));
    println!("  Time:     {} - {}", schedule.start_time.format("%H:%M"), schedule.end_time.format("%H:%M"));
    if let Some(r) = &event.description {
        println!("  Reason:   {}", r);
    }

    Ok(())
}

fn handle_override(
    schedule_id: i64,
    date: String,
    room: Option<String>,
    time: Option<String>,
) -> Result<()> {
    let conn = connection::get()?;

    // Verify schedule exists
    let schedule = queries::schedule::get_by_id(&conn, schedule_id)?;
    let course = queries::course::get_by_id(&conn, schedule.course_id)?;

    // Parse date
    let date_parsed = parse_date(&date)?;

    // Check if date is within schedule range
    if date_parsed < schedule.start_date || date_parsed > schedule.end_date {
        return Err(MmsError::Other(format!(
            "Date {} is outside schedule range ({} to {})",
            date_parsed, schedule.start_date, schedule.end_date
        )));
    }

    // Create override event
    let mut event = CourseEvent::new_override(
        schedule_id,
        schedule.course_id,
        schedule.schedule_type,
        date_parsed,
    );

    // Parse time if provided
    if let Some(t) = time {
        let parts: Vec<&str> = t.split('-').collect();
        if parts.len() != 2 {
            return Err(MmsError::Parse(format!("Invalid time range: {}. Use HH:MM-HH:MM format", t)));
        }
        let start_time = NaiveTime::parse_from_str(parts[0], "%H:%M")
            .map_err(|_| MmsError::Parse(format!("Invalid start time: {}", parts[0])))?;
        let end_time = NaiveTime::parse_from_str(parts[1], "%H:%M")
            .map_err(|_| MmsError::Parse(format!("Invalid end time: {}", parts[1])))?;
        event = event.with_time(start_time, end_time);
    }

    if let Some(r) = room {
        event = event.with_room(r);
    }

    let id = queries::event::insert(&conn, &event)?;

    println!("{}", "✓ Schedule overridden for this date!".green());
    println!("  ID:       {}", id);
    println!("  Course:   {}", course.name.bold());
    println!("  Date:     {}", date_parsed.format("%d.%m.%Y"));
    println!("  Day:      {}", day_of_week_to_string(schedule.day_of_week));
    if let Some(start) = event.start_time {
        if let Some(end) = event.end_time {
            println!("  New Time: {} - {}", start.format("%H:%M"), end.format("%H:%M"));
        }
    } else {
        println!("  Time:     {} - {} (unchanged)", schedule.start_time.format("%H:%M"), schedule.end_time.format("%H:%M"));
    }
    if let Some(r) = &event.room {
        println!("  New Room: {}", r);
    }

    Ok(())
}

fn handle_list(course_input: Option<String>) -> Result<()> {
    let conn = connection::get()?;

    // Resolve course
    let (course_id, course) = CourseResolver::resolve(course_input)?;

    // Get schedules and events
    let schedules = queries::schedule::list_by_course(&conn, course_id)?;
    let events = queries::event::list_by_course(&conn, course_id)?;

    println!("{}", format!("Schedule for: {}", course.name).bold().underline());
    println!();

    if schedules.is_empty() && events.is_empty() {
        println!("{}", "No schedules or events found.".yellow());
        println!("Use 'mms schedule add' to create one.");
        return Ok(());
    }

    // Display recurring schedules
    if !schedules.is_empty() {
        println!("{}", "Recurring Schedules:".bold());
        for schedule in schedules {
            let type_str = match schedule.schedule_type {
                ScheduleType::Lecture => "Lecture".cyan(),
                ScheduleType::Tutorium => "Tutorium".yellow(),
                ScheduleType::Exercise => "Exercise".green(),
            };
            println!(
                "  [{}] {} - {} {} - {}",
                schedule.id.unwrap(),
                day_of_week_to_string(schedule.day_of_week).bold(),
                schedule.start_time.format("%H:%M"),
                schedule.end_time.format("%H:%M"),
                type_str,
            );
            println!(
                "      Period: {} to {}",
                schedule.start_date.format("%d.%m.%Y"),
                schedule.end_date.format("%d.%m.%Y")
            );
            if let Some(room) = &schedule.room {
                println!("      Room: {}", room);
            }
            if let Some(location) = &schedule.location {
                println!("      Location: {}", location);
            }
        }
        println!();
    }

    // Display events
    if !events.is_empty() {
        println!("{}", "Events:".bold());
        for event in events {
            let type_str = match event.event_type {
                EventType::OneTime => "One-time".cyan(),
                EventType::Makeup => "Makeup".yellow(),
                EventType::Special => "Special".green(),
                EventType::Override => "Override".magenta(),
                EventType::Cancelled => "Cancelled".red(),
            };

            print!(
                "  [{}] {} - {}",
                event.id.unwrap(),
                event.date.format("%d.%m.%Y"),
                type_str,
            );

            if event.event_type != EventType::Cancelled {
                if let (Some(start), Some(end)) = (event.start_time, event.end_time) {
                    print!(" {} - {}", start.format("%H:%M"), end.format("%H:%M"));
                }
            }

            println!();

            if let Some(room) = &event.room {
                println!("      Room: {}", room);
            }
            if let Some(location) = &event.location {
                println!("      Location: {}", location);
            }
            if let Some(desc) = &event.description {
                println!("      Note: {}", desc);
            }
        }
    }

    Ok(())
}

fn handle_edit_schedule(schedule_id: i64) -> Result<()> {
    let conn = connection::get()?;
    let mut schedule = queries::schedule::get_by_id(&conn, schedule_id)?;
    let course = queries::course::get_by_id(&conn, schedule.course_id)?;

    println!("{}", "Edit Schedule".bold().underline());
    println!();
    println!("Course: {}", course.name.bold());
    println!("Leave blank to keep current value.");
    println!();

    // Edit day of week
    let day_input: String = Input::new()
        .with_prompt(format!(
            "Day of week [{}] (monday-sunday)",
            day_of_week_to_string(schedule.day_of_week)
        ))
        .allow_empty(true)
        .interact_text()?;
    if !day_input.is_empty() {
        schedule.day_of_week = parse_day_of_week(&day_input)?;
    }

    // Edit start time
    let start_input: String = Input::new()
        .with_prompt(format!("Start time [{}] (HH:MM)", schedule.start_time.format("%H:%M")))
        .allow_empty(true)
        .interact_text()?;
    if !start_input.is_empty() {
        schedule.start_time = NaiveTime::parse_from_str(&start_input, "%H:%M")
            .map_err(|_| MmsError::Parse(format!("Invalid time: {}", start_input)))?;
    }

    // Edit end time
    let end_input: String = Input::new()
        .with_prompt(format!("End time [{}] (HH:MM)", schedule.end_time.format("%H:%M")))
        .allow_empty(true)
        .interact_text()?;
    if !end_input.is_empty() {
        schedule.end_time = NaiveTime::parse_from_str(&end_input, "%H:%M")
            .map_err(|_| MmsError::Parse(format!("Invalid time: {}", end_input)))?;
    }

    // Edit start date
    let start_date_input: String = Input::new()
        .with_prompt(format!("Start date [{}] (dd.mm.yyyy)", schedule.start_date.format("%d.%m.%Y")))
        .allow_empty(true)
        .interact_text()?;
    if !start_date_input.is_empty() {
        schedule.start_date = parse_date(&start_date_input)?;
    }

    // Edit end date
    let end_date_input: String = Input::new()
        .with_prompt(format!("End date [{}] (dd.mm.yyyy)", schedule.end_date.format("%d.%m.%Y")))
        .allow_empty(true)
        .interact_text()?;
    if !end_date_input.is_empty() {
        schedule.end_date = parse_date(&end_date_input)?;
    }

    // Edit room
    let room_input: String = Input::new()
        .with_prompt(format!(
            "Room [{}] (type 'none' to remove)",
            schedule.room.as_deref().unwrap_or("none")
        ))
        .allow_empty(true)
        .interact_text()?;
    if !room_input.is_empty() {
        if room_input.to_lowercase() == "none" {
            schedule.room = None;
        } else {
            schedule.room = Some(room_input);
        }
    }

    // Edit location
    let location_input: String = Input::new()
        .with_prompt(format!(
            "Location [{}] (type 'none' to remove)",
            schedule.location.as_deref().unwrap_or("none")
        ))
        .allow_empty(true)
        .interact_text()?;
    if !location_input.is_empty() {
        if location_input.to_lowercase() == "none" {
            schedule.location = None;
        } else {
            schedule.location = Some(location_input);
        }
    }

    queries::schedule::update(&conn, &schedule)?;

    println!();
    println!("{}", "✓ Schedule updated successfully!".green());

    Ok(())
}

fn handle_delete_schedule(schedule_id: i64) -> Result<()> {
    let conn = connection::get()?;
    let schedule = queries::schedule::get_by_id(&conn, schedule_id)?;
    let course = queries::course::get_by_id(&conn, schedule.course_id)?;

    println!("{}", "Delete Schedule".bold().underline());
    println!();
    println!("Course: {}", course.name.bold());
    println!("Day:    {}", day_of_week_to_string(schedule.day_of_week));
    println!("Time:   {} - {}", schedule.start_time.format("%H:%M"), schedule.end_time.format("%H:%M"));
    println!("Period: {} to {}", schedule.start_date.format("%d.%m.%Y"), schedule.end_date.format("%d.%m.%Y"));
    println!();

    let confirm = dialoguer::Confirm::new()
        .with_prompt("Are you sure you want to delete this schedule?")
        .default(false)
        .interact()?;

    if !confirm {
        println!("{}", "Cancelled.".yellow());
        return Ok(());
    }

    queries::schedule::delete(&conn, schedule_id)?;

    println!("{}", "✓ Schedule deleted!".green());

    Ok(())
}

fn handle_edit_event(event_id: i64) -> Result<()> {
    let conn = connection::get()?;
    let mut event = queries::event::get_by_id(&conn, event_id)?;
    let course = queries::course::get_by_id(&conn, event.course_id)?;

    println!("{}", "Edit Event".bold().underline());
    println!();
    println!("Course: {}", course.name.bold());
    println!("Type:   {}", event.event_type.to_str());
    println!("Leave blank to keep current value.");
    println!();

    // Edit date
    let date_input: String = Input::new()
        .with_prompt(format!("Date [{}] (dd.mm.yyyy)", event.date.format("%d.%m.%Y")))
        .allow_empty(true)
        .interact_text()?;
    if !date_input.is_empty() {
        event.date = parse_date(&date_input)?;
    }

    // Only edit times for non-cancelled events
    if event.event_type != EventType::Cancelled {
        // Edit start time
        let start_input: String = Input::new()
            .with_prompt(format!(
                "Start time [{}] (HH:MM, type 'none' to remove)",
                event.start_time.map(|t| t.format("%H:%M").to_string()).unwrap_or_else(|| "none".to_string())
            ))
            .allow_empty(true)
            .interact_text()?;
        if !start_input.is_empty() {
            if start_input.to_lowercase() == "none" {
                event.start_time = None;
            } else {
                event.start_time = Some(NaiveTime::parse_from_str(&start_input, "%H:%M")
                    .map_err(|_| MmsError::Parse(format!("Invalid time: {}", start_input)))?);
            }
        }

        // Edit end time
        let end_input: String = Input::new()
            .with_prompt(format!(
                "End time [{}] (HH:MM, type 'none' to remove)",
                event.end_time.map(|t| t.format("%H:%M").to_string()).unwrap_or_else(|| "none".to_string())
            ))
            .allow_empty(true)
            .interact_text()?;
        if !end_input.is_empty() {
            if end_input.to_lowercase() == "none" {
                event.end_time = None;
            } else {
                event.end_time = Some(NaiveTime::parse_from_str(&end_input, "%H:%M")
                    .map_err(|_| MmsError::Parse(format!("Invalid time: {}", end_input)))?);
            }
        }
    }

    // Edit room
    let room_input: String = Input::new()
        .with_prompt(format!(
            "Room [{}] (type 'none' to remove)",
            event.room.as_deref().unwrap_or("none")
        ))
        .allow_empty(true)
        .interact_text()?;
    if !room_input.is_empty() {
        if room_input.to_lowercase() == "none" {
            event.room = None;
        } else {
            event.room = Some(room_input);
        }
    }

    // Edit location
    let location_input: String = Input::new()
        .with_prompt(format!(
            "Location [{}] (type 'none' to remove)",
            event.location.as_deref().unwrap_or("none")
        ))
        .allow_empty(true)
        .interact_text()?;
    if !location_input.is_empty() {
        if location_input.to_lowercase() == "none" {
            event.location = None;
        } else {
            event.location = Some(location_input);
        }
    }

    // Edit description
    let desc_input: String = Input::new()
        .with_prompt(format!(
            "Description [{}] (type 'none' to remove)",
            event.description.as_deref().unwrap_or("none")
        ))
        .allow_empty(true)
        .interact_text()?;
    if !desc_input.is_empty() {
        if desc_input.to_lowercase() == "none" {
            event.description = None;
        } else {
            event.description = Some(desc_input);
        }
    }

    queries::event::update(&conn, &event)?;

    println!();
    println!("{}", "✓ Event updated successfully!".green());

    Ok(())
}

fn handle_delete_event(event_id: i64) -> Result<()> {
    let conn = connection::get()?;
    let event = queries::event::get_by_id(&conn, event_id)?;
    let course = queries::course::get_by_id(&conn, event.course_id)?;

    println!("{}", "Delete Event".bold().underline());
    println!();
    println!("Course: {}", course.name.bold());
    println!("Type:   {}", event.event_type.to_str());
    println!("Date:   {}", event.date.format("%d.%m.%Y"));
    if let (Some(start), Some(end)) = (event.start_time, event.end_time) {
        println!("Time:   {} - {}", start.format("%H:%M"), end.format("%H:%M"));
    }
    println!();

    let confirm = dialoguer::Confirm::new()
        .with_prompt("Are you sure you want to delete this event?")
        .default(false)
        .interact()?;

    if !confirm {
        println!("{}", "Cancelled.".yellow());
        return Ok(());
    }

    queries::event::delete(&conn, event_id)?;

    println!("{}", "✓ Event deleted!".green());

    Ok(())
}
