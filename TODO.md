# TODO: MMS (My Study Management System) Implementation Plan

## System Architecture Overview

### Components
1. **CLI Application** - Main user interface for managing courses, todos, grades
2. **Background Service (launchd)** - Monitors time-based course schedule and updates symlinks
3. **Local Database (SQLite)** - Stores courses, semesters, todos, grades, and schedules
4. **Schedule Engine** - Determines active course based on stored time schedules
5. **Git Automation** - Auto-commits with lecture numbering

### Technology Stack
- **Rust CLI**: `clap` for argument parsing, `dialoguer` for interactive prompts
- **Database**: `rusqlite` or `sqlx` with SQLite
- **Date/Time**: `chrono` for parsing and comparing times/dates
- **Background Service**: launchd plist file + daemon mode
- **Symlink Management**: `std::os::unix::fs::symlink`
- **Git**: `git2-rs` crate or shell out to git commands
- **Config**: `serde` + `toml` or `json` for configuration

---

## Implementation Strategy

### Phase 1: Core Data Model & Database

#### Database Schema
```sql
-- Semesters
CREATE TABLE semesters (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,  -- e.g., "WS2425", "SS25"
    type TEXT NOT NULL,  -- "bachelor" or "master"
    number INTEGER NOT NULL,  -- 2-digit semester number
    is_current BOOLEAN DEFAULT 0,
    default_location TEXT DEFAULT 'Uni Tübingen',  -- default university for semester
    created_at TIMESTAMP
);

-- Courses
CREATE TABLE courses (
    id INTEGER PRIMARY KEY,
    semester_id INTEGER REFERENCES semesters(id),
    name TEXT NOT NULL,
    short_name TEXT,  -- for directory names
    category TEXT NOT NULL,  -- ML_DIV, ML_FOUND, etc.
    ects INTEGER NOT NULL,
    lecturer TEXT,
    learning_platform_url TEXT,
    location TEXT,  -- NULL means use semester's default_location
    grade REAL,  -- 1.0-5.0, NULL if not graded
    counts_towards_average BOOLEAN DEFAULT 1,
    created_at TIMESTAMP
);

-- Course Schedules (recurring weekly times)
CREATE TABLE course_schedules (
    id INTEGER PRIMARY KEY,
    course_id INTEGER REFERENCES courses(id),
    schedule_type TEXT NOT NULL DEFAULT 'lecture',  -- "lecture", "tutorium", "exercise"
    day_of_week INTEGER NOT NULL,  -- 0=Monday, 6=Sunday
    start_time TIME NOT NULL,  -- e.g., "14:15:00"
    end_time TIME NOT NULL,     -- e.g., "15:45:00"
    start_date DATE NOT NULL,   -- first occurrence date
    end_date DATE NOT NULL,     -- last occurrence date (end of semester/course)
    room TEXT,
    location TEXT,  -- NULL means use course's location (or semester's default)
    UNIQUE(course_id, day_of_week, start_time)
);

-- One-time Events (single lectures, makeup classes, overrides)
CREATE TABLE course_events (
    id INTEGER PRIMARY KEY,
    course_id INTEGER REFERENCES courses(id),
    course_schedule_id INTEGER REFERENCES course_schedules(id),  -- NULL for non-override events
    schedule_type TEXT NOT NULL DEFAULT 'lecture',  -- "lecture", "tutorium", "exercise"
    event_type TEXT NOT NULL,  -- "one-time", "makeup", "special", "override", "cancelled"
    date DATE NOT NULL,
    start_time TIME,  -- NULL for cancelled events
    end_time TIME,    -- NULL for cancelled events
    room TEXT,
    location TEXT,  -- NULL means use course's location (or semester's default)
    description TEXT,
    -- For cancelled events, start_time is NULL so we need course_schedule_id in unique constraint
    UNIQUE(course_id, date, course_schedule_id) WHERE event_type IN ('cancelled', 'override'),
    UNIQUE(course_id, date, start_time) WHERE event_type NOT IN ('cancelled', 'override')
);

-- Exams
CREATE TABLE exams (
    id INTEGER PRIMARY KEY,
    course_id INTEGER REFERENCES courses(id),
    exam_type TEXT NOT NULL,  -- "written", "oral", "project"
    date DATE NOT NULL,
    start_time TIME,
    end_time TIME,
    room TEXT,
    location TEXT,
    notes TEXT
);

-- Holiday Periods (blocks out recurring schedules)
CREATE TABLE holidays (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,  -- e.g., "Winter Break", "Easter Holiday"
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    applies_to_semester_id INTEGER REFERENCES semesters(id),  -- NULL means all semesters
    created_at TIMESTAMP
);

-- Holiday Exceptions (courses that still occur during holidays)
CREATE TABLE holiday_exceptions (
    id INTEGER PRIMARY KEY,
    holiday_id INTEGER REFERENCES holidays(id),
    course_schedule_id INTEGER REFERENCES course_schedules(id),
    exception_date DATE NOT NULL,  -- specific date during holiday that course DOES occur
    UNIQUE(holiday_id, course_schedule_id, exception_date)
);

-- Lectures (for git commit numbering)
CREATE TABLE lectures (
    id INTEGER PRIMARY KEY,
    course_id INTEGER REFERENCES courses(id),
    lecture_number INTEGER NOT NULL,
    date DATE NOT NULL,
    git_commit_hash TEXT,
    UNIQUE(course_id, lecture_number)
);

-- Todos
CREATE TABLE todos (
    id INTEGER PRIMARY KEY,
    course_id INTEGER REFERENCES courses(id),
    lecture_number INTEGER,  -- NULL for course-level todos
    description TEXT NOT NULL,
    completed BOOLEAN DEFAULT 0,
    auto_clear BOOLEAN DEFAULT 1,
    created_at TIMESTAMP,
    cleared_at TIMESTAMP
);

-- Active course tracking
CREATE TABLE active_course (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    course_id INTEGER REFERENCES courses(id),
    updated_at TIMESTAMP
);

-- Category requirements
CREATE TABLE category_requirements (
    id INTEGER PRIMARY KEY,
    category_name TEXT UNIQUE NOT NULL,
    required_ects INTEGER NOT NULL,
    counts_towards_average BOOLEAN DEFAULT 1
);
```

#### Rust Data Structures
- `Semester`, `Course`, `CourseSchedule`, `CourseEvent`, `Exam`, `Holiday`, `HolidayException`, `Lecture`, `Todo`, `CategoryRequirement` structs
- Database abstraction layer with CRUD operations
- Migration system for schema updates

---

### Phase 2: CLI Application

#### Commands Structure
```
mms
├── semester
│   ├── add <name> <type> <number> [--location <location>]
│   ├── list
│   └── set-current <id>
├── course
│   ├── add (interactive prompt)
│   ├── list [--semester <id>]
│   ├── show <id>
│   ├── edit <id>
│   ├── open <id>  (opens learning platform in browser)
│   ├── grade <id> <grade>
│   ├── set-active <id>
│   └── schedule
│       ├── add-recurring <course_id> <day> <start> <end> <start_date> <end_date> [--type <lecture|tutorium|exercise>] [--room <room>]
│       ├── add-event <course_id> <date> <start> <end> [--schedule-type <lecture|tutorium|exercise>] [--room <room>]
│       ├── cancel <schedule_id> <date> [--reason <reason>]
│       ├── override <schedule_id> <date> [--room <room>] [--time <start>-<end>]
│       └── list <course_id>
├── todo
│   ├── add <course_id> [--lecture <num>] <description>
│   ├── list [--course <id>] [--all]
│   ├── done <id>
│   └── show  (overview of all todos)
├── lecture
│   ├── record <course_id> <lecture_num>  (creates git commit)
│   └── list <course_id>
├── exam
│   ├── add <course_id> <date> [--type <type>] [--time <start>-<end>] [--room <room>]
│   ├── list [--upcoming]
│   └── remove <id>
├── holiday
│   ├── add <name> <start_date> <end_date> [--semester <id>]
│   ├── list
│   ├── add-exception <holiday_id> <course_schedule_id> <date>
│   └── remove <id>
├── stats
│   ├── average  (overall average)
│   ├── categories  (ects progress per category)
│   └── overview
└── service
    ├── install  (sets up launchd)
    ├── start
    ├── stop
    └── status
```

#### User Experience Considerations
- Rich terminal UI with `ratatui` or `skim` for todo selection
- Color-coded output with `colored` or `crossterm`
- Interactive prompts for course creation
- Tab completion support

---

### Phase 3: Schedule Engine & Symlink Management

#### Determining Active Course
The service checks stored schedules instead of external calendar:

```rust
fn get_active_course(now: DateTime<Local>) -> Option<Course> {
    let current_date = now.date();
    let current_time = now.time();
    let current_weekday = now.weekday().num_days_from_monday(); // 0=Monday

    // Check for overrides/cancellations first (highest priority)
    // If a course_event exists for this date with event_type "cancelled", skip it
    // If a course_event exists with event_type "override", use its time/room instead
    if let Some(event) = check_course_event_overrides(current_date, current_time) {
        if event.event_type == "cancelled" {
            // Recurring schedule is cancelled for this date, check other courses
            return check_other_courses(current_date, current_time, current_weekday);
        } else {
            // Use override event (different room/time)
            return Some(event.course);
        }
    }

    // Check regular one-time events (lectures not part of recurring schedule)
    if let Some(course) = check_one_time_events(current_date, current_time) {
        return Some(course);
    }

    // Check recurring schedules
    check_recurring_schedules(current_date, current_time, current_weekday)
}

fn check_recurring_schedules(date: Date, time: Time, weekday: u32) -> Option<Course> {
    // Query: SELECT * FROM course_schedules cs
    //   JOIN courses c ON cs.course_id = c.id
    //   WHERE cs.day_of_week = weekday
    //   AND cs.start_date <= date
    //   AND cs.end_date >= date
    //   AND cs.start_time <= time
    //   AND cs.end_time >= time
    //   -- Check not cancelled by course_event override
    //   AND NOT EXISTS (
    //     SELECT 1 FROM course_events ce
    //     WHERE ce.course_schedule_id = cs.id
    //     AND ce.date = date
    //     AND ce.event_type IN ('cancelled', 'override')
    //   )
    //   -- Check not during holiday (unless exception exists)
    //   AND NOT EXISTS (
    //     SELECT 1 FROM holidays h
    //     WHERE h.start_date <= date
    //     AND h.end_date >= date
    //     AND (h.applies_to_semester_id IS NULL OR h.applies_to_semester_id = c.semester_id)
    //     AND NOT EXISTS (
    //       SELECT 1 FROM holiday_exceptions he
    //       WHERE he.holiday_id = h.id
    //       AND he.course_schedule_id = cs.id
    //       AND he.exception_date = date
    //     )
    //   )
}
```

**Advantages over calendar integration:**
- No external dependencies or permissions
- Faster queries (local database)
- More reliable (no API rate limits)
- Easier testing and debugging
- Complete control over scheduling logic

**Schedule Types:**
- **lecture**: Main course lectures (tracked in `lectures` table for git commits)
- **tutorium**: Accompanying tutorial sessions (activates symlink but not tracked for commits)
- **exercise**: Exercise or lab sessions (activates symlink but not tracked for commits)

The symlink switches based on ANY active schedule (lecture, tutorium, or exercise), allowing you to work on the course during tutoriums too. However, only lectures are recorded in the `lectures` table for git commit numbering and history tracking.

#### Symlink Strategy
```rust
// Pseudo-code
fn update_symlink(course: &Course, semester: &Semester) -> Result<()> {
    let target = format!(
        "~/Documents/02_university/{}{:02}/{}",
        semester.type_initial(),  // 'b' or 'm'
        semester.number,
        course.short_name
    );
    let link = "~/cc";

    // Remove existing symlink
    if Path::new(link).exists() {
        fs::remove_file(link)?;
    }

    // Create new symlink if course is active
    if is_course_active(course) {
        unix::fs::symlink(expand_path(&target), expand_path(link))?;
    }

    Ok(())
}
```

#### Edge Cases
- No active course → remove symlink
- Overlapping schedules → prioritize overrides/cancellations first, then one-time events, then recurring schedules
- Course ends → remove symlink, auto-clear todos unless flagged
- Holidays/breaks → automatically block recurring schedules unless holiday_exception exists
- Schedule cancellations → use course_events with event_type "cancelled"
- Room changes → use course_events with event_type "override"

---

### Phase 4: Background Service (launchd)

#### Service Implementation
1. **Daemon mode** in main.rs: `mms service run`
2. **Event loop** that:
   - Checks schedules every 1-5 minutes (configurable)
   - Determines active course from database schedules
   - Updates active course tracking
   - Updates symlink
   - Auto-clears todos when lecture changes
   - Logs activity

#### launchd Configuration
Create `~/Library/LaunchAgents/com.mms.daemon.plist`:
```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.mms.daemon</string>
    <key>ProgramArguments</key>
    <array>
        <string>/path/to/mms</string>
        <string>service</string>
        <string>run</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/tmp/mms.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/mms.err</string>
</dict>
</plist>
```

#### Installation Command
`mms service install` should:
1. Copy binary to stable location (e.g., `~/.local/bin/mms`)
2. Generate plist file
3. Load with `launchctl load ~/Library/LaunchAgents/com.mms.daemon.plist`

---

### Phase 5: Git Automation

#### Auto-commit Strategy
```rust
fn commit_lecture(course: &Course, lecture_num: u32) -> Result<()> {
    let repo_path = get_course_directory(course)?;
    let repo = Repository::open(repo_path)?;

    // Stage all changes
    let mut index = repo.index()?;
    index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
    index.write()?;

    // Create commit
    let oid = repo.index()?.write_tree()?;
    let signature = repo.signature()?;
    let tree = repo.find_tree(oid)?;
    let parent_commit = repo.head()?.peel_to_commit()?;

    let message = format!("Lecture {:02} - {}", lecture_num, course.name);

    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        &message,
        &tree,
        &[&parent_commit]
    )?;

    // Store commit hash in database
    store_lecture_commit(course.id, lecture_num, oid.to_string())?;

    Ok(())
}
```

#### Triggering Auto-commits
- **Option 1**: Manual trigger via `mms lecture record <course_id> <lecture_num>`
- **Option 2**: Auto-trigger when lecture schedule ends (configurable in config)
- **Option 3**: Prompt user at end of lecture (interactive mode)

---

### Phase 6: Statistics & Reporting

#### Average Calculation
```rust
fn calculate_average() -> f32 {
    let grades: Vec<(f32, i32, bool)> = get_all_courses()
        .filter(|c| c.grade.is_some() && c.counts_towards_average)
        .map(|c| (c.grade.unwrap(), c.ects, c.counts_towards_average))
        .collect();

    let total_ects: i32 = grades.iter().map(|(_, ects, _)| ects).sum();
    let weighted_sum: f32 = grades.iter().map(|(grade, ects, _)| grade * (*ects as f32)).sum();

    weighted_sum / (total_ects as f32)
}
```

#### Category Progress
- Calculate ECTS per category
- Compare against requirements
- Show progress bars in CLI

---

## Implementation Priority Order

### MVP (Minimum Viable Product)
1. Config file setup (`~/.config/mms/config.toml`) for paths and defaults
2. Database setup with core tables (semesters, courses, course_schedules, course_events, exams, todos, lectures, holidays, holiday_exceptions)
3. Basic CLI commands: semester add/list, course add/list
4. Schedule management: add recurring schedules and one-time events with room/location
5. Holiday management: add holidays with semester scope and course exceptions
6. Manual symlink management (`mms course set-active`)
7. Todo CRUD operations
8. Basic stats (average grade)

### Enhanced Features
9. Schedule engine: determine active course from stored schedules (with holiday filtering)
10. Background service with auto-symlink
11. Git auto-commits
12. Exam tracking and notifications
13. Rich TUI for todos
14. Category requirements tracking

### Polish
15. Auto-clear todos on lecture transition
16. Browser opening for learning platforms
17. Comprehensive error handling
18. Export/backup functionality
19. Smart lecture numbering (skip holidays automatically)
20. Room/location display in schedule views

---

## Technical Challenges & Solutions

### Challenge 1: Holiday Management Complexity
- **Solution**: Use NOT EXISTS subqueries to filter out holidays, then check for exceptions
- Index on holiday dates for performance
- Cache holiday data in memory during service runtime

### Challenge 2: Background Service Reliability
- **Solution**: launchd with KeepAlive, proper error logging
- Graceful degradation if database temporarily locked

### Challenge 3: Symlink Edge Cases
- **Solution**: Always check if target directory exists before symlinking
- Create directories if they don't exist (with user confirmation)
- Handle race conditions when multiple schedules end simultaneously

### Challenge 4: Git Repository State
- **Solution**: Check for uncommitted changes before auto-commit
- Optionally stash and restore user work

### Challenge 5: Database Migrations
- **Solution**: Use `refinery` or `sqlx` migration tools
- Version database schema

---

## Configuration File Structure

`~/.config/mms/config.toml`:
```toml
[general]
university_base_path = "~/Documents/02_university"
default_location = "Uni Tübingen"
symlink_path = "~/cc"

[service]
schedule_check_interval_minutes = 2
auto_commit_on_lecture_end = true
auto_clear_todos_on_next_lecture = true

[git]
author_name = "Your Name"
author_email = "you@example.com"

[categories.ML_DIV]
required_ects = 24
counts_towards_average = true

[categories.ML_FOUND]
required_ects = 18
counts_towards_average = true

[categories.LANG]
required_ects = 6
counts_towards_average = false
```

---

## Dependencies to Add to Cargo.toml

```toml
[dependencies]
# CLI
clap = { version = "4", features = ["derive"] }
dialoguer = "0.11"
colored = "2"

# Database
rusqlite = { version = "0.31", features = ["bundled"] }
chrono = { version = "0.4", features = ["serde"] }

# Git
git2 = "0.18"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"

# Error handling
anyhow = "1"
thiserror = "1"

# Configuration
config = "0.13"
dirs = "5.0"  # for finding config directory

# Optional: rich TUI
ratatui = "0.25"
crossterm = "0.27"
```

---

## Next Steps

1. Start with database schema and core data structures
2. Implement basic CLI with semester/course management
3. Add todo functionality
4. Test manually before adding automation
5. Implement calendar integration
6. Build background service
7. Add git automation
8. Polish with statistics and TUI
