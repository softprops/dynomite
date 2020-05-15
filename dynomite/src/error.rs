//! Dynomite error types
use std::{error::Error, fmt};

/// Errors that may result of attribute value conversions
#[derive(Debug, PartialEq)]
pub enum AttributeError {
    /// Will be returned if an AttributeValue is present, and is of the expected
    /// type but its contents are not well-formatted
    InvalidFormat,
    /// Will be returned if provided AttributeValue is not of the expected type
    InvalidType,
    /// Will be returned if provided attributes does not included an
    /// expected named value
    MissingField {
        /// Name of the field that is missing
        name: String,
    },
}

impl fmt::Display for AttributeError {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            AttributeError::InvalidFormat => write!(f, "Invalid format"),
            AttributeError::InvalidType => write!(f, "Invalid type"),
            AttributeError::MissingField { name } => write!(f, "Missing field {}", name),
        }
    }
}

impl Error for AttributeError {}

#[cfg(test)]
mod tests {
    use super::AttributeError;
    use std::error::Error;

    #[test]
    fn attribute_error_impl_std_error() {
        fn test(_: impl Error) {}
        test(AttributeError::InvalidFormat)
    }

    #[test]
    fn invalid_format_displays() {
        assert_eq!(
            "Invalid format",
            format!("{}", AttributeError::InvalidFormat)
        )
    }

    #[test]
    fn invalid_type_displays() {
        assert_eq!("Invalid type", format!("{}", AttributeError::InvalidType))
    }

    #[test]
    fn missing_field_displays() {
        assert_eq!(
            "Missing field foo",
            format!("{}", AttributeError::MissingField { name: "foo".into() })
        )
    }
}
