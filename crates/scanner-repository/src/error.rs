#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("repository not found: {owner}/{name}")]
    NotFound { owner: String, name: String },
    #[error("GitHub API rate limit exceeded")]
    RateLimited,
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("git clone failed: {0}")]
    CloneFailed(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
