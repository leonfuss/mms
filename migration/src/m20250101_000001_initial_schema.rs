use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Core Entity Tables
        create_semesters_table(manager).await?;
        create_degrees_table(manager).await?;
        create_degree_areas_table(manager).await?;
        create_courses_table(manager).await?;
        create_course_possible_categories_table(manager).await?;
        create_course_degree_mappings_table(manager).await?;

        // Grading Tables
        create_grades_table(manager).await?;
        create_grade_components_table(manager).await?;
        create_exam_attempts_table(manager).await?;

        // Schedule Tables
        create_course_schedules_table(manager).await?;
        create_course_events_table(manager).await?;
        create_holidays_table(manager).await?;
        create_holiday_exceptions_table(manager).await?;

        // Lecture & Exercise Tables
        create_lectures_table(manager).await?;
        create_slides_table(manager).await?;
        create_exercises_table(manager).await?;

        // State & Context Tables
        create_active_course_table(manager).await?;
        create_todos_table(manager).await?;

        // Platform Integration Tables
        create_platform_accounts_table(manager).await?;
        create_platform_course_links_table(manager).await?;

        // Create Views
        create_views(manager).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop views first
        drop_views(manager).await?;

        // Drop tables in reverse dependency order
        manager.drop_table(Table::drop().table(PlatformCourseLinks::Table).if_exists().to_owned()).await?;
        manager.drop_table(Table::drop().table(PlatformAccounts::Table).if_exists().to_owned()).await?;
        manager.drop_table(Table::drop().table(Todos::Table).if_exists().to_owned()).await?;
        manager.drop_table(Table::drop().table(ActiveCourse::Table).if_exists().to_owned()).await?;
        manager.drop_table(Table::drop().table(Exercises::Table).if_exists().to_owned()).await?;
        manager.drop_table(Table::drop().table(Slides::Table).if_exists().to_owned()).await?;
        manager.drop_table(Table::drop().table(Lectures::Table).if_exists().to_owned()).await?;
        manager.drop_table(Table::drop().table(HolidayExceptions::Table).if_exists().to_owned()).await?;
        manager.drop_table(Table::drop().table(Holidays::Table).if_exists().to_owned()).await?;
        manager.drop_table(Table::drop().table(CourseEvents::Table).if_exists().to_owned()).await?;
        manager.drop_table(Table::drop().table(CourseSchedules::Table).if_exists().to_owned()).await?;
        manager.drop_table(Table::drop().table(ExamAttempts::Table).if_exists().to_owned()).await?;
        manager.drop_table(Table::drop().table(GradeComponents::Table).if_exists().to_owned()).await?;
        manager.drop_table(Table::drop().table(Grades::Table).if_exists().to_owned()).await?;
        manager.drop_table(Table::drop().table(CourseDegreeMappings::Table).if_exists().to_owned()).await?;
        manager.drop_table(Table::drop().table(CoursePossibleCategories::Table).if_exists().to_owned()).await?;
        manager.drop_table(Table::drop().table(Courses::Table).if_exists().to_owned()).await?;
        manager.drop_table(Table::drop().table(DegreeAreas::Table).if_exists().to_owned()).await?;
        manager.drop_table(Table::drop().table(Degrees::Table).if_exists().to_owned()).await?;
        manager.drop_table(Table::drop().table(Semesters::Table).if_exists().to_owned()).await?;

        Ok(())
    }
}

// ==========================================
// Core Entity Tables
// ==========================================

#[derive(DeriveIden)]
enum Semesters {
    Table,
    Id,
    Type,
    Number,
    DirectoryPath,
    ExistsOnDisk,
    LastScannedAt,
    StartDate,
    EndDate,
    DefaultLocation,
    University,
    IsCurrent,
    IsArchived,
    CreatedAt,
    UpdatedAt,
}

async fn create_semesters_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(Semesters::Table)
                .if_not_exists()
                .col(pk_auto(Semesters::Id))
                .col(string(Semesters::Type))
                .col(integer(Semesters::Number))
                .col(string(Semesters::DirectoryPath))
                .col(boolean(Semesters::ExistsOnDisk).default(true))
                .col(timestamp_null(Semesters::LastScannedAt))
                .col(string_null(Semesters::StartDate))
                .col(string_null(Semesters::EndDate))
                .col(string(Semesters::DefaultLocation))
                .col(string_null(Semesters::University))
                .col(boolean(Semesters::IsCurrent).default(false))
                .col(boolean(Semesters::IsArchived).default(false))
                .col(timestamp(Semesters::CreatedAt).default(Expr::current_timestamp()))
                .col(timestamp(Semesters::UpdatedAt).default(Expr::current_timestamp()))
                .to_owned(),
        )
        .await?;

    // Create indexes
    manager
        .create_index(
            Index::create()
                .name("idx_semesters_current")
                .table(Semesters::Table)
                .col(Semesters::IsCurrent)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_semesters_archived")
                .table(Semesters::Table)
                .col(Semesters::IsArchived)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_semesters_disk")
                .table(Semesters::Table)
                .col(Semesters::ExistsOnDisk)
                .to_owned(),
        )
        .await?;

    // Create unique constraint
    manager
        .create_index(
            Index::create()
                .name("idx_semesters_type_number")
                .table(Semesters::Table)
                .col(Semesters::Type)
                .col(Semesters::Number)
                .unique()
                .to_owned(),
        )
        .await?;

    Ok(())
}

#[derive(DeriveIden)]
enum Degrees {
    Table,
    Id,
    Type,
    Name,
    University,
    TotalEctsRequired,
    IsActive,
    StartDate,
    ExpectedEndDate,
    CreatedAt,
    UpdatedAt,
}

async fn create_degrees_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(Degrees::Table)
                .if_not_exists()
                .col(pk_auto(Degrees::Id))
                .col(string(Degrees::Type))
                .col(string(Degrees::Name))
                .col(string(Degrees::University))
                .col(integer(Degrees::TotalEctsRequired))
                .col(boolean(Degrees::IsActive).default(true))
                .col(string_null(Degrees::StartDate))
                .col(string_null(Degrees::ExpectedEndDate))
                .col(timestamp(Degrees::CreatedAt).default(Expr::current_timestamp()))
                .col(timestamp(Degrees::UpdatedAt).default(Expr::current_timestamp()))
                .to_owned(),
        )
        .await?;

    // Create indexes
    manager
        .create_index(
            Index::create()
                .name("idx_degrees_active")
                .table(Degrees::Table)
                .col(Degrees::IsActive)
                .to_owned(),
        )
        .await?;

    // Create unique constraint
    manager
        .create_index(
            Index::create()
                .name("idx_degrees_type_name_university")
                .table(Degrees::Table)
                .col(Degrees::Type)
                .col(Degrees::Name)
                .col(Degrees::University)
                .unique()
                .to_owned(),
        )
        .await?;

    Ok(())
}

#[derive(DeriveIden)]
enum DegreeAreas {
    Table,
    Id,
    DegreeId,
    CategoryName,
    RequiredEcts,
    CountsTowardsGpa,
    DisplayOrder,
    CreatedAt,
}

async fn create_degree_areas_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(DegreeAreas::Table)
                .if_not_exists()
                .col(pk_auto(DegreeAreas::Id))
                .col(integer(DegreeAreas::DegreeId))
                .col(string(DegreeAreas::CategoryName))
                .col(integer(DegreeAreas::RequiredEcts))
                .col(boolean(DegreeAreas::CountsTowardsGpa).default(true))
                .col(integer(DegreeAreas::DisplayOrder).default(0))
                .col(timestamp(DegreeAreas::CreatedAt).default(Expr::current_timestamp()))
                .foreign_key(
                    ForeignKey::create()
                        .from(DegreeAreas::Table, DegreeAreas::DegreeId)
                        .to(Degrees::Table, Degrees::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                )
                .to_owned(),
        )
        .await?;

    // Create indexes
    manager
        .create_index(
            Index::create()
                .name("idx_degree_areas_degree")
                .table(DegreeAreas::Table)
                .col(DegreeAreas::DegreeId)
                .to_owned(),
        )
        .await?;

    // Create unique constraint
    manager
        .create_index(
            Index::create()
                .name("idx_degree_areas_degree_category")
                .table(DegreeAreas::Table)
                .col(DegreeAreas::DegreeId)
                .col(DegreeAreas::CategoryName)
                .unique()
                .to_owned(),
        )
        .await?;

    Ok(())
}

#[derive(DeriveIden)]
enum Courses {
    Table,
    Id,
    SemesterId,
    ShortName,
    Name,
    DirectoryPath,
    TomlPath,
    ExistsOnDisk,
    TomlExists,
    LastScannedAt,
    Ects,
    Lecturer,
    LecturerEmail,
    Tutor,
    TutorEmail,
    LearningPlatformUrl,
    University,
    Location,
    IsExternal,
    OriginalPath,
    IsArchived,
    IsDropped,
    DroppedAt,
    HasGitRepo,
    GitRemoteUrl,
    CreatedAt,
    UpdatedAt,
}

async fn create_courses_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(Courses::Table)
                .if_not_exists()
                .col(pk_auto(Courses::Id))
                .col(integer(Courses::SemesterId))
                .col(string(Courses::ShortName))
                .col(string(Courses::Name))
                .col(string(Courses::DirectoryPath))
                .col(string_null(Courses::TomlPath))
                .col(boolean(Courses::ExistsOnDisk).default(true))
                .col(boolean(Courses::TomlExists).default(true))
                .col(timestamp_null(Courses::LastScannedAt))
                .col(integer(Courses::Ects))
                .col(string_null(Courses::Lecturer))
                .col(string_null(Courses::LecturerEmail))
                .col(string_null(Courses::Tutor))
                .col(string_null(Courses::TutorEmail))
                .col(string_null(Courses::LearningPlatformUrl))
                .col(string_null(Courses::University))
                .col(string_null(Courses::Location))
                .col(boolean(Courses::IsExternal).default(false))
                .col(string_null(Courses::OriginalPath))
                .col(boolean(Courses::IsArchived).default(false))
                .col(boolean(Courses::IsDropped).default(false))
                .col(timestamp_null(Courses::DroppedAt))
                .col(boolean(Courses::HasGitRepo).default(false))
                .col(string_null(Courses::GitRemoteUrl))
                .col(timestamp(Courses::CreatedAt).default(Expr::current_timestamp()))
                .col(timestamp(Courses::UpdatedAt).default(Expr::current_timestamp()))
                .foreign_key(
                    ForeignKey::create()
                        .from(Courses::Table, Courses::SemesterId)
                        .to(Semesters::Table, Semesters::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                )
                .to_owned(),
        )
        .await?;

    // Create indexes
    manager
        .create_index(
            Index::create()
                .name("idx_courses_semester")
                .table(Courses::Table)
                .col(Courses::SemesterId)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_courses_disk")
                .table(Courses::Table)
                .col(Courses::ExistsOnDisk)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_courses_archived")
                .table(Courses::Table)
                .col(Courses::IsArchived)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_courses_external")
                .table(Courses::Table)
                .col(Courses::IsExternal)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_courses_toml")
                .table(Courses::Table)
                .col(Courses::TomlExists)
                .to_owned(),
        )
        .await?;

    // Create unique constraint
    manager
        .create_index(
            Index::create()
                .name("idx_courses_semester_shortname")
                .table(Courses::Table)
                .col(Courses::SemesterId)
                .col(Courses::ShortName)
                .unique()
                .to_owned(),
        )
        .await?;

    Ok(())
}

#[derive(DeriveIden)]
enum CoursePossibleCategories {
    Table,
    Id,
    CourseId,
    DegreeId,
    AreaId,
    IsRecommended,
    Notes,
    CreatedAt,
}

async fn create_course_possible_categories_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(CoursePossibleCategories::Table)
                .if_not_exists()
                .col(pk_auto(CoursePossibleCategories::Id))
                .col(integer(CoursePossibleCategories::CourseId))
                .col(integer(CoursePossibleCategories::DegreeId))
                .col(integer(CoursePossibleCategories::AreaId))
                .col(boolean(CoursePossibleCategories::IsRecommended).default(false))
                .col(string_null(CoursePossibleCategories::Notes))
                .col(timestamp(CoursePossibleCategories::CreatedAt).default(Expr::current_timestamp()))
                .foreign_key(
                    ForeignKey::create()
                        .from(CoursePossibleCategories::Table, CoursePossibleCategories::CourseId)
                        .to(Courses::Table, Courses::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(CoursePossibleCategories::Table, CoursePossibleCategories::DegreeId)
                        .to(Degrees::Table, Degrees::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(CoursePossibleCategories::Table, CoursePossibleCategories::AreaId)
                        .to(DegreeAreas::Table, DegreeAreas::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                )
                .to_owned(),
        )
        .await?;

    // Create indexes
    manager
        .create_index(
            Index::create()
                .name("idx_possible_categories_course")
                .table(CoursePossibleCategories::Table)
                .col(CoursePossibleCategories::CourseId)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_possible_categories_degree")
                .table(CoursePossibleCategories::Table)
                .col(CoursePossibleCategories::DegreeId)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_possible_categories_area")
                .table(CoursePossibleCategories::Table)
                .col(CoursePossibleCategories::AreaId)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_possible_categories_recommended")
                .table(CoursePossibleCategories::Table)
                .col(CoursePossibleCategories::IsRecommended)
                .to_owned(),
        )
        .await?;

    // Create unique constraint
    manager
        .create_index(
            Index::create()
                .name("idx_possible_categories_course_degree_area")
                .table(CoursePossibleCategories::Table)
                .col(CoursePossibleCategories::CourseId)
                .col(CoursePossibleCategories::DegreeId)
                .col(CoursePossibleCategories::AreaId)
                .unique()
                .to_owned(),
        )
        .await?;

    Ok(())
}

#[derive(DeriveIden)]
enum CourseDegreeMappings {
    Table,
    Id,
    CourseId,
    DegreeId,
    AreaId,
    EctsOverride,
    CreatedAt,
}

async fn create_course_degree_mappings_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(CourseDegreeMappings::Table)
                .if_not_exists()
                .col(pk_auto(CourseDegreeMappings::Id))
                .col(integer(CourseDegreeMappings::CourseId))
                .col(integer(CourseDegreeMappings::DegreeId))
                .col(integer(CourseDegreeMappings::AreaId))
                .col(integer_null(CourseDegreeMappings::EctsOverride))
                .col(timestamp(CourseDegreeMappings::CreatedAt).default(Expr::current_timestamp()))
                .foreign_key(
                    ForeignKey::create()
                        .from(CourseDegreeMappings::Table, CourseDegreeMappings::CourseId)
                        .to(Courses::Table, Courses::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(CourseDegreeMappings::Table, CourseDegreeMappings::DegreeId)
                        .to(Degrees::Table, Degrees::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(CourseDegreeMappings::Table, CourseDegreeMappings::AreaId)
                        .to(DegreeAreas::Table, DegreeAreas::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                )
                .to_owned(),
        )
        .await?;

    // Create indexes
    manager
        .create_index(
            Index::create()
                .name("idx_mappings_course")
                .table(CourseDegreeMappings::Table)
                .col(CourseDegreeMappings::CourseId)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_mappings_degree")
                .table(CourseDegreeMappings::Table)
                .col(CourseDegreeMappings::DegreeId)
                .to_owned(),
        )
        .await?;

    // Create unique constraint
    manager
        .create_index(
            Index::create()
                .name("idx_mappings_course_degree_area")
                .table(CourseDegreeMappings::Table)
                .col(CourseDegreeMappings::CourseId)
                .col(CourseDegreeMappings::DegreeId)
                .col(CourseDegreeMappings::AreaId)
                .unique()
                .to_owned(),
        )
        .await?;

    Ok(())
}

// ==========================================
// Grading Tables
// ==========================================

#[derive(DeriveIden)]
enum Grades {
    Table,
    Id,
    CourseId,
    Grade,
    GradingScheme,
    OriginalGrade,
    OriginalScheme,
    ConversionTable,
    IsFinal,
    Passed,
    AttemptNumber,
    ExamDate,
    RecordedAt,
    UpdatedAt,
}

async fn create_grades_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(Grades::Table)
                .if_not_exists()
                .col(pk_auto(Grades::Id))
                .col(integer(Grades::CourseId))
                .col(double(Grades::Grade))
                .col(string(Grades::GradingScheme))
                .col(double_null(Grades::OriginalGrade))
                .col(string_null(Grades::OriginalScheme))
                .col(string_null(Grades::ConversionTable))
                .col(boolean(Grades::IsFinal).default(true))
                .col(boolean(Grades::Passed))
                .col(integer(Grades::AttemptNumber).default(1))
                .col(string_null(Grades::ExamDate))
                .col(timestamp(Grades::RecordedAt).default(Expr::current_timestamp()))
                .col(timestamp(Grades::UpdatedAt).default(Expr::current_timestamp()))
                .foreign_key(
                    ForeignKey::create()
                        .from(Grades::Table, Grades::CourseId)
                        .to(Courses::Table, Courses::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                )
                .to_owned(),
        )
        .await?;

    // Create indexes
    manager
        .create_index(
            Index::create()
                .name("idx_grades_course")
                .table(Grades::Table)
                .col(Grades::CourseId)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_grades_final")
                .table(Grades::Table)
                .col(Grades::IsFinal)
                .to_owned(),
        )
        .await?;

    Ok(())
}

#[derive(DeriveIden)]
enum GradeComponents {
    Table,
    Id,
    CourseId,
    GradeId,
    ComponentName,
    Weight,
    PointsEarned,
    PointsTotal,
    Grade,
    IsBonus,
    BonusPoints,
    IsCompleted,
    DueDate,
    CompletedAt,
    CreatedAt,
    UpdatedAt,
}

async fn create_grade_components_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(GradeComponents::Table)
                .if_not_exists()
                .col(pk_auto(GradeComponents::Id))
                .col(integer(GradeComponents::CourseId))
                .col(integer_null(GradeComponents::GradeId))
                .col(string(GradeComponents::ComponentName))
                .col(double(GradeComponents::Weight))
                .col(double_null(GradeComponents::PointsEarned))
                .col(double_null(GradeComponents::PointsTotal))
                .col(double_null(GradeComponents::Grade))
                .col(boolean(GradeComponents::IsBonus).default(false))
                .col(double(GradeComponents::BonusPoints).default(0.0))
                .col(boolean(GradeComponents::IsCompleted).default(false))
                .col(string_null(GradeComponents::DueDate))
                .col(timestamp_null(GradeComponents::CompletedAt))
                .col(timestamp(GradeComponents::CreatedAt).default(Expr::current_timestamp()))
                .col(timestamp(GradeComponents::UpdatedAt).default(Expr::current_timestamp()))
                .foreign_key(
                    ForeignKey::create()
                        .from(GradeComponents::Table, GradeComponents::CourseId)
                        .to(Courses::Table, Courses::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(GradeComponents::Table, GradeComponents::GradeId)
                        .to(Grades::Table, Grades::Id)
                        .on_delete(ForeignKeyAction::SetNull)
                )
                .to_owned(),
        )
        .await?;

    // Create indexes
    manager
        .create_index(
            Index::create()
                .name("idx_components_course")
                .table(GradeComponents::Table)
                .col(GradeComponents::CourseId)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_components_grade")
                .table(GradeComponents::Table)
                .col(GradeComponents::GradeId)
                .to_owned(),
        )
        .await?;

    Ok(())
}

#[derive(DeriveIden)]
enum ExamAttempts {
    Table,
    Id,
    CourseId,
    AttemptNumber,
    ExamDate,
    ExamType,
    Grade,
    Passed,
    GradeId,
    Notes,
    Location,
    CreatedAt,
    UpdatedAt,
}

async fn create_exam_attempts_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(ExamAttempts::Table)
                .if_not_exists()
                .col(pk_auto(ExamAttempts::Id))
                .col(integer(ExamAttempts::CourseId))
                .col(integer(ExamAttempts::AttemptNumber))
                .col(string(ExamAttempts::ExamDate))
                .col(string_null(ExamAttempts::ExamType))
                .col(double_null(ExamAttempts::Grade))
                .col(boolean(ExamAttempts::Passed).default(false))
                .col(integer_null(ExamAttempts::GradeId))
                .col(string_null(ExamAttempts::Notes))
                .col(string_null(ExamAttempts::Location))
                .col(timestamp(ExamAttempts::CreatedAt).default(Expr::current_timestamp()))
                .col(timestamp(ExamAttempts::UpdatedAt).default(Expr::current_timestamp()))
                .foreign_key(
                    ForeignKey::create()
                        .from(ExamAttempts::Table, ExamAttempts::CourseId)
                        .to(Courses::Table, Courses::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(ExamAttempts::Table, ExamAttempts::GradeId)
                        .to(Grades::Table, Grades::Id)
                        .on_delete(ForeignKeyAction::SetNull)
                )
                .to_owned(),
        )
        .await?;

    // Create indexes
    manager
        .create_index(
            Index::create()
                .name("idx_attempts_course")
                .table(ExamAttempts::Table)
                .col(ExamAttempts::CourseId)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_attempts_passed")
                .table(ExamAttempts::Table)
                .col(ExamAttempts::Passed)
                .to_owned(),
        )
        .await?;

    // Create unique constraint
    manager
        .create_index(
            Index::create()
                .name("idx_attempts_course_number")
                .table(ExamAttempts::Table)
                .col(ExamAttempts::CourseId)
                .col(ExamAttempts::AttemptNumber)
                .unique()
                .to_owned(),
        )
        .await?;

    Ok(())
}

// ==========================================
// Schedule Tables
// ==========================================

#[derive(DeriveIden)]
enum CourseSchedules {
    Table,
    Id,
    CourseId,
    ScheduleType,
    DayOfWeek,
    StartTime,
    EndTime,
    StartDate,
    EndDate,
    Room,
    Building,
    Location,
    Priority,
    CreatedAt,
    UpdatedAt,
}

async fn create_course_schedules_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(CourseSchedules::Table)
                .if_not_exists()
                .col(pk_auto(CourseSchedules::Id))
                .col(integer(CourseSchedules::CourseId))
                .col(string(CourseSchedules::ScheduleType))
                .col(integer(CourseSchedules::DayOfWeek))
                .col(string(CourseSchedules::StartTime))
                .col(string(CourseSchedules::EndTime))
                .col(string(CourseSchedules::StartDate))
                .col(string(CourseSchedules::EndDate))
                .col(string_null(CourseSchedules::Room))
                .col(string_null(CourseSchedules::Building))
                .col(string_null(CourseSchedules::Location))
                .col(integer(CourseSchedules::Priority).default(0))
                .col(timestamp(CourseSchedules::CreatedAt).default(Expr::current_timestamp()))
                .col(timestamp(CourseSchedules::UpdatedAt).default(Expr::current_timestamp()))
                .foreign_key(
                    ForeignKey::create()
                        .from(CourseSchedules::Table, CourseSchedules::CourseId)
                        .to(Courses::Table, Courses::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                )
                .to_owned(),
        )
        .await?;

    // Create indexes
    manager
        .create_index(
            Index::create()
                .name("idx_schedules_course")
                .table(CourseSchedules::Table)
                .col(CourseSchedules::CourseId)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_schedules_day")
                .table(CourseSchedules::Table)
                .col(CourseSchedules::DayOfWeek)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_schedules_time")
                .table(CourseSchedules::Table)
                .col(CourseSchedules::StartTime)
                .to_owned(),
        )
        .await?;

    Ok(())
}

#[derive(DeriveIden)]
enum CourseEvents {
    Table,
    Id,
    CourseId,
    ScheduleId,
    EventType,
    Date,
    StartTime,
    EndTime,
    Room,
    Building,
    Location,
    Title,
    Description,
    CreatedAt,
    UpdatedAt,
}

async fn create_course_events_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(CourseEvents::Table)
                .if_not_exists()
                .col(pk_auto(CourseEvents::Id))
                .col(integer(CourseEvents::CourseId))
                .col(integer_null(CourseEvents::ScheduleId))
                .col(string(CourseEvents::EventType))
                .col(string(CourseEvents::Date))
                .col(string_null(CourseEvents::StartTime))
                .col(string_null(CourseEvents::EndTime))
                .col(string_null(CourseEvents::Room))
                .col(string_null(CourseEvents::Building))
                .col(string_null(CourseEvents::Location))
                .col(string_null(CourseEvents::Title))
                .col(string_null(CourseEvents::Description))
                .col(timestamp(CourseEvents::CreatedAt).default(Expr::current_timestamp()))
                .col(timestamp(CourseEvents::UpdatedAt).default(Expr::current_timestamp()))
                .foreign_key(
                    ForeignKey::create()
                        .from(CourseEvents::Table, CourseEvents::CourseId)
                        .to(Courses::Table, Courses::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(CourseEvents::Table, CourseEvents::ScheduleId)
                        .to(CourseSchedules::Table, CourseSchedules::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                )
                .to_owned(),
        )
        .await?;

    // Create indexes
    manager
        .create_index(
            Index::create()
                .name("idx_events_course")
                .table(CourseEvents::Table)
                .col(CourseEvents::CourseId)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_events_date")
                .table(CourseEvents::Table)
                .col(CourseEvents::Date)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_events_type")
                .table(CourseEvents::Table)
                .col(CourseEvents::EventType)
                .to_owned(),
        )
        .await?;

    Ok(())
}

#[derive(DeriveIden)]
enum Holidays {
    Table,
    Id,
    Name,
    StartDate,
    EndDate,
    University,
    HolidayType,
    CreatedAt,
}

async fn create_holidays_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(Holidays::Table)
                .if_not_exists()
                .col(pk_auto(Holidays::Id))
                .col(string(Holidays::Name))
                .col(string(Holidays::StartDate))
                .col(string(Holidays::EndDate))
                .col(string_null(Holidays::University))
                .col(string(Holidays::HolidayType))
                .col(timestamp(Holidays::CreatedAt).default(Expr::current_timestamp()))
                .to_owned(),
        )
        .await?;

    // Create indexes
    manager
        .create_index(
            Index::create()
                .name("idx_holidays_dates")
                .table(Holidays::Table)
                .col(Holidays::StartDate)
                .col(Holidays::EndDate)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_holidays_university")
                .table(Holidays::Table)
                .col(Holidays::University)
                .to_owned(),
        )
        .await?;

    // Create unique constraint
    manager
        .create_index(
            Index::create()
                .name("idx_holidays_name_start_university")
                .table(Holidays::Table)
                .col(Holidays::Name)
                .col(Holidays::StartDate)
                .col(Holidays::University)
                .unique()
                .to_owned(),
        )
        .await?;

    Ok(())
}

#[derive(DeriveIden)]
enum HolidayExceptions {
    Table,
    Id,
    HolidayId,
    CourseId,
    CreatedAt,
}

async fn create_holiday_exceptions_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(HolidayExceptions::Table)
                .if_not_exists()
                .col(pk_auto(HolidayExceptions::Id))
                .col(integer(HolidayExceptions::HolidayId))
                .col(integer(HolidayExceptions::CourseId))
                .col(timestamp(HolidayExceptions::CreatedAt).default(Expr::current_timestamp()))
                .foreign_key(
                    ForeignKey::create()
                        .from(HolidayExceptions::Table, HolidayExceptions::HolidayId)
                        .to(Holidays::Table, Holidays::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(HolidayExceptions::Table, HolidayExceptions::CourseId)
                        .to(Courses::Table, Courses::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                )
                .to_owned(),
        )
        .await?;

    // Create indexes
    manager
        .create_index(
            Index::create()
                .name("idx_exceptions_holiday")
                .table(HolidayExceptions::Table)
                .col(HolidayExceptions::HolidayId)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_exceptions_course")
                .table(HolidayExceptions::Table)
                .col(HolidayExceptions::CourseId)
                .to_owned(),
        )
        .await?;

    // Create unique constraint
    manager
        .create_index(
            Index::create()
                .name("idx_exceptions_holiday_course")
                .table(HolidayExceptions::Table)
                .col(HolidayExceptions::HolidayId)
                .col(HolidayExceptions::CourseId)
                .unique()
                .to_owned(),
        )
        .await?;

    Ok(())
}

// ==========================================
// Lecture & Exercise Tables
// ==========================================

#[derive(DeriveIden)]
enum Lectures {
    Table,
    Id,
    CourseId,
    LectureNumber,
    ScheduleType,
    Date,
    StartTime,
    EndTime,
    Room,
    Building,
    Location,
    Title,
    Notes,
    SlidesCovered,
    GitCommitSha,
    NotesFilePath,
    CreatedAt,
    UpdatedAt,
}

async fn create_lectures_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(Lectures::Table)
                .if_not_exists()
                .col(pk_auto(Lectures::Id))
                .col(integer(Lectures::CourseId))
                .col(integer(Lectures::LectureNumber))
                .col(string(Lectures::ScheduleType))
                .col(string(Lectures::Date))
                .col(string(Lectures::StartTime))
                .col(string(Lectures::EndTime))
                .col(string_null(Lectures::Room))
                .col(string_null(Lectures::Building))
                .col(string_null(Lectures::Location))
                .col(string_null(Lectures::Title))
                .col(string_null(Lectures::Notes))
                .col(string_null(Lectures::SlidesCovered))
                .col(string_null(Lectures::GitCommitSha))
                .col(string_null(Lectures::NotesFilePath))
                .col(timestamp(Lectures::CreatedAt).default(Expr::current_timestamp()))
                .col(timestamp(Lectures::UpdatedAt).default(Expr::current_timestamp()))
                .foreign_key(
                    ForeignKey::create()
                        .from(Lectures::Table, Lectures::CourseId)
                        .to(Courses::Table, Courses::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                )
                .to_owned(),
        )
        .await?;

    // Create indexes
    manager
        .create_index(
            Index::create()
                .name("idx_lectures_course")
                .table(Lectures::Table)
                .col(Lectures::CourseId)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_lectures_date")
                .table(Lectures::Table)
                .col(Lectures::Date)
                .to_owned(),
        )
        .await?;

    // Create unique constraint
    manager
        .create_index(
            Index::create()
                .name("idx_lectures_course_number")
                .table(Lectures::Table)
                .col(Lectures::CourseId)
                .col(Lectures::LectureNumber)
                .unique()
                .to_owned(),
        )
        .await?;

    Ok(())
}

#[derive(DeriveIden)]
enum Slides {
    Table,
    Id,
    CourseId,
    FileName,
    FilePath,
    FileHash,
    SlideNumber,
    Title,
    PageCount,
    IsCovered,
    CoveredInLectureId,
    FileModifiedAt,
    ScannedAt,
    UpdatedAt,
}

async fn create_slides_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(Slides::Table)
                .if_not_exists()
                .col(pk_auto(Slides::Id))
                .col(integer(Slides::CourseId))
                .col(string(Slides::FileName))
                .col(string(Slides::FilePath))
                .col(string_null(Slides::FileHash))
                .col(integer_null(Slides::SlideNumber))
                .col(string_null(Slides::Title))
                .col(integer_null(Slides::PageCount))
                .col(boolean(Slides::IsCovered).default(false))
                .col(integer_null(Slides::CoveredInLectureId))
                .col(timestamp_null(Slides::FileModifiedAt))
                .col(timestamp(Slides::ScannedAt).default(Expr::current_timestamp()))
                .col(timestamp(Slides::UpdatedAt).default(Expr::current_timestamp()))
                .foreign_key(
                    ForeignKey::create()
                        .from(Slides::Table, Slides::CourseId)
                        .to(Courses::Table, Courses::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(Slides::Table, Slides::CoveredInLectureId)
                        .to(Lectures::Table, Lectures::Id)
                        .on_delete(ForeignKeyAction::SetNull)
                )
                .to_owned(),
        )
        .await?;

    // Create indexes
    manager
        .create_index(
            Index::create()
                .name("idx_slides_course")
                .table(Slides::Table)
                .col(Slides::CourseId)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_slides_covered")
                .table(Slides::Table)
                .col(Slides::IsCovered)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_slides_lecture")
                .table(Slides::Table)
                .col(Slides::CoveredInLectureId)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_slides_hash")
                .table(Slides::Table)
                .col(Slides::FileHash)
                .to_owned(),
        )
        .await?;

    // Create unique constraint
    manager
        .create_index(
            Index::create()
                .name("idx_slides_course_filename")
                .table(Slides::Table)
                .col(Slides::CourseId)
                .col(Slides::FileName)
                .unique()
                .to_owned(),
        )
        .await?;

    Ok(())
}

#[derive(DeriveIden)]
enum Exercises {
    Table,
    Id,
    CourseId,
    ExerciseNumber,
    Title,
    AssignmentFilePath,
    SolutionDirectoryPath,
    DueDate,
    SubmissionDate,
    PointsEarned,
    PointsTotal,
    Grade,
    Feedback,
    IsSubmitted,
    IsGraded,
    CreatedAt,
    UpdatedAt,
}

async fn create_exercises_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(Exercises::Table)
                .if_not_exists()
                .col(pk_auto(Exercises::Id))
                .col(integer(Exercises::CourseId))
                .col(integer(Exercises::ExerciseNumber))
                .col(string_null(Exercises::Title))
                .col(string_null(Exercises::AssignmentFilePath))
                .col(string_null(Exercises::SolutionDirectoryPath))
                .col(string_null(Exercises::DueDate))
                .col(string_null(Exercises::SubmissionDate))
                .col(double_null(Exercises::PointsEarned))
                .col(double_null(Exercises::PointsTotal))
                .col(double_null(Exercises::Grade))
                .col(string_null(Exercises::Feedback))
                .col(boolean(Exercises::IsSubmitted).default(false))
                .col(boolean(Exercises::IsGraded).default(false))
                .col(timestamp(Exercises::CreatedAt).default(Expr::current_timestamp()))
                .col(timestamp(Exercises::UpdatedAt).default(Expr::current_timestamp()))
                .foreign_key(
                    ForeignKey::create()
                        .from(Exercises::Table, Exercises::CourseId)
                        .to(Courses::Table, Courses::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                )
                .to_owned(),
        )
        .await?;

    // Create indexes
    manager
        .create_index(
            Index::create()
                .name("idx_exercises_course")
                .table(Exercises::Table)
                .col(Exercises::CourseId)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_exercises_due")
                .table(Exercises::Table)
                .col(Exercises::DueDate)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_exercises_submitted")
                .table(Exercises::Table)
                .col(Exercises::IsSubmitted)
                .to_owned(),
        )
        .await?;

    // Create unique constraint
    manager
        .create_index(
            Index::create()
                .name("idx_exercises_course_number")
                .table(Exercises::Table)
                .col(Exercises::CourseId)
                .col(Exercises::ExerciseNumber)
                .unique()
                .to_owned(),
        )
        .await?;

    Ok(())
}

// ==========================================
// State & Context Tables
// ==========================================

#[derive(DeriveIden)]
enum ActiveCourse {
    Table,
    Id,
    CourseId,
    SemesterId,
    LectureId,
    ActivatedAt,
    UpdatedAt,
}

async fn create_active_course_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(ActiveCourse::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(ActiveCourse::Id)
                        .integer()
                        .not_null()
                        .primary_key()
                )
                .col(integer_null(ActiveCourse::CourseId))
                .col(integer_null(ActiveCourse::SemesterId))
                .col(integer_null(ActiveCourse::LectureId))
                .col(timestamp_null(ActiveCourse::ActivatedAt))
                .col(timestamp(ActiveCourse::UpdatedAt).default(Expr::current_timestamp()))
                .foreign_key(
                    ForeignKey::create()
                        .from(ActiveCourse::Table, ActiveCourse::CourseId)
                        .to(Courses::Table, Courses::Id)
                        .on_delete(ForeignKeyAction::SetNull)
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(ActiveCourse::Table, ActiveCourse::SemesterId)
                        .to(Semesters::Table, Semesters::Id)
                        .on_delete(ForeignKeyAction::SetNull)
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(ActiveCourse::Table, ActiveCourse::LectureId)
                        .to(Lectures::Table, Lectures::Id)
                        .on_delete(ForeignKeyAction::SetNull)
                )
                .to_owned(),
        )
        .await?;

    // Initialize singleton row
    manager
        .get_connection()
        .execute_unprepared("INSERT OR IGNORE INTO active_course (id) VALUES (1)")
        .await?;

    Ok(())
}

#[derive(DeriveIden)]
enum Todos {
    Table,
    Id,
    CourseId,
    LectureId,
    ExerciseId,
    Title,
    Description,
    DueDate,
    Completed,
    CompletedAt,
    AutoClear,
    CreatedAt,
    UpdatedAt,
}

async fn create_todos_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(Todos::Table)
                .if_not_exists()
                .col(pk_auto(Todos::Id))
                .col(integer_null(Todos::CourseId))
                .col(integer_null(Todos::LectureId))
                .col(integer_null(Todos::ExerciseId))
                .col(string(Todos::Title))
                .col(string_null(Todos::Description))
                .col(string_null(Todos::DueDate))
                .col(boolean(Todos::Completed).default(false))
                .col(timestamp_null(Todos::CompletedAt))
                .col(boolean(Todos::AutoClear).default(true))
                .col(timestamp(Todos::CreatedAt).default(Expr::current_timestamp()))
                .col(timestamp(Todos::UpdatedAt).default(Expr::current_timestamp()))
                .foreign_key(
                    ForeignKey::create()
                        .from(Todos::Table, Todos::CourseId)
                        .to(Courses::Table, Courses::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(Todos::Table, Todos::LectureId)
                        .to(Lectures::Table, Lectures::Id)
                        .on_delete(ForeignKeyAction::SetNull)
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(Todos::Table, Todos::ExerciseId)
                        .to(Exercises::Table, Exercises::Id)
                        .on_delete(ForeignKeyAction::SetNull)
                )
                .to_owned(),
        )
        .await?;

    // Create indexes
    manager
        .create_index(
            Index::create()
                .name("idx_todos_course")
                .table(Todos::Table)
                .col(Todos::CourseId)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_todos_completed")
                .table(Todos::Table)
                .col(Todos::Completed)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_todos_due")
                .table(Todos::Table)
                .col(Todos::DueDate)
                .to_owned(),
        )
        .await?;

    Ok(())
}

// ==========================================
// Platform Integration Tables
// ==========================================

#[derive(DeriveIden)]
enum PlatformAccounts {
    Table,
    Id,
    PlatformType,
    PlatformUrl,
    University,
    Username,
    Token,
    IsActive,
    LastSyncAt,
    CreatedAt,
    UpdatedAt,
}

async fn create_platform_accounts_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(PlatformAccounts::Table)
                .if_not_exists()
                .col(pk_auto(PlatformAccounts::Id))
                .col(string(PlatformAccounts::PlatformType))
                .col(string(PlatformAccounts::PlatformUrl))
                .col(string_null(PlatformAccounts::University))
                .col(string_null(PlatformAccounts::Username))
                .col(string_null(PlatformAccounts::Token))
                .col(boolean(PlatformAccounts::IsActive).default(true))
                .col(timestamp_null(PlatformAccounts::LastSyncAt))
                .col(timestamp(PlatformAccounts::CreatedAt).default(Expr::current_timestamp()))
                .col(timestamp(PlatformAccounts::UpdatedAt).default(Expr::current_timestamp()))
                .to_owned(),
        )
        .await?;

    // Create indexes
    manager
        .create_index(
            Index::create()
                .name("idx_platform_accounts_active")
                .table(PlatformAccounts::Table)
                .col(PlatformAccounts::IsActive)
                .to_owned(),
        )
        .await?;

    // Create unique constraint
    manager
        .create_index(
            Index::create()
                .name("idx_platform_accounts_type_url_username")
                .table(PlatformAccounts::Table)
                .col(PlatformAccounts::PlatformType)
                .col(PlatformAccounts::PlatformUrl)
                .col(PlatformAccounts::Username)
                .unique()
                .to_owned(),
        )
        .await?;

    Ok(())
}

#[derive(DeriveIden)]
enum PlatformCourseLinks {
    Table,
    Id,
    CourseId,
    PlatformAccountId,
    PlatformCourseId,
    PlatformCourseUrl,
    AutoSyncExercises,
    AutoSyncSlides,
    AutoSyncAnnouncements,
    LastSyncedAt,
    CreatedAt,
    UpdatedAt,
}

async fn create_platform_course_links_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(PlatformCourseLinks::Table)
                .if_not_exists()
                .col(pk_auto(PlatformCourseLinks::Id))
                .col(integer(PlatformCourseLinks::CourseId))
                .col(integer(PlatformCourseLinks::PlatformAccountId))
                .col(string(PlatformCourseLinks::PlatformCourseId))
                .col(string_null(PlatformCourseLinks::PlatformCourseUrl))
                .col(boolean(PlatformCourseLinks::AutoSyncExercises).default(false))
                .col(boolean(PlatformCourseLinks::AutoSyncSlides).default(false))
                .col(boolean(PlatformCourseLinks::AutoSyncAnnouncements).default(false))
                .col(timestamp_null(PlatformCourseLinks::LastSyncedAt))
                .col(timestamp(PlatformCourseLinks::CreatedAt).default(Expr::current_timestamp()))
                .col(timestamp(PlatformCourseLinks::UpdatedAt).default(Expr::current_timestamp()))
                .foreign_key(
                    ForeignKey::create()
                        .from(PlatformCourseLinks::Table, PlatformCourseLinks::CourseId)
                        .to(Courses::Table, Courses::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(PlatformCourseLinks::Table, PlatformCourseLinks::PlatformAccountId)
                        .to(PlatformAccounts::Table, PlatformAccounts::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                )
                .to_owned(),
        )
        .await?;

    // Create indexes
    manager
        .create_index(
            Index::create()
                .name("idx_platform_links_course")
                .table(PlatformCourseLinks::Table)
                .col(PlatformCourseLinks::CourseId)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_platform_links_account")
                .table(PlatformCourseLinks::Table)
                .col(PlatformCourseLinks::PlatformAccountId)
                .to_owned(),
        )
        .await?;

    // Create unique constraint
    manager
        .create_index(
            Index::create()
                .name("idx_platform_links_course_account")
                .table(PlatformCourseLinks::Table)
                .col(PlatformCourseLinks::CourseId)
                .col(PlatformCourseLinks::PlatformAccountId)
                .unique()
                .to_owned(),
        )
        .await?;

    Ok(())
}

// ==========================================
// Views
// ==========================================

async fn create_views(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    // v_orphaned_courses
    manager.get_connection().execute_unprepared(
        "CREATE VIEW IF NOT EXISTS v_orphaned_courses AS
        SELECT
            c.*,
            s.type || s.number as semester_code
        FROM courses c
        JOIN semesters s ON c.semester_id = s.id
        WHERE c.exists_on_disk = 0
           OR c.toml_exists = 0"
    ).await?;

    // v_current_gpa
    manager.get_connection().execute_unprepared(
        "CREATE VIEW IF NOT EXISTS v_current_gpa AS
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
        GROUP BY d.id, d.name, d.type"
    ).await?;

    // v_degree_progress
    manager.get_connection().execute_unprepared(
        "CREATE VIEW IF NOT EXISTS v_degree_progress AS
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
        ORDER BY d.id, da.display_order"
    ).await?;

    // v_course_categories
    manager.get_connection().execute_unprepared(
        "CREATE VIEW IF NOT EXISTS v_course_categories AS
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
        WHERE c.is_archived = 0"
    ).await?;

    // v_unmapped_courses
    manager.get_connection().execute_unprepared(
        "CREATE VIEW IF NOT EXISTS v_unmapped_courses AS
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
        HAVING COUNT(DISTINCT cpc.area_id) > 0"
    ).await?;

    // v_degree_progress_extended
    manager.get_connection().execute_unprepared(
        "CREATE VIEW IF NOT EXISTS v_degree_progress_extended AS
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
        ORDER BY d.id, da.display_order"
    ).await?;

    Ok(())
}

async fn drop_views(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager.get_connection().execute_unprepared("DROP VIEW IF EXISTS v_degree_progress_extended").await?;
    manager.get_connection().execute_unprepared("DROP VIEW IF EXISTS v_unmapped_courses").await?;
    manager.get_connection().execute_unprepared("DROP VIEW IF EXISTS v_course_categories").await?;
    manager.get_connection().execute_unprepared("DROP VIEW IF EXISTS v_degree_progress").await?;
    manager.get_connection().execute_unprepared("DROP VIEW IF EXISTS v_current_gpa").await?;
    manager.get_connection().execute_unprepared("DROP VIEW IF EXISTS v_orphaned_courses").await?;

    Ok(())
}
