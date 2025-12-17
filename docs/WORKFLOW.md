### Workflow 1: Starting University

```bash
# Initial setup
mms init
mms config import --code "TUM-CS-B-2023"
mms degree create bachelor "Computer Science" --university TUM

# First semester
mms semester create b1 --start "2021-10-01" --end "2022-03-31"
mms course add cs101 cs102 math101 phys101
mms schedule import timetable.ics

# Activate
mms semester set b1
mms course set cs101
cd ~/cs
```

### Workflow 2: Recording Grades

```bash
cd ~/Studies/b1/cs101
mms grade setup --components "midterm:40,final:60"
mms grade bonus --max-points 20 --max-bonus-percent 20 --function linear

# Throughout semester
mms ex grade assignment01 --points 9/10
mms ex grade assignment02 --points 7/10

# After exams
mms grade add --midterm 2.0
mms grade add --final 1.5

# View result
mms grade show
# Output shows final grade with bonus applied
```

### Workflow 3: Lecture Notes

```bash
cd ~/cc  # Current course
mms notes init
mms notes new --title "Introduction"
# Zed opens, Typst watch starts, Skim opens PDF

# During lecture
mms notes slides 01 --covered "01.pdf:1-25,02.pdf:1-10"
mms notes slides 01 --skip "01.pdf:15-17"

# End of semester
mms notes compile --pdf
mms notes coverage --missing  # Check what wasn't covered
```

### Workflow 4: Abroad Semester

```bash
# Set up abroad semester
mms semester create b4 \
    --start "2025-04-01" \
    --end "2025-09-30" \
    --university "Università di Milano"

# Add courses (inherits Italian scheme)
mms course add algorithms
mms course add databases

# Record grades (automatic scheme)
mms grade add algorithms 28
mms grade add databases 30+

# View with conversion
mms grade show --convert german
# Shows: 28 (Italian) → 1.7 (German)

# Check progress
mms progress --bachelor
# Italian grades shown with ECTS counted, GPA calculated
```

### Workflow 5: Failed Exam Retake

```bash
# First attempt
mms exam add cs101 --grade 5.0 --date "2024-07-15"
# Output: Failed (2 attempts remaining)

# Second attempt
mms exam add cs101 --grade 2.3 --date "2024-09-20"
# Output: Passed (first passing attempt now active)

# Try to improve (policy might prevent)
mms exam add cs101 --grade 1.7
# Error: Cannot retake after passing (TUM policy)

# View history
mms exam history cs101
# Shows all attempts, active grade marked
```

### Workflow 6: Double Major

```bash
# Set up two degrees
mms degree create bachelor "Computer Science" --university TUM
mms degree create bachelor "Mathematics" --university TUM

# Add course that counts towards both
mms course add linear_algebra
mms course map linear_algebra \
    --degree bachelor-cs --area "Mathematics" \
    --degree bachelor-math --area "Core Mathematics"

# Progress tracking
mms progress --degree bachelor-cs
mms progress --degree bachelor-math
# Shows 6 ECTS counted in both
```

### Workflow 7: Import Existing Studies

```bash
# Import old structure
mms import semester ~/OldStudies/Semester1 \
    --code b1 --start "2021-10-01" --end "2022-03-31"

# Or from CSV
mms import --csv old-grades.csv

# View imported external courses
mms course list --external
```


### User Experience Examples

### Example 1: Course with Unselected Categories

```bash
$ mms progress

Bachelor in Computer Science (TUM)
═════════════════════════════════════════════════

Core CS          ████████░░  45/60  (75%)   GPA: 1.6
ML Specialization ███░░░░░░░  12/30  (40%)   GPA: 1.8
Electives        ████░░░░░░  20/30  (67%)   GPA: 2.1

⚠️  Warning: 2 courses need category assignment:
  - Machine Learning (6 ECTS) - can count towards 3 areas
  - Deep Learning (6 ECTS) - can count towards 2 areas

Run: mms course apply-pending
```

### Example 2: Smart Recommendations

```bash
$ mms course apply ml

Where should "Machine Learning" (6 ECTS) count?

1. ✓ Core CS (45/60 ECTS) [recommended]
     → Would reach 51/60 ECTS (85%)

2.   ML Specialization (12/30 ECTS)
     → Would reach 18/30 ECTS (60%)

3.   Electives (20/30 ECTS)
     → Would reach 26/30 ECTS (87%)

Recommendation: Choose Core CS (you need 15 more ECTS there)

Choice [1-3]:
```

### Example 3: Bulk Category Assignment

```bash
$ mms course apply-pending

Found 3 courses needing category assignment:

1. Machine Learning (6 ECTS)
   Options: Core CS [rec], ML Specialization, Electives
   → Apply to: Core CS

2. Deep Learning (6 ECTS)
   Options: ML Specialization [rec], Electives
   → Apply to: ML Specialization

3. Computer Vision (6 ECTS)
   Options: Core CS, ML Specialization [rec], Electives
   → Apply to: ML Specialization

Confirm these assignments? [Y/n]: y

✓ Machine Learning → Core CS
✓ Deep Learning → ML Specialization
✓ Computer Vision → ML Specialization

Updated progress:
Core CS:           51/60 ECTS (85%)
ML Specialization: 24/30 ECTS (80%)
```

---

## How It Works

### Scenario 1: Course Creation

```bash
$ mms course create ml --name "Machine Learning"

# System checks: Does this course have multiple possible categories?
# Query:
SELECT COUNT(*) FROM course_possible_categories
WHERE course_id = ?;

# If count = 0: Prompt user to add categories
# If count = 1: Auto-assign to that category
# If count > 1: Prompt user to choose
```

### Scenario 2: Adding Possible Categories

```bash
$ mms course add-category ml --degree bachelor-cs --area "Core CS"
$ mms course add-category ml --degree bachelor-cs --area "ML Specialization"
$ mms course add-category ml --degree bachelor-cs --area "Electives"

# Result: 3 rows in course_possible_categories
# No rows yet in course_degree_mappings (user hasn't chosen)
```

### Scenario 3: User Selects Where to Count the Course

```bash
$ mms course apply ml

# Output:
# Where should "Machine Learning" count?
#
# 1. Core CS (45/60 ECTS) [recommended]
# 2. ML Specialization (12/30 ECTS)
# 3. Electives (20/30 ECTS)
#
# Choice [1-3]: 2

# Result: Insert into course_degree_mappings
INSERT INTO course_degree_mappings (course_id, degree_id, area_id)
VALUES (?, ?, ?);
```

### Scenario 4: Changing the Mapping

```bash
$ mms course reapply ml

# Output:
# "Machine Learning" currently counts towards: ML Specialization
# Change to a different area?
#
# 1. Core CS (45/60 ECTS) [recommended]
# 2. ML Specialization (12/30 ECTS) [current]
# 3. Electives (20/30 ECTS)
#
# Choice [1-3, or 0 to cancel]: 1

# Result: Update course_degree_mappings
UPDATE course_degree_mappings
SET area_id = ?
WHERE course_id = ? AND degree_id = ?;
```

### Scenario 5: Importing Course with Multiple Categories

When importing from a shared config or university catalogue:

```toml
# course-catalogue.toml
[[courses]]
code = "cs301"
name = "Advanced Machine Learning"
ects = 6

[[courses.possible_categories]]
degree = "Bachelor Computer Science"
area = "Core CS"
recommended = true

[[courses.possible_categories]]
degree = "Bachelor Computer Science"
area = "ML Specialization"

[[courses.possible_categories]]
degree = "Bachelor Computer Science"
area = "Electives"
```

---
