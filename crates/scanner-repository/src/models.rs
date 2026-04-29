use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum FileContent {
    Text(String),
    Binary(Vec<u8>),
}

#[derive(Debug, Clone)]
pub struct RepoFile {
    pub path: PathBuf,
    pub content: FileContent,
    pub size_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct RepoSnapshot {
    pub owner: String,
    pub name: String,
    pub files: Vec<RepoFile>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn file_content_is_text_for_utf8() {
        let content = FileContent::Text("console.log('hi')".to_owned());
        assert!(matches!(content, FileContent::Text(_)));
    }

    #[test]
    fn repo_file_stores_path_and_size() {
        let f = RepoFile {
            path: PathBuf::from("src/index.js"),
            content: FileContent::Text(String::new()),
            size_bytes: 42,
        };
        assert_eq!(f.path.to_str().unwrap(), "src/index.js");
        assert_eq!(f.size_bytes, 42);
    }

    #[test]
    fn repo_snapshot_exposes_files() {
        let snap = RepoSnapshot {
            owner: "alice".to_owned(),
            name: "repo".to_owned(),
            files: vec![],
        };
        assert!(snap.files.is_empty());
    }
}
