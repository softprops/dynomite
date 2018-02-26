//! Dynomite error types

/// Errors that may result of attribute value conventions
#[derive(Debug, Fail, PartialEq)]
pub enum AttributeError {
  /// Will be returned if an AttributeValue is present, and is of the expected
  /// type but its contents are not well-formatted
  #[fail(display = "Invalid type")]
  InvalidFormat,
  /// Will be returned if provided AttributeValue is not of the expected type
  #[fail(display = "Missing value")]
  InvalidType,
  /// Will be returned if provided attributes does not included an
  /// expected named value
  #[fail(display = "Missing field {}", name)]
  MissingField { name: String },
}
