# MMS Architecture

This document describes the high-level architecture and file structure for the MMS (My Study Management System) project.

## Project Structure

```
mms/
├── Cargo.toml
├── CLAUDE.md
├── TODO.md
├── ARCHITECTURE.md
├── src/
│   ├── main.rs                    # Entry point, CLI setup
│   ├── cli/
│   │   ├── mod.rs                 # CLI module root
│   │   ├── commands/
│   │   │   ├── mod.rs             # Command module exports
│   │   │   ├── semester.rs        # Semester commands
│   │   │   ├── course.rs          # Course commands
│   │   │   ├── schedule.rs        # Schedule management commands
│   │   │   ├── todo.rs            # Todo commands
│   │   │   ├── lecture.rs         # Lecture recording commands
│   │   │   ├── exam.rs            # Exam management commands
│   │   │   ├── holiday.rs         # Holiday management commands
│   │   │   ├── stats.rs           # Statistics commands
│   │   │   └── service.rs         # Service management commands
│   │   └── args.rs                # CLI argument definitions (clap)
│   ├── db/
│   │   ├── mod.rs                 # Database module root
│   │   ├── connection.rs          # SQLite connection management
│   │   ├── migrations.rs          # Database migration logic
│   │   ├── models/
│   │   │   ├── mod.rs             # Model exports
│   │   │   ├── semester.rs        # Semester struct & methods
│   │   │   ├── course.rs          # Course struct & methods
│   │   │   ├── schedule.rs        # CourseSchedule struct & methods
│   │   │   ├── event.rs           # CourseEvent struct & methods
│   │   │   ├── exam.rs            # Exam struct & methods
│   │   │   ├── holiday.rs         # Holiday & HolidayException structs
│   │   │   ├── lecture.rs         # Lecture struct & methods
│   │   │   ├── todo.rs            # Todo struct & methods
│   │   │   ├── active_course.rs   # ActiveCourse struct & methods
│   │   │   └── category.rs        # CategoryRequirement struct & methods
│   │   └── queries/
│   │       ├── mod.rs             # Query module exports
│   │       ├── semester.rs        # Semester CRUD queries
│   │       ├── course.rs          # Course CRUD queries
│   │       ├── schedule.rs        # Schedule CRUD queries
│   │       ├── event.rs           # Event CRUD queries
│   │       ├── exam.rs            # Exam CRUD queries
│   │       ├── holiday.rs         # Holiday CRUD queries
│   │       ├── lecture.rs         # Lecture CRUD queries
│   │       ├── todo.rs            # Todo CRUD queries
│   │       └── stats.rs           # Statistics queries
│   ├── service/
│   │   ├── mod.rs                 # Service module root
│   │   ├── daemon.rs              # Background service daemon loop
│   │   ├── scheduler.rs           # Schedule engine (determine active course)
│   │   ├── symlink.rs             # Symlink management
│   │   ├── git.rs                 # Git automation (commits)
│   │   └── installer.rs           # launchd service installation
│   ├── config/
│   │   ├── mod.rs                 # Config module root
│   │   └── settings.rs            # Config file loading & structures
│   ├── utils/
│   │   ├── mod.rs                 # Utils module root
│   │   ├── path.rs                # Path expansion and validation
│   │   ├── time.rs                # Time/date utilities
│   │   └── format.rs              # Output formatting helpers
│   └── error.rs                   # Error types and handling
├── migrations/
│   ├── 001_initial_schema.sql     # Initial database schema
│   ├── 002_add_holidays.sql       # Holiday tables (if incremental)
│   └── ...                        # Future migrations
└── tests/
    ├── integration/
    │   ├── semester_tests.rs
    │   ├── course_tests.rs
    │   ├── schedule_tests.rs
    │   └── ...
    └── fixtures/
        └── test_data.sql
```

## Module Architecture

### 1. Main Entry Point (`src/main.rs`)

**Responsibility:**
- Parse CLI arguments using `clap`
- Route to appropriate command handlers
- Initialize config and database connection
- Handle top-level errors

**Flow:**
```rust
fn main() -> Result<()> {
    // Parse CLI args
    let cli = Cli::parse();

    // Load config
    let config = Config::load()?;

    // Initialize database
    let db = Database::connect(&config)?;

    // Route to command
    match cli.command {
        Commands::Semester(args) => semester::handle(args, &db, &config),
        Commands::Course(args) => course::handle(args, &db, &config),
        // ... other commands
    }
}
```

---

### 2. CLI Module (`src/cli/`)

**Responsibility:**
- Define CLI structure and arguments
- Implement command handlers
- User interaction (prompts, confirmations)
- Output formatting

**Structure:**
- `args.rs`: All `clap` derive structures for argument parsing
- `commands/`: Individual command implementations that call into db/service layers

**Example command handler:**
```rust
// src/cli/commands/semester.rs
pub fn handle(args: SemesterArgs, db: &Database, config: &Config) -> Result<()> {
    match args.action {
        SemesterAction::Add { name, type_, number, location } => {
            let semester = Semester::new(name, type_, number, location);
            db.semesters().insert(&semester)?;
            println!("✓ Semester added: {}", semester.name);
        }
        SemesterAction::List => {
            let semesters = db.semesters().list()?;
            display_semester_table(semesters);
        }
        // ...
    }
    Ok(())
}
```

---

### 3. Database Module (`src/db/`)

**Responsibility:**
- SQLite connection management
- Database migrations
- Data models (structs)
- CRUD operations
- Complex queries

**Structure:**
- `connection.rs`: Connection pool, transaction handling
- `migrations.rs`: Schema versioning and migration runner
- `models/`: Rust structs representing database tables
- `queries/`: Database operations organized by entity

**Design Pattern: Repository Pattern**

Each entity has:
1. A model struct in `models/`
2. A query module in `queries/` with methods like:
   - `insert()`, `update()`, `delete()`, `get_by_id()`, `list()`
   - Complex queries specific to that entity

**Example:**
```rust
// src/db/models/course.rs
#[derive(Debug, Clone)]
pub struct Course {
    pub id: Option<i64>,
    pub semester_id: i64,
    pub name: String,
    pub short_name: String,
    pub category: String,
    pub ects: i32,
    pub lecturer: Option<String>,
    pub learning_platform_url: Option<String>,
    pub location: Option<String>,
    pub grade: Option<f32>,
    pub counts_towards_average: bool,
}

// src/db/queries/course.rs
pub struct CourseQueries<'a> {
    conn: &'a Connection,
}

impl<'a> CourseQueries<'a> {
    pub fn insert(&self, course: &Course) -> Result<i64> { ... }
    pub fn get_by_id(&self, id: i64) -> Result<Course> { ... }
    pub fn list_by_semester(&self, semester_id: i64) -> Result<Vec<Course>> { ... }
    pub fn update(&self, course: &Course) -> Result<()> { ... }
    pub fn delete(&self, id: i64) -> Result<()> { ... }
}
```

**Database facade:**
```rust
// src/db/mod.rs
pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn semesters(&self) -> SemesterQueries { ... }
    pub fn courses(&self) -> CourseQueries { ... }
    pub fn schedules(&self) -> ScheduleQueries { ... }
    // ... other query accessors
}
```

---

### 4. Service Module (`src/service/`)

**Responsibility:**
- Background service daemon
- Schedule engine (determine active course)
- Symlink management
- Git automation
- launchd service installation

**Key Components:**

#### `scheduler.rs` - Schedule Engine
```rust
pub struct ScheduleEngine<'a> {
    db: &'a Database,
}

impl<'a> ScheduleEngine<'a> {
    pub fn get_active_course(&self, now: DateTime<Local>) -> Result<Option<Course>> {
        // Priority:
        // 1. Check for overrides/cancellations
        // 2. Check one-time events
        // 3. Check recurring schedules (filtered by holidays)
    }

    fn check_overrides(&self, date: Date, time: Time) -> Result<Option<CourseEvent>> { ... }
    fn check_one_time_events(&self, date: Date, time: Time) -> Result<Option<Course>> { ... }
    fn check_recurring_schedules(&self, date: Date, time: Time, weekday: u32) -> Result<Option<Course>> { ... }
}
```

#### `symlink.rs` - Symlink Management
```rust
pub struct SymlinkManager {
    config: Config,
}

impl SymlinkManager {
    pub fn update(&self, course: Option<&Course>, semester: &Semester) -> Result<()> {
        // Remove existing symlink
        // Create new symlink if course is active
        // Create directory if it doesn't exist
    }
}
```

#### `daemon.rs` - Background Service
```rust
pub struct Daemon {
    db: Database,
    config: Config,
    scheduler: ScheduleEngine,
    symlink_manager: SymlinkManager,
}

impl Daemon {
    pub fn run(&self) -> Result<()> {
        loop {
            let interval = self.config.service.schedule_check_interval_minutes;

            // Determine active course
            let active_course = self.scheduler.get_active_course(Local::now())?;

            // Update symlink
            self.update_active_course(active_course)?;

            // Auto-clear todos if configured
            if self.config.service.auto_clear_todos_on_next_lecture {
                self.check_and_clear_todos()?;
            }

            // Sleep until next check
            thread::sleep(Duration::from_secs(interval * 60));
        }
    }
}
```

#### `git.rs` - Git Automation
```rust
pub struct GitManager {
    config: Config,
}

impl GitManager {
    pub fn commit_lecture(&self, course: &Course, lecture_num: u32) -> Result<String> {
        // Open repository
        // Stage all changes
        // Create commit with "Lecture XX - Course Name"
        // Return commit hash
    }
}
```

---

### 5. Config Module (`src/config/`)

**Responsibility:**
- Load and parse `~/.config/mms/config.toml`
- Provide default values
- Config validation

**Structure:**
```rust
// src/config/settings.rs
#[derive(Debug, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub service: ServiceConfig,
    pub git: GitConfig,
    pub categories: HashMap<String, CategoryConfig>,
}

#[derive(Debug, Deserialize)]
pub struct GeneralConfig {
    pub university_base_path: PathBuf,
    pub default_location: String,
    pub symlink_path: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct ServiceConfig {
    pub schedule_check_interval_minutes: u64,
    pub auto_commit_on_lecture_end: bool,
    pub auto_clear_todos_on_next_lecture: bool,
}

#[derive(Debug, Deserialize)]
pub struct GitConfig {
    pub author_name: String,
    pub author_email: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = dirs::config_dir()
            .ok_or(anyhow!("Could not find config directory"))?
            .join("mms")
            .join("config.toml");

        if !config_path.exists() {
            return Self::default();
        }

        let content = fs::read_to_string(config_path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn default() -> Self {
        // Return default config
    }
}
```

---

### 6. Utils Module (`src/utils/`)

**Responsibility:**
- Common utility functions
- Path manipulation
- Time/date helpers
- Formatting

**Examples:**
```rust
// src/utils/path.rs
pub fn expand_tilde(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(&path[2..]);
        }
    }
    PathBuf::from(path)
}

pub fn ensure_directory_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

// src/utils/time.rs
pub fn parse_time(s: &str) -> Result<NaiveTime> {
    NaiveTime::parse_from_str(s, "%H:%M")
        .or_else(|_| NaiveTime::parse_from_str(s, "%H:%M:%S"))
        .context("Invalid time format. Use HH:MM or HH:MM:SS")
}

pub fn parse_date(s: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .context("Invalid date format. Use YYYY-MM-DD")
}

// src/utils/format.rs
pub fn format_grade(grade: f32) -> String {
    format!("{:.1}", grade)
}

pub fn display_course_table(courses: Vec<Course>) {
    // Pretty table formatting
}
```

---

### 7. Error Handling (`src/error.rs`)

**Responsibility:**
- Custom error types
- Error conversions
- User-friendly error messages

**Structure:**
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MmsError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Course not found: {0}")]
    CourseNotFound(i64),

    #[error("Semester not found: {0}")]
    SemesterNotFound(i64),

    #[error("No active course at this time")]
    NoActiveCourse,

    #[error("Invalid schedule: {0}")]
    InvalidSchedule(String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, MmsError>;
```

---

## Data Flow

### Example: Adding a Course Schedule

```
User runs: mms course schedule add-recurring 1 monday 14:00 16:00 2024-10-14 2025-02-14 --type lecture --room "HS N14"

1. main.rs
   ↓ Parse args with clap

2. cli/commands/schedule.rs::handle()
   ↓ Validate input (dates, times)
   ↓ Parse day_of_week, start_time, end_time

3. db/queries/schedule.rs::insert()
   ↓ Insert into course_schedules table
   ↓ Return schedule ID

4. cli/commands/schedule.rs
   ↓ Display success message

5. User sees: "✓ Schedule added: Monday 14:00-16:00 (HS N14)"
```

### Example: Background Service Updating Symlink

```
service/daemon.rs runs every 2 minutes

1. daemon.rs::run()
   ↓ Get current time

2. service/scheduler.rs::get_active_course()
   ↓ Query database for active course
   ↓ Check overrides → events → recurring schedules
   ↓ Filter by holidays
   ↓ Return active course (or None)

3. service/symlink.rs::update()
   ↓ Remove existing symlink
   ↓ If active course exists:
   │  ↓ Get semester info
   │  ↓ Build target path
   │  ↓ Create symlink

4. db/queries/active_course.rs::update()
   ↓ Update active_course table
   ↓ Log state change
```

---

## Testing Strategy

### Unit Tests
- Place unit tests in same file as implementation (Rust convention)
- Test individual functions in isolation
- Mock database with in-memory SQLite

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_time() {
        assert_eq!(parse_time("14:30").unwrap(), NaiveTime::from_hms(14, 30, 0));
    }
}
```

### Integration Tests
- Place in `tests/` directory
- Test complete command flows
- Use test database with fixtures

```rust
// tests/integration/schedule_tests.rs
#[test]
fn test_add_recurring_schedule() {
    let db = setup_test_db();
    let config = Config::default();

    // Add semester and course
    let semester_id = db.semesters().insert(...)?;
    let course_id = db.courses().insert(...)?;

    // Add schedule
    let schedule = CourseSchedule { ... };
    let schedule_id = db.schedules().insert(&schedule)?;

    // Verify schedule was added
    let retrieved = db.schedules().get_by_id(schedule_id)?;
    assert_eq!(retrieved.day_of_week, 0); // Monday
}
```

---

## Build and Development

### Development Mode
```bash
cargo run -- semester add WS2425 master 01
cargo run -- course add
cargo run -- service run  # Run service in foreground for debugging
```

### Testing
```bash
cargo test                    # All tests
cargo test --test schedule_tests  # Specific integration test
cargo test db::queries        # Tests for db/queries module
```

### Building Release
```bash
cargo build --release
# Binary at: target/release/mms
```

### Installation
```bash
cargo install --path .
# Or manually:
cp target/release/mms ~/.local/bin/
mms service install  # Sets up launchd
```

---

## Key Design Principles

1. **Separation of Concerns**: CLI, database, service logic are separate
2. **Repository Pattern**: Database access is abstracted through query modules
3. **Single Responsibility**: Each module has a clear, focused purpose
4. **Testability**: Business logic is independent of CLI/IO
5. **Error Handling**: Use `Result<T>` everywhere, provide context with `anyhow`
6. **Configuration**: Externalize configuration, provide sensible defaults
7. **Rust Idioms**: Use `Option` for nullable fields, `Result` for fallible operations

---

## Future Considerations

### Potential Additions
- **Web UI**: Add `actix-web` server, expose REST API
- **Sync**: Add cloud sync with encrypted backup
- **Notifications**: Desktop notifications for upcoming exams
- **Analytics**: Track study time, productivity metrics
- **Import/Export**: JSON/CSV import of courses and schedules
- **Calendar Integration**: Optional iCal export for viewing in Calendar.app

### Scalability
- Current SQLite design handles single-user local use well
- For multi-user: consider PostgreSQL backend
- For sync: consider CRDT or event sourcing patterns
