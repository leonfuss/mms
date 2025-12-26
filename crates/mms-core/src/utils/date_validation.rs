use crate::error::{MmsError, Result};
use chrono::NaiveDate;

/// Validates German date format (DD.MM.YYYY)
///
/// Accepts flexible formats:
/// - 12.02.2004 (with leading zeros)
/// - 7.8.2003 (without leading zeros)
/// - 01.12.2024 (mixed)
pub fn validate_date_format(date: &str) -> Result<()> {
    parse_german_date(date)?;
    Ok(())
}

/// Parse German date format (DD.MM.YYYY) to NaiveDate
pub fn parse_german_date(date: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(date, "%d.%m.%Y").map_err(MmsError::ChronoParse)
}

/// Validates start_date < end_date
pub fn validate_date_range(start: &Option<String>, end: &Option<String>) -> Result<()> {
    if let (Some(s), Some(e)) = (start, end) {
        let start_date = parse_german_date(s)?;
        let end_date = parse_german_date(e)?;

        if start_date >= end_date {
            return Err(MmsError::InvalidDateRange {
                start: s.clone(),
                end: e.clone(),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_date_format() {
        assert!(validate_date_format("01.10.2024").is_ok());
        assert!(validate_date_format("7.8.2003").is_ok());
        assert!(validate_date_format("99.99.2024").is_err());
        assert!(validate_date_format("invalid").is_err());
    }

    #[test]
    fn test_validate_date_range() {
        let start = Some("01.10.2024".to_string());
        let end = Some("31.03.2025".to_string());
        assert!(validate_date_range(&start, &end).is_ok());

        let bad_start = Some("01.05.2025".to_string());
        assert!(validate_date_range(&bad_start, &end).is_err());
    }
}
