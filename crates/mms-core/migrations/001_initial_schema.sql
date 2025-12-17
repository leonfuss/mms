-- Initial schema for MMS (My Study Management System)

-- Semesters table
CREATE TABLE IF NOT EXISTS semesters (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    type TEXT NOT NULL CHECK(type IN ('Bachelor', 'Master')),
    number INTEGER NOT NULL,
    is_current BOOLEAN NOT NULL DEFAULT 0,
    default_location TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(type, number)
);

-- Courses table
CREATE TABLE IF NOT EXISTS courses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    semester_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    short_name TEXT NOT NULL,
    category TEXT NOT NULL,
    ects INTEGER NOT NULL,
    lecturer TEXT,
    learning_platform_url TEXT,
    location TEXT,
    grade REAL,
    counts_towards_average BOOLEAN NOT NULL DEFAULT 1,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (semester_id) REFERENCES semesters(id) ON DELETE CASCADE
);

-- Course schedules (recurring weekly)
CREATE TABLE IF NOT EXISTS course_schedules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    course_id INTEGER NOT NULL,
    schedule_type TEXT NOT NULL CHECK(schedule_type IN ('Lecture', 'Tutorium', 'Exercise')),
    day_of_week INTEGER NOT NULL CHECK(day_of_week BETWEEN 0 AND 6),
    start_time TEXT NOT NULL,
    end_time TEXT NOT NULL,
    start_date TEXT NOT NULL,
    end_date TEXT NOT NULL,
    room TEXT,
    location TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE
);

-- Course events (one-time, cancellations, overrides)
CREATE TABLE IF NOT EXISTS course_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    course_id INTEGER NOT NULL,
    course_schedule_id INTEGER,
    schedule_type TEXT NOT NULL CHECK(schedule_type IN ('Lecture', 'Tutorium', 'Exercise')),
    event_type TEXT NOT NULL CHECK(event_type IN ('OneTime', 'Makeup', 'Special', 'Override', 'Cancelled')),
    date TEXT NOT NULL,
    start_time TEXT,
    end_time TEXT,
    room TEXT,
    location TEXT,
    description TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE,
    FOREIGN KEY (course_schedule_id) REFERENCES course_schedules(id) ON DELETE SET NULL
);

-- Exams table
CREATE TABLE IF NOT EXISTS exams (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    course_id INTEGER NOT NULL,
    exam_type TEXT NOT NULL CHECK(exam_type IN ('Written', 'Oral', 'Project', 'Presentation', 'Other')),
    date TEXT NOT NULL,
    start_time TEXT,
    end_time TEXT,
    room TEXT,
    location TEXT,
    description TEXT,
    result REAL,
    passed BOOLEAN,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE
);

-- Holidays table
CREATE TABLE IF NOT EXISTS holidays (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    semester_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    start_date TEXT NOT NULL,
    end_date TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (semester_id) REFERENCES semesters(id) ON DELETE CASCADE
);

-- Holiday exceptions (courses that continue during holidays)
CREATE TABLE IF NOT EXISTS holiday_exceptions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    holiday_id INTEGER NOT NULL,
    course_id INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (holiday_id) REFERENCES holidays(id) ON DELETE CASCADE,
    FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE,
    UNIQUE(holiday_id, course_id)
);

-- Lectures table (history of attended lectures for git commits)
CREATE TABLE IF NOT EXISTS lectures (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    course_id INTEGER NOT NULL,
    schedule_type TEXT NOT NULL CHECK(schedule_type IN ('Lecture', 'Tutorium', 'Exercise')),
    lecture_number INTEGER NOT NULL,
    date TEXT NOT NULL,
    start_time TEXT NOT NULL,
    end_time TEXT NOT NULL,
    room TEXT,
    location TEXT,
    notes TEXT,
    git_commit_sha TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE
);

-- Todos table
CREATE TABLE IF NOT EXISTS todos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    course_id INTEGER,
    lecture_id INTEGER,
    title TEXT NOT NULL,
    description TEXT,
    due_date TEXT,
    completed BOOLEAN NOT NULL DEFAULT 0,
    auto_clear BOOLEAN NOT NULL DEFAULT 1,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE,
    FOREIGN KEY (lecture_id) REFERENCES lectures(id) ON DELETE SET NULL
);

-- Active course table (singleton for current active course)
CREATE TABLE IF NOT EXISTS active_course (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    course_id INTEGER,
    lecture_id INTEGER,
    activated_at DATETIME,
    FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE SET NULL,
    FOREIGN KEY (lecture_id) REFERENCES lectures(id) ON DELETE SET NULL
);

-- Initialize active_course singleton
INSERT OR IGNORE INTO active_course (id) VALUES (1);

-- Category requirements table
CREATE TABLE IF NOT EXISTS category_requirements (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    semester_id INTEGER NOT NULL,
    category TEXT NOT NULL,
    required_ects INTEGER NOT NULL,
    counts_towards_average BOOLEAN NOT NULL DEFAULT 1,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (semester_id) REFERENCES semesters(id) ON DELETE CASCADE,
    UNIQUE(semester_id, category)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_courses_semester ON courses(semester_id);
CREATE INDEX IF NOT EXISTS idx_schedules_course ON course_schedules(course_id);
CREATE INDEX IF NOT EXISTS idx_events_course ON course_events(course_id);
CREATE INDEX IF NOT EXISTS idx_events_date ON course_events(date);
CREATE INDEX IF NOT EXISTS idx_exams_course ON exams(course_id);
CREATE INDEX IF NOT EXISTS idx_holidays_semester ON holidays(semester_id);
CREATE INDEX IF NOT EXISTS idx_lectures_course ON lectures(course_id);
CREATE INDEX IF NOT EXISTS idx_todos_course ON todos(course_id);
CREATE INDEX IF NOT EXISTS idx_todos_lecture ON todos(lecture_id);
CREATE INDEX IF NOT EXISTS idx_category_reqs_semester ON category_requirements(semester_id);
