//! Dynomite error types

/// Errors that may result of attribute value conventions
#[derive(Debug, Fail, PartialEq)]
pub enum AttributeError {
  /// Will be returned if an AttributeValue is present, and is of the expected
  /// type but its contents are not well-formatted
  #[fail(display = "Invalid format")]
  InvalidFormat,
  /// Will be returned if provided AttributeValue is not of the expected type
  #[fail(display = "Invalid type")]
  InvalidType,
  /// Will be returned if provided attributes does not included an
  /// expected named value
  #[fail(display = "Missing field {}", name)]
  MissingField { name: String },
}

#[cfg(test)]
mod tests {
  use super::AttributeError;
  #[test]
  fn invalid_format_derives() {
    assert_eq!(
      "Invalid format",
      format!("{}", AttributeError::InvalidFormat)
    )
  }

  #[test]
  fn invalid_type_derives() {
    assert_eq!("Invalid type", format!("{}", AttributeError::InvalidType))
  }

  #[test]
  fn missing_field_derives() {
    assert_eq!(
      "Missing field foo",
      format!("{}", AttributeError::MissingField { name: "foo".into() })
    )
  }
}
