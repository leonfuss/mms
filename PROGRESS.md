# MMS Development Progress

## Completed

### Data Structures (MVP)
✅ All core data models implemented and compiling:

- **Semester** (`src/db/models/semester.rs`)
  - Fields: id, name, type (bachelor/master), number, is_current, default_location
  - Builder pattern with `new()` constructor

- **Course** (`src/db/models/course.rs`)
  - Fields: id, semester_id, name, short_name, category, ects, lecturer, learning_platform_url, location, grade, counts_towards_average
  - Fluent API with `with_*()` methods

- **CourseSchedule** (`src/db/models/schedule.rs`)
  - Fields: id, course_id, schedule_type (lecture/tutorium/exercise), day_of_week, start_time, end_time, start_date, end_date, room, location
  - Supports recurring weekly schedules

- **CourseEvent** (`src/db/models/event.rs`)
  - Fields: id, course_id, course_schedule_id, schedule_type, event_type (one-time/makeup/special/override/cancelled), date, start_time, end_time, room, location, description
  - Handles one-time events, cancellations, and overrides

- **Exam** (`src/db/models/exam.rs`)
  - Fields: id, course_id, exam_type (written/oral/project), date, start_time, end_time, room, location, notes

- **Holiday** (`src/db/models/holiday.rs`)
  - Fields: id, name, start_date, end_date, applies_to_semester_id
  - Supports semester-specific or global holidays

- **HolidayException** (`src/db/models/holiday.rs`)
  - Fields: id, holiday_id, course_schedule_id, exception_date
  - Allows courses to occur during holidays

- **Lecture** (`src/db/models/lecture.rs`)
  - Fields: id, course_id, lecture_number, date, git_commit_hash
  - Tracks lecture history for git commits

- **Todo** (`src/db/models/todo.rs`)
  - Fields: id, course_id, lecture_number, description, completed, auto_clear, created_at, cleared_at
  - Supports course-level and lecture-specific todos

- **ActiveCourse** (`src/db/models/active_course.rs`)
  - Fields: id (always 1), course_id, updated_at
  - Singleton pattern for tracking current active course

- **CategoryRequirement** (`src/db/models/category.rs`)
  - Fields: id, category_name, required_ects, counts_towards_average
  - Defines ECTS requirements per category

### Error Handling
✅ Custom error types (`src/error.rs`):
- Database errors
- Config errors
- Git errors
- IO errors
- Parse errors
- Not found errors (course, semester, schedule, todo, holiday)
- Validation errors (invalid dates, times, types)
- Using `thiserror` for ergonomic error definitions

### Database Infrastructure
✅ Basic database setup (`src/db/`):
- Connection management (`connection.rs`)
- Migration system placeholder (`migrations.rs`)
- Module structure for queries

## Next Steps

### 1. Database Schema & Migrations
- [ ] Create initial migration SQL file
- [ ] Implement migration runner
- [ ] Add schema versioning

### 2. Database Queries (CRUD Operations)
- [ ] Semester queries
- [ ] Course queries
- [ ] Schedule queries
- [ ] Event queries
- [ ] Exam queries
- [ ] Holiday queries
- [ ] Lecture queries
- [ ] Todo queries
- [ ] Stats queries

### 3. Configuration
- [ ] Config file structure
- [ ] Config loading from `~/.config/mms/config.toml`
- [ ] Default values
- [ ] Category requirements loading

### 4. CLI Foundation
- [ ] CLI argument definitions with `clap`
- [ ] Command routing
- [ ] Basic semester commands (add, list, set-current)
- [ ] Basic course commands (add, list, show)

### 5. Utils
- [ ] Path expansion utilities
- [ ] Time/date parsing
- [ ] Output formatting helpers

## Project Status

**Current Phase:** MVP Foundation - Data Structures ✅

**Architecture:**
- Clean separation of concerns (models, queries, CLI, service)
- Repository pattern for database access
- Error handling with custom types
- Builder pattern for model construction

**Compilation:** ✅ All code compiles successfully
**Dependencies:** ✅ All dependencies added to Cargo.toml

**Warnings:** 45 warnings for unused code (expected at this stage)
