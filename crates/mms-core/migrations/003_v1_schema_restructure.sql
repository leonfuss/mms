-- Migration 003: V1 Schema Restructure
-- Implements the full MMS 1.0 Schema

-- Disable foreign keys during reconstruction
PRAGMA foreign_keys = OFF;

-- Drop existing tables (cleanup old schema)
DROP TABLE IF EXISTS active;
DROP TABLE IF EXISTS active_course;
DROP TABLE IF EXISTS todos;
DROP TABLE IF EXISTS lectures;
DROP TABLE IF EXISTS holiday_exceptions;
DROP TABLE IF EXISTS holidays;
DROP TABLE IF EXISTS exams;
DROP TABLE IF EXISTS course_events;
DROP TABLE IF EXISTS course_schedules;
DROP TABLE IF EXISTS courses;
DROP TABLE IF EXISTS semesters;
DROP TABLE IF EXISTS category_requirements;
DROP TABLE IF EXISTS degrees;
DROP TABLE IF EXISTS degree_areas;
DROP TABLE IF EXISTS course_possible_categories;
DROP TABLE IF EXISTS course_degree_mappings;
DROP TABLE IF EXISTS grades;
DROP TABLE IF EXISTS grade_components;
DROP TABLE IF EXISTS exam_attempts;
DROP TABLE IF EXISTS slides;
DROP TABLE IF EXISTS exercises;
DROP TABLE IF EXISTS platform_accounts;
DROP TABLE IF EXISTS platform_course_links;

-- Drop existing views
DROP VIEW IF EXISTS v_orphaned_courses;
DROP VIEW IF EXISTS v_current_gpa;
DROP VIEW IF EXISTS v_degree_progress;
DROP VIEW IF EXISTS v_course_categories;
DROP VIEW IF EXISTS v_unmapped_courses;
DROP VIEW IF EXISTS v_degree_progress_extended;

-- Re-enable foreign keys
PRAGMA foreign_keys = ON;

-- ==========================================
-- Core Entity Tables
-- ==========================================

-- Semesters
CREATE TABLE semesters (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    -- Core identifiers
    type TEXT NOT NULL CHECK(type IN ('Bachelor', 'Master')),
    number INTEGER NOT NULL,

    -- Filesystem tracking
    directory_path TEXT NOT NULL,
    exists_on_disk BOOLEAN NOT NULL DEFAULT 1,
    last_scanned_at DATETIME,

    -- Metadata
    start_date TEXT,
    end_date TEXT,
    default_location TEXT NOT NULL,
    university TEXT,

    -- State
    is_current BOOLEAN NOT NULL DEFAULT 0,
    is_archived BOOLEAN NOT NULL DEFAULT 0,

    -- Timestamps
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(type, number)
);

CREATE INDEX idx_semesters_current ON semesters(is_current);
CREATE INDEX idx_semesters_archived ON semesters(is_archived);
CREATE INDEX idx_semesters_disk ON semesters(exists_on_disk);

-- Degrees
CREATE TABLE degrees (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    -- Identifiers
    type TEXT NOT NULL CHECK(type IN ('Bachelor', 'Master', 'PhD')),
    name TEXT NOT NULL,
    university TEXT NOT NULL,

    -- Requirements
    total_ects_required INTEGER NOT NULL,

    -- State
    is_active BOOLEAN NOT NULL DEFAULT 1,
    start_date TEXT,
    expected_end_date TEXT,

    -- Timestamps
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(type, name, university)
);

CREATE INDEX idx_degrees_active ON degrees(is_active);

-- Degree Areas
CREATE TABLE degree_areas (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    degree_id INTEGER NOT NULL,

    -- Area definition
    category_name TEXT NOT NULL,
    required_ects INTEGER NOT NULL,
    counts_towards_gpa BOOLEAN NOT NULL DEFAULT 1,

    -- Display
    display_order INTEGER NOT NULL DEFAULT 0,

    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (degree_id) REFERENCES degrees(id) ON DELETE CASCADE,
    UNIQUE(degree_id, category_name)
);

CREATE INDEX idx_degree_areas_degree ON degree_areas(degree_id);

-- Courses
CREATE TABLE courses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    semester_id INTEGER NOT NULL,

    -- Core identifiers
    short_name TEXT NOT NULL,
    name TEXT NOT NULL,

    -- Filesystem tracking
    directory_path TEXT NOT NULL,
    toml_path TEXT,
    exists_on_disk BOOLEAN NOT NULL DEFAULT 1,
    toml_exists BOOLEAN NOT NULL DEFAULT 1,
    last_scanned_at DATETIME,

    -- Metadata
    ects INTEGER NOT NULL,
    lecturer TEXT,
    lecturer_email TEXT,
    tutor TEXT,
    tutor_email TEXT,
    learning_platform_url TEXT,
    university TEXT,
    location TEXT,

    -- State flags
    is_external BOOLEAN NOT NULL DEFAULT 0,
    original_path TEXT,
    is_archived BOOLEAN NOT NULL DEFAULT 0,
    is_dropped BOOLEAN NOT NULL DEFAULT 0,
    dropped_at DATETIME,

    -- Git integration
    has_git_repo BOOLEAN NOT NULL DEFAULT 0,
    git_remote_url TEXT,

    -- Timestamps
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (semester_id) REFERENCES semesters(id) ON DELETE CASCADE,
    UNIQUE(semester_id, short_name)
);

CREATE INDEX idx_courses_semester ON courses(semester_id);
CREATE INDEX idx_courses_disk ON courses(exists_on_disk);
CREATE INDEX idx_courses_archived ON courses(is_archived);
CREATE INDEX idx_courses_external ON courses(is_external);
CREATE INDEX idx_courses_toml ON courses(toml_exists);

-- Course Possible Categories
CREATE TABLE course_possible_categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    course_id INTEGER NOT NULL,
    degree_id INTEGER NOT NULL,
    area_id INTEGER NOT NULL,

    -- Metadata
    is_recommended BOOLEAN NOT NULL DEFAULT 0,
    notes TEXT,

    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE,
    FOREIGN KEY (degree_id) REFERENCES degrees(id) ON DELETE CASCADE,
    FOREIGN KEY (area_id) REFERENCES degree_areas(id) ON DELETE CASCADE,
    UNIQUE(course_id, degree_id, area_id)
);

CREATE INDEX idx_possible_categories_course ON course_possible_categories(course_id);
CREATE INDEX idx_possible_categories_degree ON course_possible_categories(degree_id);
CREATE INDEX idx_possible_categories_area ON course_possible_categories(area_id);
CREATE INDEX idx_possible_categories_recommended ON course_possible_categories(is_recommended);

-- Course Degree Mappings
CREATE TABLE course_degree_mappings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    course_id INTEGER NOT NULL,
    degree_id INTEGER NOT NULL,
    area_id INTEGER NOT NULL,

    -- Override settings
    ects_override INTEGER,

    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE,
    FOREIGN KEY (degree_id) REFERENCES degrees(id) ON DELETE CASCADE,
    FOREIGN KEY (area_id) REFERENCES degree_areas(id) ON DELETE CASCADE,
    UNIQUE(course_id, degree_id, area_id)
);

CREATE INDEX idx_mappings_course ON course_degree_mappings(course_id);
CREATE INDEX idx_mappings_degree ON course_degree_mappings(degree_id);

-- ==========================================
-- Grading Tables
-- ==========================================

-- Grades
CREATE TABLE grades (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    course_id INTEGER NOT NULL,

    -- Grade value
    grade REAL NOT NULL,
    grading_scheme TEXT NOT NULL,

    -- Conversion
    original_grade REAL,
    original_scheme TEXT,
    conversion_table TEXT,

    -- State
    is_final BOOLEAN NOT NULL DEFAULT 1,
    passed BOOLEAN NOT NULL,

    -- Exam attempt tracking
    attempt_number INTEGER NOT NULL DEFAULT 1,
    exam_date TEXT,

    -- Timestamps
    recorded_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE
);

CREATE INDEX idx_grades_course ON grades(course_id);
CREATE INDEX idx_grades_final ON grades(is_final);

-- Grade Components
CREATE TABLE grade_components (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    course_id INTEGER NOT NULL,
    grade_id INTEGER,

    -- Component definition
    component_name TEXT NOT NULL,
    weight REAL NOT NULL,

    -- Score
    points_earned REAL,
    points_total REAL,
    grade REAL,

    -- Bonus
    is_bonus BOOLEAN NOT NULL DEFAULT 0,
    bonus_points REAL DEFAULT 0,

    -- State
    is_completed BOOLEAN NOT NULL DEFAULT 0,

    -- Timestamps
    due_date TEXT,
    completed_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE,
    FOREIGN KEY (grade_id) REFERENCES grades(id) ON DELETE SET NULL
);

CREATE INDEX idx_components_course ON grade_components(course_id);
CREATE INDEX idx_components_grade ON grade_components(grade_id);

-- Exam Attempts
CREATE TABLE exam_attempts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    course_id INTEGER NOT NULL,

    -- Attempt info
    attempt_number INTEGER NOT NULL,
    exam_date TEXT NOT NULL,
    exam_type TEXT CHECK(exam_type IN ('Regular', 'Retake', 'Makeup', 'Special')),

    -- Result
    grade REAL,
    passed BOOLEAN NOT NULL DEFAULT 0,
    grade_id INTEGER,

    -- Details
    notes TEXT,
    location TEXT,

    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE,
    FOREIGN KEY (grade_id) REFERENCES grades(id) ON DELETE SET NULL,
    UNIQUE(course_id, attempt_number)
);

CREATE INDEX idx_attempts_course ON exam_attempts(course_id);
CREATE INDEX idx_attempts_passed ON exam_attempts(passed);

-- ==========================================
-- Schedule Tables
-- ==========================================

-- Course Schedules
CREATE TABLE course_schedules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    course_id INTEGER NOT NULL,

    -- Schedule definition
    schedule_type TEXT NOT NULL CHECK(schedule_type IN ('Lecture', 'Tutorium', 'Exercise', 'Lab')),
    day_of_week INTEGER NOT NULL CHECK(day_of_week BETWEEN 0 AND 6),
    start_time TEXT NOT NULL,
    end_time TEXT NOT NULL,

    -- Validity period
    start_date TEXT NOT NULL,
    end_date TEXT NOT NULL,

    -- Location
    room TEXT,
    building TEXT,
    location TEXT,

    -- Priority
    priority INTEGER NOT NULL DEFAULT 0,

    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE
);

CREATE INDEX idx_schedules_course ON course_schedules(course_id);
CREATE INDEX idx_schedules_day ON course_schedules(day_of_week);
CREATE INDEX idx_schedules_time ON course_schedules(start_time);

-- Course Events
CREATE TABLE course_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    course_id INTEGER NOT NULL,
    schedule_id INTEGER,

    -- Event details
    event_type TEXT NOT NULL CHECK(event_type IN ('OneTime', 'Cancellation', 'RoomChange', 'TimeChange')),
    date TEXT NOT NULL,
    start_time TEXT,
    end_time TEXT,

    -- Location
    room TEXT,
    building TEXT,
    location TEXT,

    -- Details
    title TEXT,
    description TEXT,

    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE,
    FOREIGN KEY (schedule_id) REFERENCES course_schedules(id) ON DELETE CASCADE
);

CREATE INDEX idx_events_course ON course_events(course_id);
CREATE INDEX idx_events_date ON course_events(date);
CREATE INDEX idx_events_type ON course_events(event_type);

-- Holidays
CREATE TABLE holidays (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    -- Holiday definition
    name TEXT NOT NULL,
    start_date TEXT NOT NULL,
    end_date TEXT NOT NULL,
    university TEXT,

    -- Type
    holiday_type TEXT NOT NULL CHECK(holiday_type IN ('Public', 'Semester Break', 'Exam Period', 'Other')),

    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(name, start_date, university)
);

CREATE INDEX idx_holidays_dates ON holidays(start_date, end_date);
CREATE INDEX idx_holidays_university ON holidays(university);

-- Holiday Exceptions
CREATE TABLE holiday_exceptions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    holiday_id INTEGER NOT NULL,
    course_id INTEGER NOT NULL,

    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (holiday_id) REFERENCES holidays(id) ON DELETE CASCADE,
    FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE,
    UNIQUE(holiday_id, course_id)
);

CREATE INDEX idx_exceptions_holiday ON holiday_exceptions(holiday_id);
CREATE INDEX idx_exceptions_course ON holiday_exceptions(course_id);

-- ==========================================
-- Lecture & Exercise Tables
-- ==========================================

-- Lectures
CREATE TABLE lectures (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    course_id INTEGER NOT NULL,

    -- Lecture info
    lecture_number INTEGER NOT NULL,
    schedule_type TEXT NOT NULL CHECK(schedule_type IN ('Lecture', 'Tutorium', 'Exercise')),
    date TEXT NOT NULL,
    start_time TEXT NOT NULL,
    end_time TEXT NOT NULL,

    -- Location
    room TEXT,
    building TEXT,
    location TEXT,

    -- Content tracking
    title TEXT,
    notes TEXT,
    slides_covered TEXT,

    -- Git integration
    git_commit_sha TEXT,
    notes_file_path TEXT,

    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE,
    UNIQUE(course_id, lecture_number)
);

CREATE INDEX idx_lectures_course ON lectures(course_id);
CREATE INDEX idx_lectures_date ON lectures(date);

-- Slides
CREATE TABLE slides (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    course_id INTEGER NOT NULL,

    -- File info
    file_name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_hash TEXT,

    -- Metadata
    slide_number INTEGER,
    title TEXT,
    page_count INTEGER,

    -- Coverage tracking
    is_covered BOOLEAN NOT NULL DEFAULT 0,
    covered_in_lecture_id INTEGER,

    -- Timestamps
    file_modified_at DATETIME,
    scanned_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE,
    FOREIGN KEY (covered_in_lecture_id) REFERENCES lectures(id) ON DELETE SET NULL,
    UNIQUE(course_id, file_name)
);

CREATE INDEX idx_slides_course ON slides(course_id);
CREATE INDEX idx_slides_covered ON slides(is_covered);
CREATE INDEX idx_slides_lecture ON slides(covered_in_lecture_id);
CREATE INDEX idx_slides_hash ON slides(file_hash);

-- Exercises
CREATE TABLE exercises (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    course_id INTEGER NOT NULL,

    -- Exercise info
    exercise_number INTEGER NOT NULL,
    title TEXT,

    -- File paths
    assignment_file_path TEXT,
    solution_directory_path TEXT,

    -- Deadlines
    due_date TEXT,
    submission_date TEXT,

    -- Grading
    points_earned REAL,
    points_total REAL,
    grade REAL,
    feedback TEXT,

    -- State
    is_submitted BOOLEAN NOT NULL DEFAULT 0,
    is_graded BOOLEAN NOT NULL DEFAULT 0,

    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE,
    UNIQUE(course_id, exercise_number)
);

CREATE INDEX idx_exercises_course ON exercises(course_id);
CREATE INDEX idx_exercises_due ON exercises(due_date);
CREATE INDEX idx_exercises_submitted ON exercises(is_submitted);

-- ==========================================
-- State & Context Tables
-- ==========================================

-- Active Course (Singleton)
CREATE TABLE active_course (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    course_id INTEGER,
    semester_id INTEGER,
    lecture_id INTEGER,

    activated_at DATETIME,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE SET NULL,
    FOREIGN KEY (semester_id) REFERENCES semesters(id) ON DELETE SET NULL,
    FOREIGN KEY (lecture_id) REFERENCES lectures(id) ON DELETE SET NULL
);

-- Initialize singleton
INSERT OR IGNORE INTO active_course (id) VALUES (1);

-- Todos
CREATE TABLE todos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    -- Associations
    course_id INTEGER,
    lecture_id INTEGER,
    exercise_id INTEGER,

    -- Task details
    title TEXT NOT NULL,
    description TEXT,
    due_date TEXT,

    -- State
    completed BOOLEAN NOT NULL DEFAULT 0,
    completed_at DATETIME,

    -- Behaviour
    auto_clear BOOLEAN NOT NULL DEFAULT 1,

    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE,
    FOREIGN KEY (lecture_id) REFERENCES lectures(id) ON DELETE SET NULL,
    FOREIGN KEY (exercise_id) REFERENCES exercises(id) ON DELETE SET NULL
);

CREATE INDEX idx_todos_course ON todos(course_id);
CREATE INDEX idx_todos_completed ON todos(completed);
CREATE INDEX idx_todos_due ON todos(due_date);

-- ==========================================
-- Platform Integration Tables
-- ==========================================

-- Platform Accounts
CREATE TABLE platform_accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    -- Platform
    platform_type TEXT NOT NULL CHECK(platform_type IN ('Moodle', 'Ilias', 'Google Drive', 'Custom')),
    platform_url TEXT NOT NULL,
    university TEXT,

    -- Credentials
    username TEXT,
    token TEXT,

    -- State
    is_active BOOLEAN NOT NULL DEFAULT 1,
    last_sync_at DATETIME,

    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(platform_type, platform_url, username)
);

CREATE INDEX idx_platform_accounts_active ON platform_accounts(is_active);

-- Platform Course Links
CREATE TABLE platform_course_links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    course_id INTEGER NOT NULL,
    platform_account_id INTEGER NOT NULL,

    -- Platform-specific IDs
    platform_course_id TEXT NOT NULL,
    platform_course_url TEXT,

    -- Sync settings
    auto_sync_exercises BOOLEAN NOT NULL DEFAULT 0,
    auto_sync_slides BOOLEAN NOT NULL DEFAULT 0,
    auto_sync_announcements BOOLEAN NOT NULL DEFAULT 0,

    -- State
    last_synced_at DATETIME,

    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE,
    FOREIGN KEY (platform_account_id) REFERENCES platform_accounts(id) ON DELETE CASCADE,
    UNIQUE(course_id, platform_account_id)
);

CREATE INDEX idx_platform_links_course ON platform_course_links(course_id);
CREATE INDEX idx_platform_links_account ON platform_course_links(platform_account_id);

-- ==========================================
-- Views
-- ==========================================

-- v_orphaned_courses
CREATE VIEW v_orphaned_courses AS
SELECT
    c.*,
    s.type || s.number as semester_code
FROM courses c
JOIN semesters s ON c.semester_id = s.id
WHERE c.exists_on_disk = 0
   OR c.toml_exists = 0;

-- v_current_gpa
CREATE VIEW v_current_gpa AS
SELECT
    d.id as degree_id,
    d.name as degree_name,
    d.type as degree_type,
    COUNT(DISTINCT c.id) as courses_completed,
    SUM(c.ects) as total_ects,
    ROUND(SUM(g.grade * c.ects) / NULLIF(SUM(c.ects), 0), 2) as gpa
FROM degrees d
LEFT JOIN course_degree_mappings cdm ON d.id = cdm.degree_id
LEFT JOIN courses c ON cdm.course_id = c.id
LEFT JOIN grades g ON c.id = g.course_id AND g.is_final = 1 AND g.passed = 1
LEFT JOIN degree_areas da ON cdm.area_id = da.id
WHERE da.counts_towards_gpa = 1
GROUP BY d.id, d.name, d.type;

-- v_degree_progress
CREATE VIEW v_degree_progress AS
SELECT
    d.id as degree_id,
    d.name as degree_name,
    da.category_name,
    da.required_ects,
    da.counts_towards_gpa,
    COALESCE(SUM(CASE
        WHEN g.passed = 1 THEN
            COALESCE(cdm.ects_override, c.ects)
        ELSE 0
    END), 0) as earned_ects,
    ROUND(
        COALESCE(SUM(CASE
            WHEN g.passed = 1 AND da.counts_towards_gpa = 1
            THEN g.grade * COALESCE(cdm.ects_override, c.ects)
            ELSE 0
        END), 0) / NULLIF(SUM(CASE
            WHEN g.passed = 1 AND da.counts_towards_gpa = 1
            THEN COALESCE(cdm.ects_override, c.ects)
            ELSE 0
        END), 0), 2
    ) as area_gpa
FROM degrees d
JOIN degree_areas da ON d.id = da.degree_id
LEFT JOIN course_degree_mappings cdm ON da.id = cdm.area_id
LEFT JOIN courses c ON cdm.course_id = c.id
LEFT JOIN grades g ON c.id = g.course_id AND g.is_final = 1
GROUP BY d.id, d.name, da.id, da.category_name, da.required_ects, da.counts_towards_gpa
ORDER BY d.id, da.display_order;

-- v_course_categories
CREATE VIEW v_course_categories AS
SELECT
    c.id as course_id,
    c.short_name,
    c.name,
    c.ects,
    d.name as degree_name,
    d.id as degree_id,
    da.category_name,
    da.id as area_id,
    CASE
        WHEN cdm.id IS NOT NULL THEN 'selected'
        ELSE 'possible'
    END as status,
    cpc.is_recommended
FROM courses c
LEFT JOIN course_possible_categories cpc ON c.id = cpc.course_id
LEFT JOIN degrees d ON cpc.degree_id = d.id
LEFT JOIN degree_areas da ON cpc.area_id = da.id
LEFT JOIN course_degree_mappings cdm
    ON c.id = cdm.course_id
    AND cpc.degree_id = cdm.degree_id
    AND cpc.area_id = cdm.area_id
WHERE c.is_archived = 0;

-- v_unmapped_courses
CREATE VIEW v_unmapped_courses AS
SELECT
    c.id,
    c.short_name,
    c.name,
    c.ects,
    COUNT(DISTINCT cpc.area_id) as possible_areas,
    GROUP_CONCAT(DISTINCT da.category_name, ', ') as area_names
FROM courses c
JOIN course_possible_categories cpc ON c.id = cpc.course_id
JOIN degree_areas da ON cpc.area_id = da.id
LEFT JOIN course_degree_mappings cdm ON c.id = cdm.course_id
WHERE c.is_archived = 0
  AND c.is_dropped = 0
  AND cdm.id IS NULL
GROUP BY c.id, c.short_name, c.name, c.ects
HAVING COUNT(DISTINCT cpc.area_id) > 0;

-- v_degree_progress_extended
CREATE VIEW v_degree_progress_extended AS
SELECT
    d.id as degree_id,
    d.name as degree_name,
    da.category_name,
    da.required_ects,
    da.counts_towards_gpa,
    COALESCE(SUM(CASE
        WHEN cdm.id IS NOT NULL AND g.passed = 1 THEN
            COALESCE(cdm.ects_override, c.ects)
        ELSE 0
    END), 0) as earned_ects,
    COUNT(DISTINCT CASE
        WHEN cdm.id IS NULL AND cpc.id IS NOT NULL THEN c.id
        ELSE NULL
    END) as unmapped_courses,
    COALESCE(SUM(CASE
        WHEN cdm.id IS NULL AND cpc.id IS NOT NULL THEN c.ects
        ELSE 0
    END), 0) as unmapped_ects,
    ROUND(
        COALESCE(SUM(CASE
            WHEN cdm.id IS NOT NULL AND g.passed = 1 AND da.counts_towards_gpa = 1
            THEN g.grade * COALESCE(cdm.ects_override, c.ects)
            ELSE 0
        END), 0) / NULLIF(SUM(CASE
            WHEN cdm.id IS NOT NULL AND g.passed = 1 AND da.counts_towards_gpa = 1
            THEN COALESCE(cdm.ects_override, c.ects)
            ELSE 0
        END), 0), 2
    ) as area_gpa
FROM degrees d
JOIN degree_areas da ON d.id = da.degree_id
LEFT JOIN course_possible_categories cpc ON da.id = cpc.area_id
LEFT JOIN courses c ON cpc.course_id = c.id AND c.is_archived = 0 AND c.is_dropped = 0
LEFT JOIN course_degree_mappings cdm
    ON c.id = cdm.course_id
    AND da.degree_id = cdm.degree_id
    AND da.id = cdm.area_id
LEFT JOIN grades g ON c.id = g.course_id AND g.is_final = 1
GROUP BY d.id, d.name, da.id, da.category_name, da.required_ects, da.counts_towards_gpa
ORDER BY d.id, da.display_order;
