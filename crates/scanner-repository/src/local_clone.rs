use crate::{
    error::RepositoryError,
    models::{FileContent, RepoFile, RepoSnapshot},
    port::RepositoryPort,
};

pub struct LocalCloneClient {
    token: String,
}

impl LocalCloneClient {
    #[must_use]
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
        }
    }

    #[must_use]
    pub fn authenticated_url(&self, url: &str) -> String {
        if url.contains("github.com") {
            url.replacen(
                "https://",
                &format!("https://x-access-token:{}@", self.token),
                1,
            )
        } else {
            url.to_owned()
        }
    }

    /// # Errors
    /// Returns an error if the git clone fails, directory walking fails, or I/O errors occur.
    pub fn fetch_url(&self, url: &str) -> Result<RepoSnapshot, RepositoryError> {
        let auth_url = self.authenticated_url(url);
        let tmp = tempfile::TempDir::new()?;
        let output = std::process::Command::new("git")
            .args([
                "clone",
                "--depth=1",
                &auth_url,
                tmp.path().to_str().unwrap_or("."),
            ])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(RepositoryError::CloneFailed(stderr));
        }

        let mut files = Vec::new();
        walk_dir(tmp.path(), tmp.path(), &mut files)?;

        Ok(RepoSnapshot {
            owner: String::new(),
            name: url.split('/').next_back().unwrap_or("repo").to_owned(),
            files,
        })
    }
}

fn walk_dir(
    base: &std::path::Path,
    current: &std::path::Path,
    files: &mut Vec<RepoFile>,
) -> Result<(), RepositoryError> {
    for entry in std::fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|n| n == ".git") {
                continue;
            }
            walk_dir(base, &path, files)?;
        } else {
            let rel = path.strip_prefix(base).unwrap_or(&path).to_owned();
            let size_bytes = entry.metadata()?.len();
            let bytes = std::fs::read(&path)?;
            let content = match String::from_utf8(bytes.clone()) {
                Ok(text) => FileContent::Text(text),
                Err(_) => FileContent::Binary(bytes),
            };
            files.push(RepoFile {
                path: rel,
                content,
                size_bytes,
            });
        }
    }
    Ok(())
}

impl RepositoryPort for LocalCloneClient {
    async fn fetch(&self, _owner: &str, name: &str) -> Result<RepoSnapshot, RepositoryError> {
        self.fetch_url(name)
    }
}
