use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "mms")]
#[command(about = "My Study Management System - Manage your university courses, schedules, and todos", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage semesters
    Semester {
        #[command(subcommand)]
        action: SemesterAction,
    },
    /// Manage courses
    Course {
        #[command(subcommand)]
        action: CourseAction,
    },
    /// Manage schedules and events
    Schedule {
        #[command(subcommand)]
        action: ScheduleAction,
    },
    /// Manage todos
    Todo {
        #[command(subcommand)]
        action: TodoAction,
    },
    /// Manage lectures
    Lecture {
        #[command(subcommand)]
        action: LectureAction,
    },
    /// Manage exams
    Exam {
        #[command(subcommand)]
        action: ExamAction,
    },
    /// Manage holidays
    Holiday {
        #[command(subcommand)]
        action: HolidayAction,
    },
    /// View statistics
    Stats {
        #[command(subcommand)]
        action: StatsAction,
    },
    /// Manage background service
    Service {
        #[command(subcommand)]
        action: ServiceAction,
    },
    /// Check sync status between filesystem and database
    Status,
    /// Sync filesystem with database (create missing folders)
    Sync {
        /// Perform a dry-run without making changes
        #[arg(long)]
        dry_run: bool,
    },
    /// Show today's schedule (events and lectures)
    Today,
    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

// ============================================================================
// Semester Commands
// ============================================================================

#[derive(Subcommand, Debug)]
pub enum SemesterAction {
    /// Add a new semester
    Add {
        /// Semester type: bachelor or master
        #[arg(value_parser = ["bachelor", "b", "master", "m"])]
        type_: String,
        /// Semester number
        number: i32,
        /// Default location for this semester
        #[arg(short, long)]
        location: Option<String>,
    },
    /// List all semesters
    List,
    /// Set the current semester
    SetCurrent {
        /// Semester ID
        id: i64,
    },
}

// ============================================================================
// Course Commands
// ============================================================================

#[derive(Subcommand, Debug)]
pub enum CourseAction {
    /// Add a new course (interactive)
    Add,
    /// List courses
    List {
        /// Semester ID or shortname (optional - uses current semester if not provided)
        semester: Option<String>,
    },
    /// Show course details
    Show {
        /// Course ID
        id: i64,
    },
    /// Edit a course
    Edit {
        /// Course ID
        id: i64,
    },
    /// Open course learning platform in browser
    Open {
        /// Course ID
        id: i64,
    },
    /// Set course grade
    Grade {
        /// Course ID
        id: i64,
        /// Grade (1.0-5.0 or 'none' to remove)
        grade: String,
    },
    /// Set active course manually
    SetActive {
        /// Course ID
        id: i64,
    },
}

#[derive(Subcommand, Debug)]
pub enum ScheduleAction {
    /// Add a schedule or event (interactive if no arguments provided)
    Add {
        /// Course ID or shortname (optional - uses active course if not provided)
        course: Option<String>,
        /// Day of week (monday, tuesday, etc.) - required for recurring schedules
        #[arg(value_parser = ["monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday"])]
        day: Option<String>,
        /// Date (dd.mm.yyyy) - required for one-time events
        date: Option<String>,
        /// Start time (HH:MM)
        start: Option<String>,
        /// End time (HH:MM)
        end: Option<String>,
        /// Start date (dd.mm.yyyy) - for recurring schedules
        start_date: Option<String>,
        /// End date (dd.mm.yyyy) - for recurring schedules
        end_date: Option<String>,
        /// Make this a recurring weekly schedule
        #[arg(long)]
        recurring: bool,
        /// Schedule type
        #[arg(short = 't', long, value_parser = ["lecture", "tutorium", "exercise"])]
        schedule_type: Option<String>,
        /// Room
        #[arg(short = 'r', long)]
        room: Option<String>,
        /// Location (overrides course/semester default)
        #[arg(short, long)]
        location: Option<String>,
        /// Description (for one-time events)
        #[arg(short = 'd', long)]
        description: Option<String>,
    },
    /// Cancel a specific occurrence of a recurring schedule
    Cancel {
        /// Schedule ID
        schedule_id: i64,
        /// Date to cancel (dd.mm.yyyy)
        date: String,
        /// Reason for cancellation
        #[arg(short, long)]
        reason: Option<String>,
    },
    /// Override a specific occurrence (change room/time)
    Override {
        /// Schedule ID
        schedule_id: i64,
        /// Date to override (dd.mm.yyyy)
        date: String,
        /// New room
        #[arg(short, long)]
        room: Option<String>,
        /// New time range (HH:MM-HH:MM)
        #[arg(short, long)]
        time: Option<String>,
    },
    /// List weekly schedule overview (or for specific course)
    List {
        /// Course ID or shortname (optional - shows all courses if not specified)
        course: Option<String>,
    },
    /// Edit a schedule or event
    Edit {
        /// Schedule or event ID
        id: i64,
        /// Edit an event (default is schedule)
        #[arg(short, long)]
        event: bool,
    },
    /// Delete a schedule or event
    Delete {
        /// Schedule or event ID
        id: i64,
        /// Delete an event (default is schedule)
        #[arg(short, long)]
        event: bool,
    },
}

// ============================================================================
// Todo Commands
// ============================================================================

#[derive(Subcommand, Debug)]
pub enum TodoAction {
    /// Add a new todo
    Add {
        /// Todo description
        description: String,
        /// Course ID or shortname (optional - infers from directory or uses active course)
        #[arg(short = 'c', long)]
        course: Option<String>,
        /// Lecture number (optional)
        #[arg(short, long)]
        lecture: Option<i32>,
        /// Disable auto-clear
        #[arg(long)]
        no_auto_clear: bool,
    },
    /// List todos
    List {
        /// Filter by course ID or shortname (optional)
        #[arg(short = 'c', long)]
        course: Option<String>,
        /// Show all todos (including completed)
        #[arg(short, long)]
        all: bool,
    },
    /// Show todo overview
    Show,
    /// Mark todo as done
    Done {
        /// Todo ID
        id: i64,
    },
}

// ============================================================================
// Lecture Commands
// ============================================================================

#[derive(Subcommand, Debug)]
pub enum LectureAction {
    /// Record a lecture (create git commit)
    Record {
        /// Lecture number
        lecture_num: i32,
        /// Course ID or shortname (optional - infers from directory or uses active course)
        #[arg(short = 'c', long)]
        course: Option<String>,
    },
    /// List lectures for a course
    List {
        /// Course ID or shortname (optional - infers from directory or uses active course)
        #[arg(short = 'c', long)]
        course: Option<String>,
    },
}

// ============================================================================
// Exam Commands
// ============================================================================

#[derive(Subcommand, Debug)]
pub enum ExamAction {
    /// Add a new exam
    Add {
        /// Exam date (YYYY-MM-DD)
        date: String,
        /// Course ID or shortname (optional - infers from directory or uses active course)
        #[arg(short = 'c', long)]
        course: Option<String>,
        /// Exam type
        #[arg(short = 't', long, value_parser = ["written", "oral", "project"], default_value = "written")]
        exam_type: String,
        /// Time range (HH:MM-HH:MM)
        #[arg(long)]
        time: Option<String>,
        /// Room
        #[arg(short, long)]
        room: Option<String>,
        /// Location
        #[arg(short, long)]
        location: Option<String>,
        /// Notes
        #[arg(short, long)]
        notes: Option<String>,
    },
    /// List exams
    List {
        /// Show only upcoming exams
        #[arg(short, long)]
        upcoming: bool,
    },
    /// Remove an exam
    Remove {
        /// Exam ID
        id: i64,
    },
}

// ============================================================================
// Holiday Commands
// ============================================================================

#[derive(Subcommand, Debug)]
pub enum HolidayAction {
    /// Add a holiday period
    Add {
        /// Holiday name (e.g., "Winter Break")
        name: String,
        /// Start date (YYYY-MM-DD)
        start_date: String,
        /// End date (YYYY-MM-DD)
        end_date: String,
        /// Apply only to specific semester (optional - infers from directory or uses current semester)
        #[arg(short, long)]
        semester: Option<String>,
    },
    /// List holidays
    List,
    /// Add an exception (course still occurs during holiday)
    AddException {
        /// Holiday ID
        holiday_id: i64,
        /// Course schedule ID
        course_schedule_id: i64,
        /// Exception date (YYYY-MM-DD)
        date: String,
    },
    /// Remove a holiday
    Remove {
        /// Holiday ID
        id: i64,
    },
}

// ============================================================================
// Stats Commands
// ============================================================================

#[derive(Subcommand, Debug)]
pub enum StatsAction {
    /// Show grade average
    Average,
    /// Show ECTS progress by category
    Categories,
    /// Show overview of all statistics
    Overview,
}

// ============================================================================
// Service Commands
// ============================================================================

#[derive(Subcommand, Debug)]
pub enum ServiceAction {
    /// Install the background service (launchd)
    Install,
    /// Uninstall the background service (launchd)
    Uninstall,
    /// Start the service
    Start,
    /// Stop the service
    Stop,
    /// Show service status
    Status,
    /// Run the service in foreground (for debugging)
    Run,
}

// ============================================================================
// Config Commands
// ============================================================================

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Initialize config file with defaults
    Init,
    /// Show current configuration
    Show,
    /// Edit configuration file
    Edit,
}
