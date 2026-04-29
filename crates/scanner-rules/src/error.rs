#[derive(Debug, thiserror::Error)]
pub enum RulesError {
    #[error("failed to read ruleset file: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse ruleset TOML: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("duplicate rule ID: {0}")]
    DuplicateId(String),
}
