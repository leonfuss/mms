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
