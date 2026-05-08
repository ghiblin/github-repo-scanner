use crate::models::Language;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct RuleId(String);

#[derive(Debug, thiserror::Error)]
pub enum RuleIdError {
    #[error("invalid rule ID format '{id}': expected <LANG>-<NNN> with a known language prefix")]
    InvalidRuleId { id: String },
}

impl RuleId {
    /// # Errors
    /// Returns `InvalidRuleId` if `s` does not match `<LANG>-<NNN>` with a known language prefix.
    pub fn parse(s: &str) -> Result<Self, RuleIdError> {
        let err = || RuleIdError::InvalidRuleId { id: s.to_owned() };
        let (prefix, number) = s.split_once('-').ok_or_else(err)?;
        if prefix.is_empty() || !prefix.chars().all(|c| c.is_ascii_uppercase()) {
            return Err(err());
        }
        if number.len() != 3 || !number.chars().all(|c| c.is_ascii_digit()) {
            return Err(err());
        }
        if Language::from_prefix(prefix).is_none() {
            return Err(err());
        }
        Ok(Self(s.to_owned()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
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
    fn parse_accepts_valid_node_id() {
        assert!(RuleId::parse("NODE-001").is_ok());
    }

    #[test]
    fn parse_accepts_valid_any_id() {
        assert!(RuleId::parse("ANY-042").is_ok());
    }

    #[test]
    fn parse_rejects_lowercase_prefix() {
        assert!(RuleId::parse("node-001").is_err());
    }

    #[test]
    fn parse_rejects_short_number() {
        assert!(RuleId::parse("NODE-1").is_err());
    }

    #[test]
    fn parse_rejects_missing_hyphen() {
        assert!(RuleId::parse("NODE001").is_err());
    }

    #[test]
    fn parse_rejects_unknown_prefix() {
        assert!(RuleId::parse("FOO-001").is_err());
    }

    #[test]
    fn parse_rejects_empty_string() {
        assert!(RuleId::parse("").is_err());
    }

    #[test]
    fn display_shows_full_string() {
        let id = RuleId::parse("NODE-001").unwrap();
        assert_eq!(id.to_string(), "NODE-001");
    }

    #[test]
    fn as_str_returns_inner_string() {
        let id = RuleId::parse("NODE-001").unwrap();
        assert_eq!(id.as_str(), "NODE-001");
    }

    #[test]
    fn parse_accepts_valid_vscode_id() {
        assert!(RuleId::parse("VSCODE-001").is_ok());
    }
}
