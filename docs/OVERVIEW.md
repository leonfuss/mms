# MMS (My Management System) - Refined CLI Specification

**Version:** 2.0
**Last Updated:** December 2024
**Target Platform:** macOS (M1), Linux support planned

---

## Table of Contents

1. [Overview](#overview)
2. [Core Concepts](#core-concepts)
3. [Context Resolution](#context-resolution)
4. [Installation & Initialization](#installation--initialization)
5. [Configuration Management](#configuration-management)
6. [Degree Management](#degree-management)
7. [Semester Management](#semester-management)
8. [Course Management](#course-management)
9. [Grade Management](#grade-management)
10. [Exam Attempts & Retakes](#exam-attempts--retakes)
11. [Exercise Management](#exercise-management)
12. [Lecture Notes System](#lecture-notes-system)
13. [Schedule & Timetable](#schedule--timetable)
14. [Progress Tracking](#progress-tracking)
15. [Simulation & Planning](#simulation--planning)
16. [Platform Integration](#platform-integration)
17. [Reports & Export](#reports--export)
18. [Contact Management](#contact-management)
19. [Templates](#templates)
20. [Archive & History](#archive--history)
21. [Import External Structure](#import-external-structure)
22. [Shell Integration](#shell-integration)

---

## Overview

MMS is a command-line tool for managing university studies, designed for students who want:
- Structured organisation of courses, grades, and materials
- Automatic tracking of ECTS progress and GPA
- Support for multiple simultaneous degree programmes
- Integrated lecture note-taking with Typst
- Smart course switching based on timetable
- Grade simulation and planning tools
- Flexible handling of mixed/abroad semesters

**Command Structure:** `mms <command> [subcommand] [options] [arguments]`

---

## Core Concepts

### Directory Structure

```
~/Studies/                           # Root study directory
├── b1/                              # Bachelor Semester 1
│   ├── .semester.toml              # Semester metadata
│   └── cs101/                      # Course: CS101
│       ├── .course.toml            # Course metadata
│       ├── slides/                 # Lecture slides
│       │   ├── 01.pdf
│       │   ├── 02.pdf
│       │   └── 03.pdf
│       ├── notes/                  # Lecture notes (Typst)
│       │   ├── main.typ
│       │   ├── template.typ
│       │   ├── config.typ
│       │   ├── lectures/
│       │   │   ├── lecture_01.typ
│       │   │   └── lecture_02.typ
│       │   └── build/
│       └── exercises/              # Exercise submissions
│           ├── assignment01/
│           │   └── solution.typ
│           └── assignment02/
│               └── solution.typ
├── b2/
├── b3/
├── b7/                             # Bachelor Semester 7 (overlap case)
├── m1/                             # Master Semester 1 (can overlap with b7)
└── .archive/                       # Archived/dropped courses
    ├── b2/
    │   └── dropped_course/
    └── b3/
        └── failed_course/

~/.config/mms/                      # Configuration directory
├── config.toml                     # User configuration
├── data.db                         # SQLite database
├── schemes/                        # Grading schemes (synced)
│   ├── german.toml
│   ├── swiss.toml
│   ├── italian.toml
│   └── conversions.toml
├── requirements/                   # Degree requirements (synced)
│   ├── tum-cs-bachelor.toml
│   └── tum-cs-master.toml
├── catalogs/                       # Course catalogs (synced)
│   └── tum-courses.toml
└── templates/                      # Document templates
    ├── typst-exercise.typ
    └── latex-report.tex

~/cc -> ~/Studies/b3/cs101          # Symlink to current course
~/cs -> ~/Studies/b3                # Symlink to current semester
```

### Data Model

- **Degree Programme**: Bachelor/Master in specific subject (e.g., "Bachelor Computer Science")
  - Multiple simultaneous degrees supported
  - Courses can count towards multiple degrees in different areas
- **Semester**: Period of study (b1, b2, m1, etc.)
  - Status: planned, active, completed
  - Can be mixed (courses from multiple universities)
  - Each semester has a default university
- **Course**: Individual class taken in a semester
  - Can be external (imported without standard structure)
  - Can count towards multiple degrees
- **ECTS Area**: Category for degree requirements (Core CS, Math, Electives, etc.)
  - Some areas count towards ECTS but not GPA
- **Grade**: Final grade for a course (may have multiple components)
  - Follows university-specific grading scheme
- **Grade Component**: Part of final grade (midterm, final, project, etc.)
- **Bonus System**: Configurable per course (linear/threshold, pre/post pass)
- **Exam Attempt**: Individual exam sitting (can have multiple per course)
  - Default policy: first passing attempt counts (configurable per university)
- **Exercise**: Assignment/homework with deadline
- **Lecture**: Individual class session with notes and slides

---

## Context Resolution

**Priority Order for Context Detection:**

1. **Explicit flags**: `--course <name>`, `--semester <code>`
2. **Current working directory (pwd)**: First course found in path (searches upward)
3. **Active course/semester**: Via `~/cc` or `~/cs` symlinks
4. **Fail with error**: Prompt user to specify context

### Examples

```bash
# Explicit context (always works)
mms grade add 2.3 --course cs101 --semester b3
mms grade add 2.3 -c cs101 -s b3

# PWD context (searches upward for first course)
cd ~/Studies/b3/cs101/
mms grade add 2.3                    # ✓ Uses cs101 from pwd

cd ~/Studies/b3/cs101/exercises/assignment01/
mms ex submit                        # ✓ Finds cs101 by searching parent dirs
mms grade add 2.3                    # ✓ Uses cs101 (first course in path)

# PWD takes priority over active course
mms course set math201               # Active: math201
cd ~/Studies/b3/cs101/
mms grade show                       # ✓ Shows cs101 grades (not math201)

# Active context (when not in Studies directory)
cd ~/Documents/
mms grade show                       # ✓ Uses course from ~/cc symlink

# Fail case
mms course set ""                    # Clear active course
cd ~/Documents/
mms grade show                       # ✗ Error: No course context found
# Output: "Error: No course specified. Use --course <name> or navigate to course directory"
```

---

## Installation & Initialization

### Installation

```bash
# Install via Homebrew (planned)
brew install mms

# Or via Cargo
cargo install mms

# Or download binary
curl -L https://github.com/user/mms/releases/latest/download/mms-macos -o mms
chmod +x mms
mv mms /usr/local/bin/
```

### First-Time Setup

```bash
# Initialize MMS
mms init

# Interactive setup wizard
# Prompts for:
# - Studies root directory (default: ~/Studies)
# - Student name
# - Student ID/matriculation number
# - Default editor (zed, nvim, code, etc.)
# - Default PDF viewer (skim, preview, evince, etc.)
# - Create directory structure? [Y/n]

# Example interaction:
$ mms init
Welcome to MMS! Let's set up your study environment.

Studies root directory [~/Studies]:
Student name: John Doe
Student ID: 12345678
Default editor (zed/nvim/code) [zed]:
Default PDF viewer (skim/preview) [skim]:
Create directory structure? [Y/n]: y

✓ Created ~/Studies/
✓ Created ~/.config/mms/
✓ Initialised database
✓ Created default templates

Next steps:
1. Set up your degree: mms degree create
2. Import configuration: mms config import --code <code>
3. Create your first semester: mms semester create b1
```

---

## Configuration Management

### Configuration Structure

Configuration is stored in `~/.config/mms/config.toml`:

```toml
[general]
studies_root = "~/Studies"
student_name = "John Doe"
student_id = "12345678"
default_editor = "zed"
default_pdf_viewer = "skim"

[grading]
default_scheme = "german"

[notes]
auto_watch = true
auto_open_pdf = true
template = "default-lecture"

[schedule]
auto_switch = true
switch_window_minutes = 10
notify = true

[sync]
auto_fetch = false
platforms = ["moodle", "ilias"]
```

### Configuration Commands

```bash
# View all configuration
mms config show

# View specific section
mms config show general
mms config show grading
mms config show notes

# Set individual values
mms config set general.default_editor nvim
mms config set notes.auto_watch false
mms config set schedule.notify true

# Edit configuration file directly
mms config edit
# Opens ~/.config/mms/config.toml in $EDITOR

# Reset to defaults
mms config reset
mms config reset grading  # Reset only grading section
```

### Importing Shared Configurations

Shared configurations allow students at the same university to use standardised:
- Degree requirements (ECTS areas, mandatory courses)
- Grading schemes and conversion tables
- Course catalogues (available courses with details)
- University-specific policies

```bash
# Import via code
mms config import --code "TUM-CS-B-2023"

# Import via URL
mms config import --url "https://mms-share.io/configs/tum-cs-bachelor-2023.json"

# Import from file
mms config import --file tum-config.json

# What gets imported:
# - Degree structure (areas, ECTS requirements)
# - Grading scheme for university
# - University policies (max attempts, retake rules)
# - Course catalogue (optional)
# - Conversion tables (for abroad semesters)

# View imported configs
mms config imported

# Update imported config
mms config update --code "TUM-CS-B-2023"
# Fetches latest version, asks what to update
```

### Exporting/Sharing Configurations

```bash
# Export your configuration for sharing
mms config export --public

# Output:
# Created shareable package: tum-cs-bachelor-2023.json
# Share code: TUM-CS-B-2023-a7f3d
# Share URL: https://mms-share.io/abc123
#
# Others can import with:
#   mms config import --code TUM-CS-B-2023-a7f3d

# Export with specific components
mms config export --include areas,schemes,catalog
mms config export --exclude personal  # Exclude grades, personal data
```

### University Policies

University-specific policies are stored and inherited by all courses/semesters from that university unless overridden.

```bash
# Configure university-specific policies
mms config set-policy --university TUM \
    --max-exam-attempts 3 \
    --active-attempt "first-passing" \
    --require-grade-for-completion true \
    --warn-final-attempt true

mms config set-policy --university "ETH Zürich" \
    --max-exam-attempts unlimited \
    --active-attempt "first-passing" \
    --require-grade-for-completion true

mms config set-policy --university "Università di Milano" \
    --max-exam-attempts unlimited \
    --active-attempt "best" \
    --require-grade-for-completion false

# View policies
mms config show-policy
mms config show-policy TUM

# Override policy for specific course
mms course set-policy cs101 --max-exam-attempts 5
mms course set-policy cs101 --warn-final-attempt false
```

### Grading Schemes

Each university has a default grading scheme that defines valid grades and conversions.

```bash
# Add grading scheme
mms scheme add german \
    --scale "1.0,1.3,1.7,2.0,2.3,2.7,3.0,3.3,3.7,4.0,5.0" \
    --pass-threshold 4.0 \
    --best 1.0 \
    --worst 5.0

mms scheme add italian \
    --scale "0-30" \
    --pass-threshold 18 \
    --best 30+ \
    --worst 0 \
    --note "30+ indicates 30 with distinction"

mms scheme add swiss \
    --scale "1.0-6.0" \
    --pass-threshold 4.0 \
    --best 6.0 \
    --worst 1.0

# View schemes
mms scheme list
mms scheme show german

# Set conversion table
mms scheme convert --from italian --to german \
    --table "30+=1.0,30=1.3,29=1.5,28=1.7,..."

# Test conversion
mms scheme convert 28 --from italian --to german
# Output: "28 (Italian) → 1.7 (German)"
```

---

## Degree Management

### Creating Degree Programmes

MMS supports multiple simultaneous degree programmes. Courses can count towards multiple degrees in different areas.

```bash
# Create degree programme
mms degree create bachelor "Computer Science" \
    --university "TUM" \
    --total-ects 180 \
    --start-date "2021-10-01"

# Create second concurrent degree (e.g., double major)
mms degree create bachelor "Mathematics" \
    --university "TUM" \
    --total-ects 180 \
    --start-date "2021-10-01"

# Create overlapping Master
mms degree create master "Computer Science" \
    --university "TUM" \
    --total-ects 120 \
    --start-date "2024-10-01"

# Add ECTS requirements by area
mms degree add-area bachelor-cs "Core CS" --ects 60
mms degree add-area bachelor-cs "Mathematics" --ects 30
mms degree add-area bachelor-cs "Practical" --ects 30
mms degree add-area bachelor-cs "Electives" --ects 30
mms degree add-area bachelor-cs "Seminar" --ects 15 --no-gpa
mms degree add-area bachelor-cs "Thesis" --ects 15

# Areas marked with --no-gpa count towards ECTS but not towards degree GPA
# Grades are shown in reports but marked as non-GPA

# Or import from shared configuration
mms config import --code "TUM-CS-B-2023"
# This imports:
# - Degree structure
# - ECTS requirements
# - Grading scheme
# - University policies
# - Course catalogue
```

### Managing Degrees

```bash
# List degrees
mms degree list
# Output:
# Active Degrees:
# [1] Bachelor Computer Science (TUM)
#     Started: Oct 2021, Progress: 98/180 ECTS (54%)
# [2] Bachelor Mathematics (TUM)
#     Started: Oct 2021, Progress: 45/180 ECTS (25%)
# [3] Master Computer Science (TUM)
#     Started: Oct 2024, Progress: 8/120 ECTS (7%)

# Show degree details
mms degree show bachelor-cs
# Output:
# Bachelor Computer Science (TUM)
# Started: October 2021
# Expected completion: September 2024
# Total ECTS: 180
#
# ECTS Requirements:
# - Core CS:      60 ECTS (GPA: Yes)
# - Mathematics:  30 ECTS (GPA: Yes)
# - Practical:    30 ECTS (GPA: Yes)
# - Electives:    30 ECTS (GPA: Yes)
# - Seminar:      15 ECTS (GPA: No)
# - Thesis:       15 ECTS (GPA: Yes)

# Edit degree
mms degree edit bachelor-cs
mms degree update bachelor-cs --total-ects 180

# Map course to multiple degrees
mms course map cs101 --degree bachelor-cs --area "Core CS"
mms course map cs101 --degree bachelor-math --area "Electives"
# Now cs101 counts 8 ECTS towards both degrees in different areas
```

---

## Semester Management

### Semester Statuses

Semesters have the following statuses:
- **planned**: Future semester, not yet started
- **active**: Currently ongoing semester (only one active per degree level)
- **completed**: Past semester, finished

Transition rules:
- Future semesters are automatically **planned**
- Current semester (by date or manual activation) is **active**
- Past semesters are automatically **completed**
- Exception: b7 and m1 can both be active simultaneously (different degree levels)

### Creating Semesters

All semesters support mixed universities by default. Each semester has a default university, and individual courses can override this.

```bash
# Basic semester creation
mms semester create b1              # Bachelor Semester 1
mms semester create m2              # Master Semester 2

# With dates and default university
mms semester create b3 \
    --start "2024-10-01" \
    --end "2025-03-31" \
    --university TUM

# Abroad semester (different default university)
mms semester create b4 \
    --start "2025-04-01" \
    --end "2025-09-30" \
    --university "ETH Zürich"

# Mixed semester (some courses home, some abroad)
# No special setup needed - just set default and override per course
mms semester create b5 --university TUM
# Then when adding courses:
mms course add cs101              # Uses default (TUM)
mms course add math201 --university "Università di Milano"  # Override
```

### Semester Operations

```bash
# List semesters
mms semester list
# Output:
# Bachelor Semesters:
# ✓ b1  (2021-10 to 2022-03)  TUM           [completed]
# ✓ b2  (2022-04 to 2022-09)  TUM           [completed]
# ● b3  (2024-10 to 2025-03)  TUM           [active]
#   b4  (2025-04 to 2025-09)  ETH Zürich    [planned]
#   b7  (2026-10 to 2027-03)  TUM           [planned]
#
# Master Semesters:
#   m1  (2026-10 to 2027-03)  TUM           [planned]

# Show all including archived
mms semester list --all
mms semester list --archived

# Set active semester
mms semester set b3
# Output:
# ✓ Active semester: b3
# ✓ Symlink ~/cs -> ~/Studies/b3

# Semester information
mms semester info
mms semester info b3
# Output:
# Semester: b3 (Bachelor, Semester 3)
# Period: 2024-10-01 to 2025-03-31
# Default University: TUM
# Status: Active
# Courses: 5 (4 enrolled, 1 completed)
#   - 4 at TUM
#   - 1 at ETH Zürich (math201)
# Total ECTS: 28
# Average grade: 1.8 (2 courses graded)

# Complete semester manually
mms semester complete b3
# Sets status to completed, validates all courses

# Delete semester (with confirmation)
mms semester delete b4 --confirm
# Permanently removes semester and all courses
```

---

## Course Management

### Creating/Adding Courses

Courses inherit the default university from their semester unless explicitly overridden.

```bash
# Add course (uses semester's default university)
mms course add cs101

# Add to specific semester
mms course add cs101 --semester b3

# Override university (for mixed semester)
mms course add math201 --university "Università di Milano"

# Create manual course (not in catalogue)
mms course create myproject \
    --name "Special Research Project" \
    --ects 8 \
    --area "Electives" \
    --grading single

# With full details
mms course create cs101 \
    --name "Introduction to Algorithms" \
    --ects 8 \
    --area "Core CS" \
    --grading multiple \
    --lecturer "Prof. Dr. Schmidt" \
    --lecturer-email "schmidt@tum.de" \
    --tutor "Anna Müller" \
    --tutor-email "anna@tum.de"

# Map to multiple degrees
mms course create shared_course \
    --map bachelor-cs:"Core CS" \
    --map bachelor-math:"Electives"
```

### External Courses

External courses are imported from existing structures without standard MMS organisation.

```bash
# Import external course
mms course import ~/OldStructure/Algorithms \
    --name "Algorithms" \
    --code cs101 \
    --ects 8 \
    --grade 1.7 \
    --scheme german \
    --semester b1 \
    --university TUM \
    --area "Core CS"

# Mark as external (warns about missing structure)
# External courses:
# - Show warning when accessed
# - Don't assume standard directories
# - Only track: name, ECTS, grade, scheme
# - Linked to original directory

# Convert external to standard
mms course convert cs101
# Creates standard structure, imports files if possible
```

### Listing Courses

```bash
# Current semester courses
mms course list

# All courses
mms course list --all

# Filter by status
mms course list --status enrolled
mms course list --status completed
mms course list --status dropped

# Filter by area
mms course list --area "Core CS"

# Filter by semester
mms course list --semester b3

# Filter by university
mms course list --university TUM
mms course list --university "ETH Zürich"

# Show external courses
mms course list --external

# Short format (just names, for scripting)
mms course list --short
# Output:
# cs101
# math201
# phys101
```

### Course Operations

```bash
# Set active course
mms course set cs101
# Output:
# ✓ Active course: cs101
# ✓ Symlink ~/cc -> ~/Studies/b3/cs101

# Clear active course
mms course set ""

# Show course info
mms course info
mms course info cs101
# Output:
# CS101 - Introduction to Algorithms
# Semester: b3 (2024-10 to 2025-03)
# University: TUM (Grading: German 1.0-5.0)
# ECTS: 8 (Area: Core CS)
# Status: Enrolled
# Degrees:
#   - Bachelor CS (Core CS)
#   - Bachelor Math (Electives)
# Lecturer: Prof. Dr. Schmidt (schmidt@tum.de)
# Tutor: Anna Müller (anna@tum.de)
#
# Grading: Multiple components
# - Midterm: 40% (not graded)
# - Final: 60% (not graded)
# Current grade: --
#
# Exercises: 3 total, 2 submitted, 1 upcoming
# Next deadline: assignment03 (2024-12-20)
#
# Platform: Moodle
# URL: https://moodle.tum.de/course/123
# Last sync: 2024-12-15 10:30

# Edit course
mms course edit cs101
# Opens course details for editing

# Update specific field
mms course update cs101 --tutor "New Tutor Name"
mms course update cs101 --ects 6
mms course update cs101 --area "Electives"
mms course update cs101 --university "ETH Zürich"

# Platform integration
mms course link moodle "https://moodle.tum.de/course/123"
mms course link ilias "https://ilias.ethz.ch/goto.php?target=crs_456"
mms course link gdrive "https://drive.google.com/drive/folders/abc123"

# Remove platform link
mms course unlink moodle

# Change status
mms course complete cs101        # Mark as completed (requires final grade per university policy)
mms course drop cs101            # Mark as dropped, move to archive

# Delete course (with confirmation)
mms course delete cs101 --confirm
# Permanently removes course and all data
```

---

## Grade Management

### Grade Setup

Courses can have different grading structures:

1. **Single grade**: One final grade
2. **Multiple components**: Weighted components (midterm, final, project, etc.)
3. **Pass/Fail**: No numeric grade (depends on university policy)

```bash
# Set up grade components
mms grade setup cs101 --components "midterm:40,final:60"
mms grade setup phys101 --components "exam:70,project:20,attendance:10"

# Add component after setup
mms grade component add cs101 --name "project" --weight 20
# Note: This requires rebalancing other components!

# Add bonus component
mms grade component add cs101 --name "bonus" --type bonus

# Configure bonus system
mms grade bonus cs101 \
    --max-points 20 \
    --max-bonus-percent 20 \
    --function linear \
    --apply-after-pass \
    --cap 1.0

# Bonus configuration explained:
# - Collect max 20 points in assignments
# - Get max 20% bonus on exam grade
# - Bonus is linear to points earned
# - Only applies if you pass the exam
# - Final grade capped at 1.0 (German best grade)

# Alternative bonus system (before pass check)
mms grade bonus math201 \
    --max-points 10 \
    --max-bonus-percent 10 \
    --function threshold \
    --thresholds "5:5,10:10" \
    --apply-before-pass

# View component structure
mms grade components
mms grade components cs101
# Output:
# CS101 Grade Components:
# - Midterm:    40% (not graded)
# - Final:      60% (not graded)
# - Bonus:      max 20% on final grade
#   • Max points: 20
#   • Function: linear
#   • Applied after passing
#   • Grade cap: 1.0
#
# Total weight: 100%

# Edit component
mms grade component edit cs101 midterm --weight 35
mms grade component edit cs101 midterm --name "Midterm Exam"

# Remove component (requires rebalancing)
mms grade component remove cs101 bonus --rebalance
```

### Recording Grades

```bash
# Single-grade course
mms grade add cs101 2.3

# Multiple components (as you receive them)
mms grade add cs101 --midterm 85 --points       # 85 points
mms grade add cs101 --midterm 1.7               # Or as grade
mms grade add cs101 --final 92 --points         # 92 points

# For points: specify points and percentage to max
mms grade add cs101 --midterm 85 --max-points 100
# System calculates: 85/100 = 85% → converts to grade via scheme

# Bonus points
mms grade add cs101 --bonus 15
# System applies bonus per configured system:
# - 15/20 points = 75% of max bonus
# - 75% of 20% = 15% bonus on final grade
# - If final grade is 2.0 and passed: 2.0 - 15% = 1.7
# - Capped at 1.0

# Pass/Fail (if university allows)
mms grade add seminar101 --pass

# View current grades
mms grade show                           # Current semester
mms grade show cs101                     # Specific course
mms grade show --all                     # All courses
mms grade show --semester b3

# Update grade
mms grade update cs101 --final 1.3
mms grade update cs101 --midterm 90 --points

# Delete component grade
mms grade delete cs101 --midterm

# Delete all grades for course
mms grade delete cs101 --all --confirm
```

### Grade Calculation with Points

When working with points-based systems (common in Italy, Switzerland):

```bash
# Italian system example (0-30, pass at 18)
mms course create italian_course \
    --university "Università di Milano" \
    --grading single

# Record grade with Italian scheme
mms grade add italian_course 28
# System recognises Italian scheme from university

# View with scheme info
mms grade show italian_course
# Output:
# Italian Course
# University: Università di Milano (Italian: 0-30, 30+)
# Grade: 28/30 (Passed)

# German system example (1.0-5.0, pass at 4.0)
mms course create german_course --university TUM
mms grade add german_course 1.7
# Output:
# German Course
# University: TUM (German: 1.0-5.0)
# Grade: 1.7 (Passed)
```

### Abroad Grades & Conversion

```bash
# Course at abroad university automatically uses that university's scheme
mms course add math201 --university "Università di Milano"
mms grade add math201 28

# View grade with conversion
mms grade show math201 --convert german
# Output:
# MATH201 - Linear Algebra
# Original: 28 (Italian scheme)
# Converted: 1.7 (German scheme)
# Conversion: Standard table

# Multiple scheme display
mms grade show --all --convert german
# Shows all grades with conversions to German scheme
```

### Grade Transfer Control

For abroad/non-GPA courses, control which grades count towards your degree:

```bash
# Course that counts for ECTS but not GPA (e.g., seminar)
# Already configured in degree area with --no-gpa flag
mms grade show seminar101
# Output:
# Seminar 101
# Grade: 1.0
# ECTS: 6 (counts towards degree)
# GPA: No (area marked as non-GPA)

# View all with transfer status
mms grade show --all
# Output:
# CS101:    Grade 2.3  [ECTS: ✓] [GPA: ✓]
# MATH201:  Grade 1.7  [ECTS: ✓] [GPA: ✓]
# SEM101:   Grade 1.0  [ECTS: ✓] [GPA: ✗] (non-GPA area)
# ART101:   Pass       [ECTS: ✓] [GPA: ✗] (pass/fail)

# In progress view shows with special notation
mms progress
# Output:
# Core CS:        45/60 ECTS (75%)  GPA: 1.6
# Seminar:        6/15 ECTS (40%)   GPA: -- (non-GPA)
#                 (6 ECTS)           Grade: 1.0
```

---

## Exam Attempts & Retakes

Multiple exam attempts can be recorded for the same course. The active grade is determined by university policy (default: first passing attempt).

### Recording Exam Attempts

```bash
# First attempt
mms exam add cs101 --grade 5.0 --date "2024-07-15"
# Output:
# ✓ Exam attempt 1 recorded: 5.0 (Failed)
# ✓ Retake allowed (2 attempts remaining)
# Policy: First passing attempt will be active

# Second attempt (same semester)
mms exam add cs101 --grade 2.3 --date "2024-09-20"
# Output:
# ✓ Exam attempt 2 recorded: 2.3 (Passed)
# ✓ This grade is now active (first passing attempt)

# Third attempt (trying to improve - if allowed by university)
mms exam add cs101 --grade 1.7 --date "2024-10-15"
# Output:
# ⚠ Warning: You have already passed with 2.3
# Per university policy (TUM), first passing attempt (2.3) remains active
# New attempt (1.7) recorded but not active
# Use 'mms exam set-active cs101 --attempt 3' to change

# If policy prevents retake:
# Error: Cannot retake exam
# - Current grade: 2.3 (Passed)
# - Policy (TUM): Cannot retake after passing
# - Use --force to override (requires justification)

# Force retake (if you have special permission)
mms exam add cs101 --grade 1.7 --force --note "Approved by Prüfungsamt"
```

### Viewing Exam History

```bash
# View all attempts for a course
mms exam history cs101
# Output:
# CS101 - Introduction to Algorithms
# University: TUM
# Policy: First passing attempt active, max 3 attempts
#
# Attempt 1 (2024-07-15): 5.0 (Failed)
# Attempt 2 (2024-09-20): 2.3 (Passed) ✓ [active]
# Attempt 3 (2024-10-15): 1.7 (Passed)
#
# Active grade: 2.3 (Attempt 2 - first passing)
# Attempts used: 3/3 (no more attempts)

# List all exam attempts across courses
mms exam list
mms exam list --failed           # Only failed attempts
mms exam list --retakeable       # Courses that can be retaken
mms exam list --final-attempt    # Courses on last attempt

# Check exam status
mms exam status cs101
# Output:
# Current: 2.3 (Attempt 2/3) [active]
# Best attempt: 1.7 (Attempt 3)
# Policy: First passing attempt
# Next retake available: No (already passed, TUM policy)
# Warning: This is your final attempt slot
```

### Managing Active Grade

By default, the active grade follows university policy. You can override per course if needed.

```bash
# Set specific attempt as active (override policy)
mms exam set-active cs101 --attempt 3
# Prompts:
# "Change active grade from 2.3 (Attempt 2) to 1.7 (Attempt 3)? [y/N]"
# "This overrides university policy (first passing attempt)"
# "⚠ Warning: This will change your GPA!"
# "Reason for override: " _

# Set best attempt as active (override policy)
mms exam set-active cs101 --best
# Automatically selects attempt with best grade (1.7)
# Prompts for override reason

# Reset to policy default
mms exam set-active cs101 --policy
# Resets to first passing attempt (2.3)

# View which attempt is active
mms grade show cs101
# Shows: "Grade: 2.3 (Attempt 2 of 3) [active - first passing]"
```

### Retaking Entire Course

When retaking a course in a different semester:

```bash
# Same semester retake - just new exam attempt
mms exam add cs101 --attempt 2

# Different semester retake - need to decide on course handling
# Option 1: Create new course entry
mms course add cs101 --semester b4 --retake-of b3/cs101

# Option 2: Restore from archive and create links
mms archive restore cs101 --semester b4 --link-to-original
# Creates: ~/Studies/b4/cs101/ (new structure)
# Symlinks:
#   ~/Studies/b4/cs101/notes_old -> ~/Studies/.archive/b3/cs101/notes
#   ~/Studies/b4/cs101/exercises_old -> ~/Studies/.archive/b3/cs101/exercises

# User prompt on course retake:
# "CS101 was previously taken in b3. Options:
#  [1] Start fresh (new directories)
#  [2] Link to old materials (symbolic links to archive)
#  [3] Copy old materials
# Choice [1]: " 2
```

### University-Specific Warnings

```bash
# Warning on approaching max attempts (if enabled in university policy)
mms exam add cs101 --grade 5.0
# Output:
# ✓ Exam attempt 2 recorded: 5.0 (Failed)
# ⚠ WARNING: This is your second-to-last attempt!
# ⚠ You have 1 attempt remaining
# Next exam date: 2025-03-15 (if available)

# Can be disabled per course
mms course set-policy cs101 --warn-final-attempt false
```

---

## Exercise Management

### Creating Exercises

Exercises follow a standard naming scheme: `assignment01`, `assignment02`, etc.

```bash
# Create new exercise (auto-increments number)
mms ex new
# Creates: ~/cc/exercises/assignment01/
# Generates: assignment01/solution.typ from template

# With details
mms ex new --number 01 --title "Dynamic Programming" --due "2024-12-20"
mms ex new 02 --due "2024-12-27"

# Specify template
mms ex new --template report
mms ex new --template code

# For specific course
mms ex new --course cs101 --number 03

# Custom naming (if course uses different scheme)
mms ex new --name "homework_1" --scheme "homework_{n}"
# Future exercises will follow: homework_1, homework_2, etc.
```

### Exercise Fetching and Customisation

```bash
# Fetch exercises from platform (auto-creates directories)
mms ex fetch
# Output:
# Fetching from Moodle...
# Found 2 new exercises:
#
# Creating assignment03/
# ✓ Downloaded assignment03.pdf → assignment03/
# ✓ Created assignment03/solution.typ
#
# Creating assignment04/
# ✓ Downloaded assignment04.pdf → assignment04/
# ✓ Created assignment04/solution.typ

# Fetch with custom naming detection
mms ex fetch --detect-naming
# Analyses filenames: "homework1.pdf", "homework2.pdf"
# Prompts: "Detected naming scheme: homework{n}. Use this? [Y/n]"
# Updates course config to use this scheme

# Set custom naming for course
mms course config cs101 --exercise-naming "ex{n:02d}"
# Future exercises: ex01, ex02, ex03, etc.

mms course config math201 --exercise-naming "sheet_{n}"
# Future exercises: sheet_1, sheet_2, etc.

# Configure fetch behaviour per course
mms course config cs101 --exercise-auto-create true
mms course config cs101 --exercise-fetch-solutions true
mms course config cs101 --exercise-template custom-algo

# Custom init script for course-specific setup
mms course config cs101 --exercise-init-script "init.sh"
# After creating exercise directory, runs: ./init.sh assignment03/
# Example init.sh:
# #!/bin/bash
# cd $1
# mkdir src tests
# cp ../template.py src/main.py
```

### Exercise Operations

```bash
# List exercises
mms ex list                          # Current course
mms ex list --all                    # All courses
mms ex list --upcoming               # Upcoming deadlines
mms ex list --overdue                # Overdue exercises
mms ex list --submitted              # Already submitted

# Output format:
# CS101 Exercises:
# ✓ assignment01  Dynamic Programming      Submitted (2024-12-10)  Grade: 45/50
#   assignment02  Graph Algorithms          Due: 2024-12-20 (3 days)
# ⚠ assignment03  NP-Completeness          Overdue by 2 days

# Show exercise details
mms ex info 01
mms ex info assignment01
# Output:
# Exercise 01 - Dynamic Programming
# Course: CS101
# Assigned: 2024-12-01
# Due: 2024-12-10
# Status: Submitted (2024-12-09 16:30)
# Grade: 45/50 (90%)
# Bonus contribution: 2.25/20 points
# Submission: solution.pdf
# Feedback: "Good work! Consider optimising..."

# Mark as submitted
mms ex submit 01
mms ex submit 01 --file solution.pdf
# Records submission timestamp, links file

# Grade exercise (if exercises contribute via bonus)
mms ex grade 01 --points 45/50
mms ex grade 01 --percentage 90
# Updates bonus point total for course

# Add feedback
mms ex feedback 01 --text "Good work overall..."
mms ex feedback 01 --file feedback.pdf

# Open exercise directory
mms ex open 01
cd ~/cc/exercises/assignment01       # Or navigate manually

# Delete exercise
mms ex delete 01 --confirm
```

### Exercise Integration with Grading

Exercises typically contribute as bonus points (if configured):

```bash
# View exercise contribution to grade
mms grade show cs101 --verbose
# Output:
# CS101 - Introduction to Algorithms
# Grading: Multiple components
# - Midterm: 2.0 (40%)
# - Final: 1.5 (60%)
# - Bonus: 15/20 points earned
#
# Exercise contributions:
# - assignment01: 9/10 points
# - assignment02: 6/10 points
# - Total: 15/20 points
#
# Base grade: (2.0 × 0.4 + 1.5 × 0.6) = 1.7
# Bonus: 15/20 = 75% of max → 15% improvement
# Final grade: 1.7 - 15% = 1.45 ≈ 1.5 (after rounding)
# (Capped at 1.0)
```

---

## Lecture Notes System

The notes system integrates with Typst for seamless note-taking during lectures. Slides are automatically numbered as `01.pdf`, `02.pdf`, `03.pdf` in the `slides/` folder.

### Initialisation

```bash
# Initialise notes for current course
cd ~/Studies/b3/cs101/
mms notes init

# Creates structure:
# notes/
# ├── main.typ              # Aggregates all lectures
# ├── template.typ          # Shared template
# ├── config.typ           # Course metadata
# ├── build/               # Compiled PDFs
# └── lectures/
#     └── .gitkeep

# With custom template
mms notes init --template advanced-lecture
```

### Creating Lecture Notes

```bash
# Create new lecture note (auto-increments)
mms notes new
# Creates: lectures/lecture_01.typ
# Opens in: Zed (or configured editor)
# Starts: typst watch
# Opens: Skim with PDF

# With details
mms notes new --number 07 --title "Dynamic Programming" --date "2024-12-16"

# Link slides (automatic from slides/ folder)
mms notes new --slides "01.pdf"
mms notes new --slides "01.pdf,02.pdf"  # Multiple decks

# Context-aware during scheduled lecture
# If currently in CS101 lecture time (from schedule):
mms notes new
# Auto-fills:
# - Number (from sequence + schedule)
# - Date (today)
# - Title (from schedule if set)
# - Slides (if available in slides/ folder)
```

### Lecture Note File Structure

**lecture_01.typ** (auto-generated):

```typst
// Auto-generated header - do not edit above this line
#import "../template.typ": *
#import "../config.typ": course, lecturer, semester

#lecture(
  number: 1,
  title: "Introduction & Complexity",
  date: datetime(year: 2024, month: 10, day: 15),
  slides: (
    (deck: "01.pdf", covered: (1, 25), skipped: ((15, 18),)),
    (deck: "02.pdf", covered: (1, 10), skipped: ()),
  )
)

// === YOUR NOTES START HERE ===

= Introduction

Today we covered the basics of algorithm complexity...

== Time Complexity

The runtime of an algorithm...

$
T(n) = O(n^2)
$

...
```

### Slide Coverage Tracking

Slide ranges can span multiple decks and support complex specifications:

```bash
# Mark slides covered (during lecture)
mms notes slides 03 --covered "01.pdf:1-20"
mms notes slides 03 --covered "01.pdf:25-40"
mms notes slides 03 --covered "02.pdf:1-10"

# Skip ranges within coverage
mms notes slides 03 --skip "01.pdf:15-17"

# Complex range specification
mms notes slides 03 --covered "01.pdf:1-20,02.pdf:1-15,03.pdf:5-10"
mms notes slides 03 --skip "01.pdf:8-10,02.pdf:12-13"

# Non-consecutive decks
mms notes slides 05 --covered "01.pdf:30-40,03.pdf:1-20"
# Deck 02.pdf not covered in this lecture

# Add more coverage incrementally
mms notes slides 03 --add-covered "02.pdf:11-20"

# View coverage for lecture
mms notes slides 03
# Output:
# Lecture 03 - Slide Coverage
#
# Deck 01.pdf:
#   Covered: 1-20, 25-40
#   Skipped: 15-17
#   Missing: 21-24, 41-50 (if deck has 50 slides)
#
# Deck 02.pdf:
#   Covered: 1-15
#   Skipped: 12-13
#   Missing: 16-30
#
# Deck 03.pdf:
#   Covered: 5-10
#   Missing: 1-4, 11-20
```

### Working with Notes

```bash
# Open existing lecture
mms notes open 03
mms notes open latest
mms notes open                       # Opens most recent

# List lectures
mms notes list
# Output:
# CS101 Lecture Notes:
# Lecture 01 (2024-10-15): Introduction & Complexity
#   Slides: 01.pdf [complete]
# Lecture 02 (2024-10-17): Graph Algorithms
#   Slides: 01.pdf [partial], 02.pdf [complete]
# Lecture 03 (2024-10-22): Dynamic Programming [IN PROGRESS]
#   Slides: 01.pdf [partial], 02.pdf [partial]
# Lecture 04 (2024-10-24): NP-Completeness
#   Slides: 01.pdf [not started]

# Edit lecture metadata
mms notes edit 03 --title "Advanced Dynamic Programming"
mms notes edit 03 --date "2024-10-23"

# Link slides after creation
mms notes link-slides 03 --file "04.pdf"

# Compile notes
mms notes compile                    # All lectures → single PDF
mms notes compile --lectures 1-5     # Specific range
mms notes compile 03                 # Single lecture

# Export
mms notes export --pdf               # Full notes PDF
mms notes export --format md         # Convert to Markdown (planned)
mms notes export 03 --format md      # Single lecture to MD

# Check workflow status
mms notes status
# Output:
# Active lecture: 03
# Watch process: Running (PID 12345)
# PDF viewer: Skim (open at lecture_03.pdf)
# Last compiled: 2 minutes ago
# Changes detected: Yes (compiling...)

# Stop watch process
mms notes stop

# Clean build artefacts
mms notes clean
```

### Slide Coverage Summary

```bash
# View slide coverage across lectures
mms notes coverage
# Output:
# CS101 - Slide Coverage
#
# Deck 01.pdf (50 slides)
#   Lecture 01: Slides 1-25 (50%)
#   Lecture 02: Slides 26-35 (20%)
#   Lecture 03: Slides 36-50 (30%)
#   Total: 100% covered
#
# Deck 02.pdf (40 slides)
#   Lecture 02: Slides 1-20 (50%)
#   Lecture 03: Slides 21-30 (25%)
#   Total: 75% covered
#   Missing: Slides 31-40
#
# Deck 03.pdf (30 slides)
#   Lecture 03: Slides 1-15 (50%)
#   Total: 50% covered
#   Missing: Slides 16-30

# Find which lecture covers specific slide
mms notes find-slide --deck "02.pdf" --page 25
# Output: "Lecture 03: Dynamic Programming (covered)"

mms notes find-slide --deck "02.pdf" --page 35
# Output: "Not covered in any lecture"

# Show gaps in coverage
mms notes coverage --missing
# Output:
# Material not covered in notes:
# - Deck 02.pdf: Slides 31-40
# - Deck 03.pdf: Slides 16-30

# Per-lecture coverage
mms notes coverage --lecture 03
# Output:
# Lecture 03 - Dynamic Programming
#
# Deck 01.pdf: Slides 36-50 (30% of deck)
# Deck 02.pdf: Slides 21-30 (25% of deck)
# Deck 03.pdf: Slides 1-15 (50% of deck)
#
# Total slides covered: 40 slides across 3 decks
```

### Advanced Notes Features

```bash
# Search across notes
mms notes search "dynamic programming"
# Output:
# Found in:
# - Lecture 03: Dynamic Programming (5 matches)
# - Lecture 07: Advanced DP (2 matches)
# - Lecture 09: DP Applications (3 matches)

# Open specific result
mms notes search "dynamic programming" --open 1
# Opens Lecture 03 in editor

# Create note from missed lecture (retroactive)
mms notes new --number 5 --retroactive --note "Not attended live"
# Marks in metadata: "Not attended live"

# Link external resources
mms notes link 03 --video "https://youtube.com/watch?v=..."
mms notes link 03 --recording "lecture_03_recording.mp4"

# View lecture with all links
mms notes info 03
# Output:
# Lecture 03 - Dynamic Programming
# Date: 2024-10-22
# Slides: 01.pdf (36-50), 02.pdf (21-30), 03.pdf (1-15)
# Coverage: 40 slides
# Resources:
#   - Video: https://youtube.com/watch?v=...
#   - Recording: lecture_03_recording.mp4
# Status: Completed
```

---

## Schedule & Timetable

Schedule entries are tied to courses and therefore to semesters. When a semester ends, its schedule is no longer active.

### Setting Up Schedule

```bash
# Add timetable entry
mms schedule add cs101 \
    --day monday \
    --time "10:00-12:00" \
    --room "MI HS1" \
    --type lecture

mms schedule add cs101 \
    --day wednesday \
    --time "14:00-16:00" \
    --room "MI HS2" \
    --type exercise \
    --tutor "Anna Müller"

# Multiple time slots
mms schedule add math201 \
    --day "tuesday,thursday" \
    --time "08:00-10:00" \
    --room "MW HS1" \
    --type lecture

# Set priority for overlaps
mms schedule add cs101 --priority 1     # Highest priority
mms schedule add math201 --priority 2   # Lower priority

# Import from file
mms schedule import timetable.ics    # iCalendar format
mms schedule import schedule.csv     # CSV format
# CSV format: course,day,start,end,room,type,priority

# Recurring events with exceptions
mms schedule add cs101 --day monday --time "10:00-12:00" \
    --except "2024-12-25,2025-01-01"

# Whole semester break
mms schedule break --start "2024-12-23" --end "2025-01-06" \
    --note "Christmas break"

# Single course cancelled
mms schedule cancel cs101 --date "2024-12-16" --reason "Public holiday"
```

### Viewing Schedule

```bash
# Current week
mms schedule show
mms schedule

# Output:
# Week of Dec 16-22, 2024
#
# Monday, Dec 16
# 10:00-12:00  CS101       Lecture    MI HS1     Prof. Schmidt    [P:1]
# 14:00-16:00  MATH201     Lecture    MW HS1     Prof. Wagner     [P:2]
#
# Tuesday, Dec 17
# 08:00-10:00  PHYS101     Lecture    PH HS2     Prof. Bauer      [P:3]
# 14:00-16:00  CS101       Exercise   MI HS2     Anna Müller      [P:1]
#
# Wednesday, Dec 18
# 10:00-12:00  CS101       Lecture    MI HS1     Prof. Schmidt    [P:1]
# 10:30-12:00  MATH201     Exercise   MW HS2     ⚠ OVERLAP
# ...

# Specific day
mms schedule show --today
mms schedule show --tomorrow
mms schedule show --date "2024-12-20"

# Specific week
mms schedule show --week next
mms schedule show --week "2024-12-23"

# Filter by course
mms schedule show cs101

# Filter by type
mms schedule show --type lecture
mms schedule show --type exercise
```

### Current/Next Event

```bash
# What's happening now
mms now
# Output:
# CS101 Lecture
# 10:00-12:00 (started 15 minutes ago)
# Room: MI HS1
# Lecturer: Prof. Schmidt
# Priority: 1

# If nothing is happening:
# Output: "No scheduled events right now"

# If overlap (shows highest priority):
# Output:
# CS101 Lecture [P:1]
# 10:00-12:00 (started 15 minutes ago)
# Room: MI HS1
# Also scheduled: MATH201 Exercise [P:2] (overlap)

# What's next
mms next
# Output:
# MATH201 Exercise
# 14:00-16:00 (in 1 hour 45 minutes)
# Room: MW HS2
# Tutor: Thomas Weber
```

### Auto-Switching

```bash
# Enable automatic course switching
mms schedule auto-switch on

# Configure behaviour
mms schedule config \
    --window 10 \              # Activate 10 min before start
    --notify true \            # Show notification
    --open-notes false         # Don't auto-open notes

# How it works:
# - 10 minutes before CS101 lecture starts
# - Automatically runs: mms course set cs101
# - Shows notification: "CS101 lecture starting in 10 min"
# - Symlink ~/cc points to cs101

# For overlaps: uses priority
# If CS101 (P:1) and MATH201 (P:2) overlap, switches to CS101

# Disable auto-switch
mms schedule auto-switch off

# Manual switch to current scheduled course
mms schedule now --activate
# Sets active course to whatever is scheduled now (highest priority if overlap)
```

### Managing Schedule

```bash
# Edit schedule entry
mms schedule edit cs101 monday --time "10:15-12:15"
mms schedule edit cs101 monday --room "MI HS3"
mms schedule edit cs101 monday --priority 2

# Remove entry
mms schedule remove cs101 monday

# Remove all for course
mms schedule remove cs101 --all

# Clear entire schedule (for semester)
mms schedule clear --confirm

# Export schedule
mms schedule export --format ics    # iCalendar
mms schedule export --format csv    # CSV
```

---

## Progress Tracking

### Overall Progress

GPA is weighted by ECTS. Areas marked with `--no-gpa` count towards ECTS but not towards degree GPA.

```bash
# View progress
mms progress

# Output:
# Bachelor in Computer Science (TUM)
# Started: Oct 2021  |  Expected completion: Sep 2024
#
# ═══════════════════════════════════════════════════
# ECTS Progress: 98/180 (54%)
# ═══════════════════════════════════════════════════
#
# Core CS          ████████░░  45/60  (75%)   GPA: 1.6
# Mathematics      ████████░░  24/30  (80%)   GPA: 1.8
# Practical        ███████░░░  20/30  (67%)   GPA: 2.1
# Electives        ███░░░░░░░   9/30  (30%)   GPA: 1.9
# Seminar          ░░░░░░░░░░   6/15  (40%)   (non-GPA)
#                                (6 ECTS)     Grade: 1.0
# Thesis           ░░░░░░░░░░   0/15  (0%)    --
#
# ═══════════════════════════════════════════════════
# Overall GPA: 1.78 (weighted by ECTS, excluding non-GPA areas)
# Completed courses: 14
# In progress: 4
# ═══════════════════════════════════════════════════

# Note: Seminar area shows ECTS and grade but marked as non-GPA
# Grade is displayed but doesn't contribute to overall GPA

# Detailed progress
mms progress --verbose

# Filter by degree
mms progress --bachelor
mms progress --master
mms progress --bachelor-cs      # Specific degree

# Multiple degrees
mms progress --all
# Shows progress for all active degrees

# Filter by area
mms progress --area "Core CS"
# Output:
# Core CS: 45/60 ECTS (75%)   GPA: 1.6
#
# Completed (45 ECTS):
# ✓ CS101  Introduction to Algorithms    8 ECTS  Grade: 1.7
# ✓ CS102  Data Structures               6 ECTS  Grade: 1.3
# ✓ CS201  Operating Systems             8 ECTS  Grade: 2.0
# ...
#
# In Progress (12 ECTS):
# ● CS301  Compiler Design               8 ECTS
# ● CS302  Database Systems              4 ECTS
#
# Still Needed (3 ECTS):
# Any Core CS course
```

### What's Missing

```bash
# Show what's still needed
mms progress --missing
mms missing

# Output:
# Still needed for graduation (Bachelor CS):
#
# Core CS (15 ECTS)
#   Any Core CS courses
#
# Mathematics (6 ECTS)
#   Any mathematics course
#
# Seminar (9 ECTS) [non-GPA area]
#   Any seminar courses
#   Note: Counts towards ECTS but not GPA
#
# Thesis (15 ECTS)
#   Bachelor thesis
#
# Total remaining: 45 ECTS (≈2 semesters)

# By area
mms missing --area "Core CS"

# By degree (if multiple)
mms missing --degree bachelor-cs
mms missing --degree bachelor-math
```

### GPA Tracking

```bash
# Current GPA (weighted by ECTS)
mms gpa
# Output: "Current GPA: 1.78 (14 courses, 98 ECTS, excluding non-GPA areas)"

# GPA history
mms gpa --history
# Output:
# GPA Trend (Bachelor CS):
# B1: 2.1 (4 courses, 24 ECTS)
# B2: 1.9 (5 courses, 28 ECTS)
# B3: 1.8 (5 courses, 30 ECTS) [in progress]
# Overall: 1.78

# GPA by area (excluding non-GPA areas)
mms gpa --by-area
# Output:
# GPA by Area (Bachelor CS):
# Core CS:      1.6 (8 courses, 45 ECTS)
# Mathematics:  1.8 (4 courses, 24 ECTS)
# Practical:    2.1 (3 courses, 20 ECTS)
# Electives:    1.9 (2 courses, 9 ECTS)
# Seminar:      -- (non-GPA area, grades not counted)
#               Courses: 1 (1.0), ECTS: 6

# By degree
mms gpa --degree bachelor-cs
mms gpa --degree bachelor-math
```

---

## Simulation & Planning

### Grade Simulation

```bash
# Interactive simulator
mms simulate
# Launches TUI with:
# - List of courses with current/missing grades
# - Inputs for future grades
# - Real-time GPA calculation
# - Weighs grades by ECTS

# Command-line simulation
mms simulate cs101 --final 1.3
# Output:
# Current situation:
# - CS101: Midterm 2.0 (40%), Final -- (60%)
# - Current GPA: 1.78 (98 ECTS)
#
# Simulation:
# - CS101 Final = 1.3 → Course grade: 1.58
# - New GPA: 1.76 (↓ 0.02, 106 ECTS total)

# Multiple courses
mms simulate cs101=1.3 math201=1.7 phys101=2.0
# Output:
# Current GPA: 1.78 (98 ECTS)
# Simulated GPA: 1.82 (↑ 0.04, 120 ECTS total)
#
# Changes:
# - CS101:  -- → 1.3 (final grade: 1.58, +8 ECTS)
# - MATH201: -- → 1.7 (+6 ECTS)
# - PHYS101: -- → 2.0 (+8 ECTS)
#
# ECTS-weighted calculation shown

# Save simulation scenario
mms simulate save optimistic \
    cs101=1.0 math201=1.3 phys101=1.5

mms simulate save realistic \
    cs101=1.7 math201=2.0 phys101=2.3

# Load scenario
mms simulate load optimistic
# Output:
# Scenario: optimistic
# - CS101:  -- → 1.0 (8 ECTS)
# - MATH201: -- → 1.3 (6 ECTS)
# - PHYS101: -- → 1.5 (4 ECTS)
#
# Simulated GPA: 1.65 (↓ 0.13 from current)
# Total ECTS after: 116

# List saved scenarios
mms simulate list
# Output:
# Saved scenarios:
# - optimistic (created: 2024-12-10)
# - realistic (created: 2024-12-10)
# - worst-case (created: 2024-12-12)

# Compare scenarios
mms simulate compare optimistic realistic
# Output:
#                 Current  Optimistic  Realistic
# CS101           --       1.0         1.7
# MATH201         --       1.3         2.0
# PHYS101         --       1.5         2.3
#
# GPA             1.78     1.65        1.85
# Total ECTS      98       116         116
# Change                   -0.13       +0.07

# Delete scenario
mms simulate delete worst-case
```

### Target GPA

```bash
# What grades needed for target?
mms simulate target 1.5

# Output:
# Target GPA: 1.5
# Current GPA: 1.78 (98 ECTS)
#
# To achieve 1.5 in remaining courses (22 ECTS):
#
# Required grades (ECTS-weighted):
# - CS101 (8 ECTS): ≤ 1.2
# - MATH201 (6 ECTS): ≤ 1.8
# - PHYS101 (4 ECTS): ≤ 1.5
# - Additional courses (4 ECTS): ≤ 1.6
#
# Assessment: Challenging
# - Requires above-average performance in all courses
# - CS101 especially difficult (≤ 1.2)
# - Weighted heavily (8 ECTS)
#
# Alternative: Target 1.6 is more realistic
# (Run: mms simulate target 1.6)

# For specific courses
mms simulate target 1.5 --courses cs101,math201

# With constraints
mms simulate target 1.5 --max-grade 1.3
# Only shows solutions where no grade worse than 1.3
```

### Semester Planning

```bash
# Plan next semester
mms plan b4
mms plan next

# Browse available courses (from catalogue if imported)
mms catalog search
mms catalog list --area "Core CS"

# Add courses to plan
mms plan add cs301
mms plan add cs302 --tentative    # Not sure yet
mms plan add math301

# Map to specific degree (if multiple)
mms plan add cs301 --degree bachelor-cs --area "Core CS"

# Remove from plan
mms plan remove cs302

# View plan
mms plan show
# Output:
# Planned semester: b4
# Default university: TUM
#
# Courses:
# ✓ CS301  Compiler Design         8 ECTS  Core CS (Bachelor CS)
# ⚠ CS302  Database Systems        6 ECTS  Core CS (Bachelor CS) [tentative]
# ✓ MATH301 Analysis II            8 ECTS  Mathematics (Bachelor CS)
#
# Total: 22 ECTS (20 without tentative)
# Areas covered: Core CS (+14), Mathematics (+8)

# Show ECTS distribution
mms plan show --ects
# Output:
# ECTS by area (Bachelor CS):
# Core CS:      +14 (total: 59/60)
# Mathematics:  +8  (total: 32/30) ⚠ exceeds requirement
# ...

# Validate plan
mms plan validate
# Output:
# ✓ Total ECTS reasonable (22)
# ⚠ Warning: Heavy workload (3 theory-heavy courses)

# Import course list
mms plan import --file courses.csv

# Simulate impact of plan
mms plan simulate
# Output:
# If you achieve your ECTS-weighted average (1.8):
# - Current GPA: 1.78 (98 ECTS)
# - After b4: 1.79 (22 ECTS at 1.8 → 120 total ECTS)
# - Progress: 120/180 ECTS (67%)

# Confirm plan (creates courses)
mms plan confirm
# Prompts:
# "Create semester b4 with 3 courses (22 ECTS)? [Y/n]"
# "Import schedule if available? [Y/n]"
#
# Creates:
# - Semester b4
# - Courses: cs301, cs302, math301
# - Directory structure
# - Imports schedule (if available)
```

---

## Platform Integration

### Supported Platforms

- **Moodle**: Course materials, exercises, grades
- **Ilias**: Similar to Moodle
- **Google Drive**: Shared folders for course materials
- **Custom**: Any platform with API or web scraping (future)

### Authentication

Authentication needs investigation - likely requires per-session login.

```bash
# Authenticate with platform (implementation TBD)
mms auth moodle
# Likely opens browser for authentication

# List authenticated platforms
mms auth list
# Output:
# Authenticated platforms:
# ✓ moodle (TUM) - session active
# ✓ ilias (ETH) - session active
# ✗ gdrive - not authenticated

# Remove authentication
mms auth remove moodle
```

### Linking Courses

```bash
# Link course to platform
mms course link moodle "https://moodle.tum.de/course/view.php?id=12345"

# Auto-detect platform from URL
mms course link "https://ilias.ethz.ch/goto.php?target=crs_123456"

# Multiple platforms
mms course link moodle "https://..."
mms course link gdrive "https://drive.google.com/drive/folders/abc123"

# View linked platforms
mms course info cs101
# Shows:
# Platforms:
# - Moodle: https://moodle.tum.de/course/12345
# - Google Drive: https://drive.google.com/...

# Remove link
mms course unlink moodle
```

### Syncing Content

In early stages, fetching requires manual `mms ex new` first. Future goal: auto-create directories.

```bash
# Sync current course (early implementation)
mms sync
# Output:
# Syncing CS101...
# Checking Moodle...
#
# Found new content:
# - lecture_07.pdf (slides)
# - assignment03.pdf (exercise)
#
# Actions:
# [1] Download lecture_07.pdf to slides/ ? [Y/n]: y
# ✓ Downloaded: lecture_07.pdf → slides/
#
# [2] Found assignment03.pdf
#     No directory exists. Create assignment03/? [Y/n]: y
# ✓ Created: exercises/assignment03/
# ✓ Downloaded: assignment03.pdf → exercises/assignment03/
# ✓ Generated: solution.typ from template

# Sync specific content type
mms sync --slides
mms sync --exercises
mms sync --grades

# Sync all courses
mms sync --all

# Check what's new (dry run)
mms sync --check
# Output:
# New content available:
#
# CS101:
# - lecture_07.pdf (slides)
# - assignment03.pdf (exercise, due: 2024-12-27)
#
# MATH201:
# - assignment02_solution.pdf (exercise solution)

# Force re-sync
mms sync --force
```

### Opening Platforms

```bash
# Open platform in browser
mms open                      # Current course, primary platform
mms open moodle              # Specific platform
mms open --all               # All linked platforms

# For specific course
mms open cs101
mms open cs101 --moodle

# Open folders
mms open slides              # Opens slides folder
mms open exercises           # Opens exercises folder
mms open notes               # Opens notes folder
```

---

## Reports & Export

### Generating Reports

```bash
# Transcript (grade overview)
mms report transcript
# Output (table format):
# Semester | Course    | ECTS | Grade | Area          | GPA
# ---------|-----------|------|-------|---------------|-----
# b1       | CS101     | 8    | 1.7   | Core CS       | Yes
# b1       | MATH101   | 6    | 2.0   | Mathematics   | Yes
# b1       | SEM101    | 6    | 1.0   | Seminar       | No
# ...

# Non-GPA courses shown with notation
# Export transcript as PDF
mms report transcript --pdf
mms report transcript --pdf --output transcript.pdf

# Filter by semester
mms report transcript --semester b3

# Filter by area
mms report transcript --area "Core CS"

# Show only GPA-counted courses
mms report transcript --gpa-only

# Official format (for university submission)
mms report transcript --official --pdf
# Formats according to university standards
```

### Grade Reports

```bash
# Grade summary
mms report grades
# Output:
# Grade Report (Bachelor CS)
# ==========================
# Overall GPA: 1.78 (weighted by ECTS, excluding non-GPA areas)
# Completed courses: 14 (12 GPA-counted, 2 non-GPA)
# Total ECTS: 98 (92 GPA-counted, 6 non-GPA)
#
# By semester:
# b1: GPA 2.1 (24 ECTS, 2 non-GPA courses with 6 ECTS)
# b2: GPA 1.9 (28 ECTS)
# b3: GPA 1.8 (30 ECTS, in progress)
#
# By area:
# Core CS:      GPA 1.6 (45 ECTS)
# Mathematics:  GPA 1.8 (24 ECTS)
# Seminar:      -- (non-GPA, 6 ECTS, avg: 1.0)
# ...

# Export as CSV
mms report grades --format csv --output grades.csv

# Export as JSON
mms report grades --format json --output grades.json

# By degree
mms report grades --degree bachelor-cs
mms report grades --degree bachelor-math
```

### Progress Reports

```bash
# Progress overview
mms report progress
# Includes:
# - ECTS progress by area (with non-GPA notation)
# - GPA trend (weighted)
# - Courses completed/in progress
# - Estimated graduation date

# Export as PDF
mms report progress --pdf --output progress-report.pdf

# By degree
mms report progress --degree bachelor-cs
```

### Data Export

```bash
# Export all data
mms export
# Creates: mms-export-2024-12-17.json
# Contains: courses, grades, exercises, notes metadata

# Export specific data
mms export grades --format csv
mms export courses --format json
mms export exercises --format csv

# Full backup (includes database)
mms export --backup
# Creates: mms-backup-2024-12-17.tar.gz
# Contains:
# - data.db (SQLite database)
# - config.toml
# - All configuration files
# - Does NOT include actual files (slides, notes, exercises)

# Export with files
mms export --backup --include-files
# Creates: mms-full-backup-2024-12-17.tar.gz
# Warning: Can be very large!

# Restore from backup
mms import --backup mms-backup-2024-12-17.tar.gz
```

---

## Contact Management

### Course Contacts

```bash
# View contacts for current course
mms contact list
mms contacts

# Output:
# CS101 Contacts:
#
# Lecturer:
#   Prof. Dr. Schmidt
#   schmidt@tum.de
#   Office: MI 03.11.055
#
# Exercise Coordinator:
#   exercises@tum.de
#
# Tutor:
#   Anna Müller
#   anna.mueller@tum.de
#   Office hours: Wed 14-16

# For specific course
mms contact list cs101
```

### Sending Emails

```bash
# Open mail client with recipient
mms contact mail lecturer
mms contact mail tutor
mms contact mail exercise

# For specific course
mms contact mail cs101 --lecturer

# CC all course contacts
mms contact mail cs101 --all

# With subject/body (if mail client supports)
mms contact mail lecturer \
    --subject "Question about assignment03" \
    --body "Dear Prof. Schmidt,..."
```

### Managing Contacts

```bash
# Update contact
mms contact update cs101 --tutor "New Tutor" --tutor-email "new@tum.de"

# Add office hours
mms contact update cs101 --tutor-office-hours "Wed 14-16"

# Export contacts
mms contacts export --format vcf    # vCard format
mms contacts export --format csv
```

---

## Templates

### Template System

Templates are used for:
- Exercise submissions (Typst/LaTeX)
- Lecture notes (Typst)
- Reports
- Custom documents

### Managing Templates

```bash
# List templates
mms template list

# Output:
# Available templates:
#
# Exercises:
# - typst-exercise (default)
# - latex-exercise
# - markdown-exercise
#
# Lecture Notes:
# - default-lecture (default)
# - advanced-lecture
# - minimal-lecture
#
# Reports:
# - project-report
# - seminar-paper

# View template
mms template show typst-exercise

# Edit template
mms template edit typst-exercise
# Opens template file in editor

# Create new template
mms template create my-custom-exercise --type exercise

# Set default
mms template set-default exercise typst-exercise
mms template set-default lecture advanced-lecture

# Delete template
mms template delete my-custom-exercise --confirm
```

### Template Variables

Templates can use these variables (auto-filled):

```
{{student_name}}           Your name
{{student_id}}             Matriculation number
{{course_name}}            Full course name
{{course_code}}            Course code (e.g., CS101)
{{semester}}               Semester code (e.g., b3)
{{exercise_number}}        Exercise number
{{due_date}}               Due date (if applicable)
{{date}}                   Current date
{{lecturer}}               Course lecturer
{{tutor}}                  Course tutor
{{university}}             Course university
```

**Example Typst template:**

```typst
#set document(title: "{{course_code}} - Exercise {{exercise_number}}")
#set page(header: [
  #grid(
    columns: (1fr, 1fr),
    [{{student_name}} ({{student_id}})],
    align(right)[{{course_code}} - {{semester}}]
  )
])

#align(center)[
  = Exercise {{exercise_number}}
  {{course_name}} ({{university}})

  Due: {{due_date}}
]

// Your solution here
```

---

## Archive & History

### Archiving

Dropped or failed courses are moved to `~/Studies/.archive/`.

```bash
# Archive dropped course
mms course drop cs101
# Prompts: "Move CS101 to archive? [Y/n]"
# Moves to: ~/Studies/.archive/b3/cs101/

# Archive completed semester
mms semester archive b1
# Moves to: ~/Studies/.archive/b1/

# View archived items
mms archive list
# Output:
# Archived courses:
# b2/cs102 (dropped 2022-11-15)
# b3/phys101 (dropped 2024-10-20)
#
# Archived semesters:
# b1 (2021-10 to 2022-03)

# Filter
mms archive list --dropped       # Only dropped courses
mms archive list --failed        # Only failed courses
mms archive list --semesters     # Only semesters

# Restore from archive (for retake with links)
mms archive restore cs101 --semester b4 --link
# Moves to: ~/Studies/b4/cs101/ (new structure)
# Creates symlinks:
#   ~/Studies/b4/cs101/notes_archived -> ~/Studies/.archive/b3/cs101/notes
#   ~/Studies/b4/cs101/exercises_archived -> ~/Studies/.archive/b3/cs101/exercises

# Restore without links (fresh start)
mms archive restore cs101 --semester b4
# Moves to: ~/Studies/b4/cs101/ (empty structure)

# Permanently delete archived item (with confirmation)
mms archive delete cs101 --confirm
# Prompts: "Permanently delete CS101 from archive? This cannot be undone. [y/N]"
# Deletes files AND database records
```

### History

Archived courses are completely excluded from statistics and GPA calculations. They are only visible in history views.

```bash
# View complete academic history
mms history

# Output:
# Academic History
# ================
#
# Bachelor in Computer Science (TUM)
# Started: Oct 2021
#
# Semester b1 (2021-10 to 2022-03) [completed]
#   CS101  Introduction to Algorithms  8 ECTS  1.7
#   CS102  Data Structures            6 ECTS  1.3
#   SEM101 Seminar                    6 ECTS  1.0 (non-GPA)
#   ...
#
# Semester b2 (2022-04 to 2022-09) [completed]
#   ...
#
# Archived/Dropped:
#   b2/PHYS101 (dropped 2022-06-15)

# Grade history only
mms history --grades

# Timeline view
mms history --timeline
# ASCII timeline showing:
# - Semesters
# - Courses started/completed
# - Grade milestones
# - GPA trend (weighted)

# Export history
mms history --format pdf --output my-history.pdf

# Include archived in history
mms history --include-archived
```

---

## Import External Structure

For users who already have an existing study directory structure, MMS provides import functionality.

### Import Existing Structure

```bash
# Import entire semester folder
mms import semester ~/MyOldStudies/Semester3 \
    --code b3 \
    --start "2024-10-01" \
    --end "2025-03-31" \
    --university TUM

# Import detects subdirectories as courses
# Prompts for each:
# "Found directory: Algorithms
#  Course code: " cs101
# "Course name: " Introduction to Algorithms
# "ECTS: " 8
# "Grade: " 1.7
# "Grading scheme: " german
# "Area: " Core CS

# Import individual course
mms import course ~/MyOldStudies/SomeClass \
    --code cs101 \
    --name "Introduction to Algorithms" \
    --ects 8 \
    --grade 1.7 \
    --scheme german \
    --semester b3 \
    --university TUM \
    --area "Core CS"

# Batch import from CSV
mms import --csv courses.csv
# CSV format:
# path,code,name,ects,grade,scheme,semester,university,area
# ~/Old/Algo,cs101,Intro to Algorithms,8,1.7,german,b3,TUM,Core CS
# ~/Old/Math,math101,Linear Algebra,6,2.0,german,b3,TUM,Mathematics

# Imported courses marked as "external"
# Shows warning when accessed:
# "⚠ Warning: CS101 is an external course (non-standard structure)"
# "Original location: ~/MyOldStudies/Algorithms"

# View external courses
mms course list --external
# Output:
# External courses:
# CS101  Introduction to Algorithms  b3  ~/MyOldStudies/Algorithms
# MATH101 Linear Algebra            b3  ~/MyOldStudies/Math

# Convert external to standard structure
mms course convert cs101
# Prompts:
# "Convert CS101 to standard MMS structure? [Y/n]"
# "This will:
#  1. Create ~/Studies/b3/cs101/
#  2. Copy/move files from ~/MyOldStudies/Algorithms
#  3. Organise into slides/, notes/, exercises/
#  4. Remove external flag
#
# Proceed? [Y/n]: " y
#
# ✓ Created standard structure
# ✓ Moved 15 PDF files → slides/
# ✓ Moved 3 directories → exercises/
# ✓ Created notes/ directory
# ✓ CS101 is now a standard course

# Import only for grades (no file management)
mms import grades-only --csv grades.csv
# CSV format:
# code,name,ects,grade,scheme,semester,university,area
# cs101,Intro to Algorithms,8,1.7,german,b1,TUM,Core CS
# math101,Linear Algebra,6,2.0,german,b1,TUM,Mathematics
#
# Creates course entries in database without linking to files
# Marked as external, no directory structure created
```

---

## Shell Integration

### Fish Shell Completions

```fish
# ~/.config/fish/completions/mms.fish

# Command completions
complete -c mms -n '__fish_use_subcommand' -a 'course semester grade ex notes schedule progress simulate plan degree'

# Course names
complete -c mms -n '__fish_seen_subcommand_from course' -a '(mms course list --short)'

# Semester codes
complete -c mms -n '__fish_seen_subcommand_from semester' -a '(mms semester list --short)'

# Exercise numbers
complete -c mms -n '__fish_seen_subcommand_from ex' -a '(mms ex list --short)'

# Degree IDs
complete -c mms -n '__fish_seen_subcommand_from degree' -a '(mms degree list --short)'

# ... (full completions)
```

### Helper Functions

```fish
# ~/.config/fish/config.fish

# Quick navigation to current course
function cc
    cd (readlink ~/cc)
end

# Quick navigation to current semester
function cs
    cd (readlink ~/cs)
end

# Fuzzy course switcher
function mcd
    set course (mms course list --short | fzf)
    and mms course set $course
    and cd ~/cc
end

# Fuzzy exercise switcher
function mex
    set ex (mms ex list --short | fzf)
    and cd ~/cc/exercises/$ex
end

# Quick grade entry
function grade
    if test (count $argv) -eq 1
        mms grade add $argv[1]
    else
        mms grade add $argv[1] --course $argv[2]
    end
end

# Show next class
function next
    mms next
end

# Current schedule
function now
    mms now
end
```

### Aliases

```fish
# Suggested aliases
alias g='mms grade'
alias ex='mms ex'
alias n='mms notes'
alias s='mms schedule'
alias p='mms progress'
alias d='mms degree'
```

---

## Implementation Priority & Roadmap

### Phase 1: Core Functionality (MVP)
- [ ] Database schema design
- [ ] Configuration management
- [ ] Degree, semester, course CRUD operations
- [ ] Context resolution (pwd, active course/semester)
- [ ] Basic grade management (single grades, schemes)
- [ ] Progress tracking (ECTS, GPA weighted)
- [ ] Non-GPA area support
- [ ] Multiple degree support
- [ ] Import external structure (basic)

### Phase 2: Academic Features
- [ ] Multiple grade components
- [ ] Bonus point systems (configurable)
- [ ] Exam attempts tracking
- [ ] First passing attempt policy
- [ ] Exercise management (manual creation)
- [ ] Lecture notes system (Typst integration)
- [ ] Slide coverage tracking (multi-deck)
- [ ] Schedule management
- [ ] Auto-switching courses

### Phase 3: Advanced Features
- [ ] Grade simulation
- [ ] Semester planning
- [ ] Platform integration (Moodle/Ilias - auth TBD)
- [ ] Exercise auto-fetching
- [ ] Reports and exports (PDF/CSV/JSON)
- [ ] Templates system
- [ ] Archive management
- [ ] History tracking

### Phase 4: Polish & Extensions
- [ ] TUI for simulation
- [ ] TUI for schedule display
- [ ] Shell completions (Fish, Zsh, Bash)
- [ ] Grade trend analysis
- [ ] Course conflict detection
- [ ] Community config repository
- [ ] Announcements/forums parsing (if feasible)
- [ ] Enhanced import features

---

## Outstanding Implementation Questions

These questions require technical investigation or user feedback before finalisation:

### Technical Questions

1. **Slide File Watching**: For auto-linking updated slides to lectures - how should the file system watcher work?
   - Should it watch the entire `slides/` directory?
   - How to handle renamed files (e.g., `lecture_07_v2.pdf`)?
   - Should it prompt before auto-linking?

2. **Platform Authentication**: Moodle/Ilias authentication strategy:
   - Session-based (re-authenticate each sync)?
   - Token-based (if supported by platforms)?
   - OAuth (if available)?
   - What's the best approach for each platform?

3. **Exercise Auto-Creation**: When auto-creating exercise directories:
   - How to detect exercise naming scheme reliably?
   - What if files don't follow `assignment{n}.pdf` pattern?
   - Should there be a learning mode that improves detection over time?

4. **Bonus Point Calculation**: Implementation of various bonus systems:
   - How to handle edge cases (e.g., bonus earned but exam failed)?
   - Should bonus be retroactively recalculated if policy changes?
   - How to store formula-based bonus systems in database?

5. **Grade Conversion Tables**: For abroad grades:
   - Should conversion be bijective (reversible)?
   - How to handle grades that don't map cleanly?
   - Should there be visual indicators of conversion quality?

### User Experience Questions

6. **External Course Conversion**: When converting external to standard:
   - How to handle unrecognised file types?
   - Should there be a preview/dry-run mode?
   - What if source directory structure is completely non-standard?

7. **Schedule Overlap Resolution**: Beyond priority numbers:
   - Should MMS prompt user on conflict?
   - Should there be a "preferred course" setting?
   - How to handle three-way overlaps?

8. **Multi-Degree Course Mapping**: When a course counts towards multiple degrees:
   - Should there be a UI to visualise cross-degree courses?
   - How to display this in reports?
   - Should GPA be calculated per-degree or overall?

9. **Archive Retention**: Although no auto-cleanup:
   - Should there be archive size warnings?
   - Should there be easy "bulk delete old archives" functionality?
   - How to handle archives from deleted degrees?

10. **Template Variables**: For advanced template features:
    - Should templates support conditionals (e.g., if abroad university)?
    - Should there be loops (e.g., for multiple tutors)?
    - How complex should template language be?
