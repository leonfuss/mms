use crate::degree::operations::{DegreeInfo, create_degree};
use crate::degree::types::DegreeType;
use crate::error::Result;
use sea_orm::DatabaseConnection;

/// Area definition for DegreeBuilder
#[derive(Debug, Clone)]
pub struct AreaDefinition {
    pub category_name: String,
    pub required_ects: i32,
    pub counts_towards_gpa: bool,
    pub display_order: i32,
}

/// Builder for creating a new degree program
///
/// # Example
/// ```no_run
/// use mms_core::degree::{DegreeBuilder, DegreeType};
/// use sea_orm::DatabaseConnection;
///
/// # async fn example(db: &DatabaseConnection) -> mms_core::error::Result<()> {
/// let degree = DegreeBuilder::new(DegreeType::Bachelor, "Computer Science", "TUM")
///     .with_total_ects(180)
///     .with_start_date("2020-10-01")
///     .with_expected_end_date("2023-09-30")
///     .with_area("Core CS", 60, true)
///     .with_area("Electives", 30, false)
///     .with_area("Thesis", 12, true)
///     .create(db)
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct DegreeBuilder {
    degree_type: DegreeType,
    name: String,
    university: String,
    total_ects_required: i32,
    start_date: Option<String>,
    expected_end_date: Option<String>,
    is_active: bool,
    areas: Vec<AreaDefinition>,
}

impl DegreeBuilder {
    /// Create a new degree builder with required fields
    ///
    /// # Arguments
    /// * `degree_type` - Degree type (Bachelor, Master, or PhD)
    /// * `name` - Degree name (e.g., "Computer Science", "Mathematics")
    /// * `university` - University name (e.g., "TUM", "LMU")
    pub fn new<S1, S2>(degree_type: DegreeType, name: S1, university: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        // Set default ECTS based on degree type
        let default_ects = match degree_type {
            DegreeType::Bachelor => 180,
            DegreeType::Master => 120,
            DegreeType::PhD => 0, // PhDs don't typically have ECTS requirements
        };

        Self {
            degree_type,
            name: name.into(),
            university: university.into(),
            total_ects_required: default_ects,
            start_date: None,
            expected_end_date: None,
            is_active: true,
            areas: Vec::new(),
        }
    }

    /// Set the total ECTS required for this degree
    pub fn with_total_ects(mut self, ects: i32) -> Self {
        self.total_ects_required = ects;
        self
    }

    /// Set the start date (ISO format: YYYY-MM-DD)
    pub fn with_start_date<S: Into<String>>(mut self, date: S) -> Self {
        self.start_date = Some(date.into());
        self
    }

    /// Set the expected end date (ISO format: YYYY-MM-DD)
    pub fn with_expected_end_date<S: Into<String>>(mut self, date: S) -> Self {
        self.expected_end_date = Some(date.into());
        self
    }

    /// Set whether this degree is currently active
    pub fn with_active(mut self, is_active: bool) -> Self {
        self.is_active = is_active;
        self
    }

    /// Add a degree area (category) with ECTS requirements
    ///
    /// # Arguments
    /// * `category_name` - Name of the area (e.g., "Core CS", "Electives")
    /// * `required_ects` - ECTS required for this area
    /// * `counts_towards_gpa` - Whether courses in this area count toward GPA
    pub fn with_area<S: Into<String>>(
        mut self,
        category_name: S,
        required_ects: i32,
        counts_towards_gpa: bool,
    ) -> Self {
        let display_order = self.areas.len() as i32;
        self.areas.push(AreaDefinition {
            category_name: category_name.into(),
            required_ects,
            counts_towards_gpa,
            display_order,
        });
        self
    }

    /// Create the degree (database entry + areas)
    ///
    /// This method will:
    /// 1. Create the degree entry in the database
    /// 2. Create all defined areas
    ///
    /// Returns the created degree info
    pub async fn create(self, db: &DatabaseConnection) -> Result<DegreeInfo> {
        create_degree(
            db,
            self.degree_type,
            self.name,
            self.university,
            self.total_ects_required,
            self.start_date,
            self.expected_end_date,
            self.is_active,
            self.areas,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_basic() {
        let builder = DegreeBuilder::new(DegreeType::Bachelor, "Computer Science", "TUM");
        assert_eq!(builder.degree_type, DegreeType::Bachelor);
        assert_eq!(builder.name, "Computer Science");
        assert_eq!(builder.university, "TUM");
        assert_eq!(builder.total_ects_required, 180);
        assert_eq!(builder.is_active, true);
        assert_eq!(builder.areas.len(), 0);
    }

    #[test]
    fn test_builder_default_ects() {
        let bachelor = DegreeBuilder::new(DegreeType::Bachelor, "CS", "TUM");
        assert_eq!(bachelor.total_ects_required, 180);

        let master = DegreeBuilder::new(DegreeType::Master, "CS", "TUM");
        assert_eq!(master.total_ects_required, 120);

        let phd = DegreeBuilder::new(DegreeType::PhD, "CS", "TUM");
        assert_eq!(phd.total_ects_required, 0);
    }

    #[test]
    fn test_builder_with_details() {
        let builder = DegreeBuilder::new(DegreeType::Master, "Data Science", "LMU")
            .with_total_ects(120)
            .with_start_date("2021-10-01")
            .with_expected_end_date("2023-09-30")
            .with_active(true);

        assert_eq!(builder.degree_type, DegreeType::Master);
        assert_eq!(builder.total_ects_required, 120);
        assert_eq!(builder.start_date, Some("2021-10-01".to_string()));
        assert_eq!(builder.expected_end_date, Some("2023-09-30".to_string()));
        assert_eq!(builder.is_active, true);
    }

    #[test]
    fn test_builder_with_areas() {
        let builder = DegreeBuilder::new(DegreeType::Bachelor, "Computer Science", "TUM")
            .with_area("Core CS", 60, true)
            .with_area("Electives", 30, false)
            .with_area("Thesis", 12, true);

        assert_eq!(builder.areas.len(), 3);
        assert_eq!(builder.areas[0].category_name, "Core CS");
        assert_eq!(builder.areas[0].required_ects, 60);
        assert_eq!(builder.areas[0].counts_towards_gpa, true);
        assert_eq!(builder.areas[0].display_order, 0);

        assert_eq!(builder.areas[1].category_name, "Electives");
        assert_eq!(builder.areas[1].required_ects, 30);
        assert_eq!(builder.areas[1].counts_towards_gpa, false);
        assert_eq!(builder.areas[1].display_order, 1);

        assert_eq!(builder.areas[2].category_name, "Thesis");
        assert_eq!(builder.areas[2].display_order, 2);
    }
}
