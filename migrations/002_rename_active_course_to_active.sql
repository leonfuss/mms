-- Rename active_course table to active and add semester_id

-- Drop the old table
DROP TABLE IF EXISTS active_course;

-- Create new active table with both semester_id and course_id
CREATE TABLE IF NOT EXISTS active (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    semester_id INTEGER,
    course_id INTEGER,
    lecture_id INTEGER,
    activated_at DATETIME,
    FOREIGN KEY (semester_id) REFERENCES semesters(id) ON DELETE SET NULL,
    FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE SET NULL,
    FOREIGN KEY (lecture_id) REFERENCES lectures(id) ON DELETE SET NULL
);

-- Initialize active singleton
INSERT OR IGNORE INTO active (id) VALUES (1);
