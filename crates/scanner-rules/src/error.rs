#[derive(Debug, thiserror::Error)]
pub enum RulesError {
    #[error("failed to read ruleset file: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse ruleset TOML: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("duplicate rule ID: {0}")]
    DuplicateId(String),
    #[error(transparent)]
    InvalidRuleId(#[from] crate::rule_id::RuleIdError),
    #[error("rule ID '{id}' prefix '{actual_prefix}' does not match language '{expected_prefix}'")]
    LanguageMismatch {
        id: String,
        actual_prefix: String,
        expected_prefix: String,
    },
}
