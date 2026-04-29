use std::fmt;
use uuid::Uuid;

const PREFIX: &str = "rul";

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct RuleId(String);

#[derive(Debug, thiserror::Error)]
pub enum RuleIdError {
    #[error("rule ID must start with 'rul_', got: {0}")]
    InvalidPrefix(String),
    #[error("rule ID contains an invalid UUID: {0}")]
    InvalidUuid(String),
    #[error("rule ID must use UUID v7, got version {0}")]
    NotV7(u8),
}

impl RuleId {
    #[must_use]
    pub fn new() -> Self {
        Self(format!("{PREFIX}_{}", Uuid::now_v7()))
    }

    /// # Errors
    /// Returns an error if the string does not start with `rul_`, the UUID part is invalid, or the UUID is not version 7.
    pub fn parse(s: &str) -> Result<Self, RuleIdError> {
        let expected_prefix = format!("{PREFIX}_");
        if !s.starts_with(&expected_prefix) {
            return Err(RuleIdError::InvalidPrefix(s.to_owned()));
        }
        let uuid_part = &s[expected_prefix.len()..];
        let uuid = Uuid::parse_str(uuid_part)
            .map_err(|_| RuleIdError::InvalidUuid(uuid_part.to_owned()))?;
        let version = uuid.get_version_num();
        if version != 7 {
            #[allow(clippy::cast_possible_truncation)]
            return Err(RuleIdError::NotV7(version as u8));
        }
        Ok(Self(s.to_owned()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for RuleId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for RuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_produces_rul_prefixed_uuid() {
        let id = RuleId::new();
        assert!(id.as_str().starts_with("rul_"), "got: {}", id.as_str());
    }

    #[test]
    fn parse_accepts_valid_rul_uuid() {
        let raw = "rul_018f1234-abcd-7000-8000-000000000001";
        assert!(RuleId::parse(raw).is_ok());
    }

    #[test]
    fn parse_rejects_wrong_prefix() {
        assert!(RuleId::parse("fnd_018f1234-abcd-7000-8000-000000000001").is_err());
    }

    #[test]
    fn parse_rejects_missing_prefix() {
        assert!(RuleId::parse("018f1234-abcd-7000-8000-000000000001").is_err());
    }

    #[test]
    fn parse_rejects_invalid_uuid() {
        assert!(RuleId::parse("rul_not-a-uuid").is_err());
    }

    #[test]
    fn display_shows_full_string() {
        let id = RuleId::parse("rul_018f1234-abcd-7000-8000-000000000001").unwrap();
        assert_eq!(id.to_string(), "rul_018f1234-abcd-7000-8000-000000000001");
    }
}
