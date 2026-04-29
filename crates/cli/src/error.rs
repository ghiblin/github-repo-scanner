#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum ScanError {
    #[error("repository error: {0}")]
    Repository(#[from] scanner_repository::RepositoryError),
    #[error("rules error: {0}")]
    Rules(#[from] scanner_rules::RulesError),
}
