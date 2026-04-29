use base64::{engine::general_purpose::STANDARD, Engine};
use std::path::PathBuf;

use crate::{
    error::RepositoryError,
    models::{FileContent, RepoFile, RepoSnapshot},
    port::RepositoryPort,
};

#[derive(serde::Deserialize)]
struct TreeItem {
    path: String,
    #[serde(rename = "type")]
    item_type: String,
    size: Option<u64>,
}

#[derive(serde::Deserialize)]
struct TreeResponse {
    tree: Vec<TreeItem>,
}

#[derive(serde::Deserialize)]
struct ContentResponse {
    #[serde(rename = "type")]
    content_type: String,
    content: Option<String>,
}

pub struct GithubApiClient {
    base_url: String,
    client: reqwest::Client,
}

impl GithubApiClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: reqwest::Client::new(),
        }
    }
}

impl RepositoryPort for GithubApiClient {
    async fn fetch(&self, owner: &str, name: &str) -> Result<RepoSnapshot, RepositoryError> {
        let tree_url = format!(
            "{}/repos/{owner}/{name}/git/trees/HEAD?recursive=1",
            self.base_url
        );

        let resp = self.client.get(&tree_url).send().await?;

        match resp.status().as_u16() {
            404 => {
                return Err(RepositoryError::NotFound {
                    owner: owner.to_owned(),
                    name: name.to_owned(),
                })
            }
            403 => return Err(RepositoryError::RateLimited),
            _ => {}
        }

        let tree: TreeResponse = resp.error_for_status()?.json().await?;

        let mut files = Vec::new();
        for item in tree.tree.iter().filter(|i| i.item_type == "blob") {
            let content_url = format!(
                "{}/repos/{owner}/{name}/contents/{}",
                self.base_url, item.path
            );
            let content_resp: ContentResponse = self
                .client
                .get(&content_url)
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;

            if content_resp.content_type != "file" {
                continue;
            }

            let raw = content_resp.content.unwrap_or_default().replace('\n', "");
            let bytes = STANDARD.decode(raw.as_bytes()).unwrap_or_default();
            let content = match String::from_utf8(bytes.clone()) {
                Ok(text) => FileContent::Text(text),
                Err(_) => FileContent::Binary(bytes),
            };

            files.push(RepoFile {
                path: PathBuf::from(&item.path),
                content,
                size_bytes: item.size.unwrap_or(0),
            });
        }

        Ok(RepoSnapshot {
            owner: owner.to_owned(),
            name: name.to_owned(),
            files,
        })
    }
}
