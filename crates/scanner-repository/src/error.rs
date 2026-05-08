#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("repository not found: {owner}/{name}")]
    NotFound { owner: String, name: String },
    #[error("GitHub API rate limit exceeded")]
    RateLimited,
    #[error("authentication failed: token is invalid or expired")]
    Unauthorized,
    #[error("access denied to {owner}/{name}: token may lack 'repo' scope")]
    Forbidden { owner: String, name: String },
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("git clone failed: {0}")]
    CloneFailed(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::RepositoryError;

    #[test]
    fn unauthorized_error_message() {
        let err = RepositoryError::Unauthorized;
        assert_eq!(
            err.to_string(),
            "authentication failed: token is invalid or expired"
        );
    }

    #[test]
    fn forbidden_error_message() {
        let err = RepositoryError::Forbidden {
            owner: "alice".to_owned(),
            name: "repo".to_owned(),
        };
        assert_eq!(
            err.to_string(),
            "access denied to alice/repo: token may lack 'repo' scope"
        );
    }
}
