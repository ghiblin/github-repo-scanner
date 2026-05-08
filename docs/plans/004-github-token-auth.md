# GitHub Token Authentication — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add mandatory GitHub PAT support — `Authorization: Bearer <token>` on all API calls, token-embedded HTTPS URL for clone mode, `--token` CLI flag overriding `GITHUB_TOKEN` env var, startup error when neither is present.

**Architecture:** No new crates. Changes confined to `scanner-repository` (error types, `GithubApiClient`, `LocalCloneClient`) and `cli` (argument parsing, token resolution). Constitution updated first.

**Tech Stack:** Rust stable, `reqwest` default headers, `clap` derive, `mockito` for HTTP mocks.

---

## File Map

```
docs/constitution.md                                  update: remove public-only restriction
crates/scanner-repository/src/error.rs                add Unauthorized + Forbidden variants
crates/scanner-repository/src/github_api.rs           require token, set Bearer header, fix 403 logic
crates/scanner-repository/tests/github_api_test.rs    add 401/403 cases, update constructor calls
crates/scanner-repository/src/local_clone.rs          add token field, embed in clone URL
crates/scanner-repository/tests/local_clone_test.rs   add URL-embedding test, update constructor calls
crates/cli/src/main.rs                                add --token flag, resolve_token(), pass to clients
crates/cli/tests/integration_test.rs                  add no-token startup error test
```

---

## Task 1: Update `docs/constitution.md`

**Files:**
- Modify: `docs/constitution.md`

- [ ] **Step 1: Apply change**

In **What It Is Not**, remove:
```
- A tool for scanning private repos (v1 scope: public repos only)
```

In **What It Is**, add:
```
- Capable of scanning private repositories when provided a GitHub token with `repo` scope
```

- [ ] **Step 2: Commit**
```bash
git add docs/constitution.md
git commit -m "docs(constitution): remove public-only restriction, note private repo support"
```

---

## Task 2: `RepositoryError` — add `Unauthorized` and `Forbidden`

**Files:**
- Modify: `crates/scanner-repository/src/error.rs`

- [ ] **Step 1: Write failing tests**

Add at the bottom of `crates/scanner-repository/src/error.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::RepositoryError;

    #[test]
    fn unauthorized_error_message() {
        let err = RepositoryError::Unauthorized;
        assert_eq!(err.to_string(), "authentication failed: token is invalid or expired");
    }

    #[test]
    fn forbidden_error_message() {
        let err = RepositoryError::Forbidden {
            owner: "alice".to_owned(),
            name:  "repo".to_owned(),
        };
        assert_eq!(
            err.to_string(),
            "access denied to alice/repo: token may lack 'repo' scope"
        );
    }
}
```

- [ ] **Step 2: Run to verify failure**
```bash
cargo test -p scanner-repository
```
Expected: compile error — `Unauthorized` and `Forbidden` variants do not exist.

- [ ] **Step 3: Implement**

Replace the full contents of `crates/scanner-repository/src/error.rs`:

```rust
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
        assert_eq!(err.to_string(), "authentication failed: token is invalid or expired");
    }

    #[test]
    fn forbidden_error_message() {
        let err = RepositoryError::Forbidden {
            owner: "alice".to_owned(),
            name:  "repo".to_owned(),
        };
        assert_eq!(
            err.to_string(),
            "access denied to alice/repo: token may lack 'repo' scope"
        );
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**
```bash
cargo test -p scanner-repository
```
Expected: 2 new tests pass; all existing tests still pass.

- [ ] **Step 5: Commit**
```bash
git add crates/scanner-repository/src/error.rs
git commit -m "feat(repository): add Unauthorized and Forbidden error variants"
```

---

## Task 3: `GithubApiClient` — token constructor, `Authorization: Bearer`, 403 disambiguation

**Files:**
- Modify: `crates/scanner-repository/src/github_api.rs`
- Modify: `crates/scanner-repository/tests/github_api_test.rs`

- [ ] **Step 1: Write failing tests**

Replace the full contents of `crates/scanner-repository/tests/github_api_test.rs`:

```rust
use mockito::Server;
use scanner_repository::{GithubApiClient, RepositoryError, RepositoryPort};

// ── helpers ────────────────────────────────────────────────────────────────

fn client(server: &mockito::Server) -> GithubApiClient {
    GithubApiClient::new(server.url(), "test-token")
}

// ── existing tests (updated constructor call) ──────────────────────────────

#[tokio::test]
async fn fetches_repo_with_text_files() {
    let mut server = Server::new_async().await;

    let tree_mock = server
        .mock("GET", "/repos/alice/repo/git/trees/HEAD?recursive=1")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{"tree":[{"path":"package.json","type":"blob","sha":"abc123","size":50}],"truncated":false}"#,
        )
        .create_async()
        .await;

    let content_mock = server
        .mock("GET", "/repos/alice/repo/contents/package.json")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"type":"file","encoding":"base64","content":"e30K"}"#)
        .create_async()
        .await;

    let snapshot = client(&server)
        .fetch("alice", "repo")
        .await
        .expect("fetch should succeed");

    assert_eq!(snapshot.owner, "alice");
    assert_eq!(snapshot.name, "repo");
    assert_eq!(snapshot.files.len(), 1);
    assert_eq!(snapshot.files[0].path.to_str().unwrap(), "package.json");

    tree_mock.assert_async().await;
    content_mock.assert_async().await;
}

#[tokio::test]
async fn returns_not_found_on_404() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("GET", "/repos/alice/missing/git/trees/HEAD?recursive=1")
        .with_status(404)
        .create_async()
        .await;

    let err = client(&server).fetch("alice", "missing").await.unwrap_err();
    assert!(matches!(err, RepositoryError::NotFound { .. }));
}

#[tokio::test]
async fn returns_rate_limited_on_403_with_zero_remaining() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("GET", "/repos/alice/repo/git/trees/HEAD?recursive=1")
        .with_status(403)
        .with_header("x-ratelimit-remaining", "0")
        .create_async()
        .await;

    let err = client(&server).fetch("alice", "repo").await.unwrap_err();
    assert!(matches!(err, RepositoryError::RateLimited));
}

// ── new auth tests ──────────────────────────────────────────────────────────

#[tokio::test]
async fn returns_unauthorized_on_401() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("GET", "/repos/alice/repo/git/trees/HEAD?recursive=1")
        .with_status(401)
        .create_async()
        .await;

    let err = client(&server).fetch("alice", "repo").await.unwrap_err();
    assert!(matches!(err, RepositoryError::Unauthorized));
}

#[tokio::test]
async fn returns_forbidden_on_403_without_rate_limit_header() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("GET", "/repos/alice/private/git/trees/HEAD?recursive=1")
        .with_status(403)
        .create_async()
        .await;

    let err = client(&server).fetch("alice", "private").await.unwrap_err();
    assert!(matches!(err, RepositoryError::Forbidden { .. }));
}

#[tokio::test]
async fn returns_forbidden_on_403_with_nonzero_remaining() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("GET", "/repos/alice/private/git/trees/HEAD?recursive=1")
        .with_status(403)
        .with_header("x-ratelimit-remaining", "4999")
        .create_async()
        .await;

    let err = client(&server).fetch("alice", "private").await.unwrap_err();
    assert!(matches!(err, RepositoryError::Forbidden { .. }));
}
```

- [ ] **Step 2: Run to verify failure**
```bash
cargo test -p scanner-repository
```
Expected: compile error — `GithubApiClient::new` still takes one argument; `RepositoryError::Unauthorized` match arms refer to variants that now exist but the client never returns them.

- [ ] **Step 3: Implement**

Replace the full contents of `crates/scanner-repository/src/github_api.rs`:

```rust
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
    pub fn new(base_url: impl Into<String>, token: impl Into<String>) -> Self {
        let auth_value = format!("Bearer {}", token.into());
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&auth_value)
                .expect("token must not contain invalid header characters"),
        );
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("failed to build reqwest client");
        Self {
            base_url: base_url.into(),
            client,
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
            401 => return Err(RepositoryError::Unauthorized),
            403 => {
                let remaining = resp
                    .headers()
                    .get("x-ratelimit-remaining")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok());
                return Err(if remaining == Some(0) {
                    RepositoryError::RateLimited
                } else {
                    RepositoryError::Forbidden {
                        owner: owner.to_owned(),
                        name: name.to_owned(),
                    }
                });
            }
            404 => {
                return Err(RepositoryError::NotFound {
                    owner: owner.to_owned(),
                    name: name.to_owned(),
                })
            }
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
```

- [ ] **Step 4: Run tests to verify they pass**
```bash
cargo test -p scanner-repository
```
Expected: all 8 tests pass (5 new + 3 updated existing).

- [ ] **Step 5: Commit**
```bash
git add crates/scanner-repository/src/github_api.rs \
        crates/scanner-repository/tests/github_api_test.rs
git commit -m "feat(repository): require token, set Authorization: Bearer, fix 403 disambiguation"
```

---

## Task 4: `LocalCloneClient` — token field, embed in clone URL

**Files:**
- Modify: `crates/scanner-repository/src/local_clone.rs`
- Modify: `crates/scanner-repository/tests/local_clone_test.rs`

- [ ] **Step 1: Write failing tests**

Replace the full contents of `crates/scanner-repository/tests/local_clone_test.rs`:

```rust
use scanner_repository::LocalCloneClient;
use std::fs;
use tempfile::TempDir;

fn create_fake_repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    let repo_path = dir.path();
    std::process::Command::new("git")
        .args(["init", repo_path.to_str().unwrap()])
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["-C", repo_path.to_str().unwrap(), "config", "user.email", "test@test.com"])
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["-C", repo_path.to_str().unwrap(), "config", "user.name", "Test"])
        .output()
        .unwrap();
    fs::write(repo_path.join("package.json"), r#"{"name":"test"}"#).unwrap();
    std::process::Command::new("git")
        .args(["-C", repo_path.to_str().unwrap(), "add", "."])
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["-C", repo_path.to_str().unwrap(), "commit", "-m", "init"])
        .output()
        .unwrap();
    dir
}

#[test]
fn clones_local_repo_and_reads_files() {
    let fake_repo = create_fake_repo();
    let url = format!("file://{}", fake_repo.path().display());
    let client = LocalCloneClient::new("test-token");
    let snapshot = client.fetch_url(&url).expect("clone should succeed");
    assert_eq!(snapshot.files.len(), 1);
    assert_eq!(snapshot.files[0].path.to_str().unwrap(), "package.json");
}

#[test]
fn returns_clone_failed_for_invalid_url() {
    let client = LocalCloneClient::new("test-token");
    let err = client
        .fetch_url("https://github.com/nonexistent-org-xyz/no-such-repo-abc123")
        .unwrap_err();
    assert!(matches!(
        err,
        scanner_repository::RepositoryError::CloneFailed(_)
    ));
}

#[test]
fn embeds_token_in_github_clone_url() {
    let client = LocalCloneClient::new("my-secret-token");
    let auth_url = client.authenticated_url("https://github.com/owner/repo");
    assert_eq!(
        auth_url,
        "https://x-access-token:my-secret-token@github.com/owner/repo"
    );
}

#[test]
fn leaves_non_github_url_unchanged() {
    let client = LocalCloneClient::new("my-secret-token");
    let url = "file:///tmp/local-repo";
    assert_eq!(client.authenticated_url(url), url);
}
```

- [ ] **Step 2: Run to verify failure**
```bash
cargo test -p scanner-repository
```
Expected: compile error — `LocalCloneClient::new` takes no arguments; `authenticated_url` method does not exist.

- [ ] **Step 3: Implement**

Replace the full contents of `crates/scanner-repository/src/local_clone.rs`:

```rust
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
        Self { token: token.into() }
    }

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
```

- [ ] **Step 4: Run tests to verify they pass**
```bash
cargo test -p scanner-repository
```
Expected: all tests pass (4 in local_clone + all api tests).

- [ ] **Step 5: Commit**
```bash
git add crates/scanner-repository/src/local_clone.rs \
        crates/scanner-repository/tests/local_clone_test.rs
git commit -m "feat(repository): add token to LocalCloneClient, embed in GitHub clone URL"
```

---

## Task 5: `cli` — `--token` flag + `GITHUB_TOKEN` env var resolution

**Files:**
- Modify: `crates/cli/src/main.rs`
- Modify: `crates/cli/tests/integration_test.rs`

- [ ] **Step 1: Write failing tests**

Add to the bottom of `crates/cli/tests/integration_test.rs`:

```rust
#[test]
fn exits_with_error_when_no_token_provided() {
    let output = bin()
        .arg("https://github.com/owner/repo")
        .env_remove("GITHUB_TOKEN")
        .output()
        .expect("binary must exist");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("GitHub token is required"),
        "expected token-required error, got: {stderr}"
    );
}

#[test]
fn accepts_token_from_env_var() {
    let output = bin()
        .arg("https://github.com/owner/repo")
        .env("GITHUB_TOKEN", "env-token-value")
        .output()
        .expect("binary must exist");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("GitHub token is required"),
        "should not show token error when GITHUB_TOKEN is set, got: {stderr}"
    );
}

#[test]
fn token_flag_takes_precedence_over_env_var() {
    let output = bin()
        .arg("https://github.com/owner/repo")
        .arg("--token")
        .arg("flag-token-value")
        .env("GITHUB_TOKEN", "env-token-value")
        .output()
        .expect("binary must exist");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("GitHub token is required"),
        "should not show token error when --token is set, got: {stderr}"
    );
}
```

- [ ] **Step 2: Run to verify failure**
```bash
cargo test -p cli
```
Expected: `exits_with_error_when_no_token_provided` fails — binary currently succeeds (no token check); `accepts_token_from_env_var` and `token_flag_takes_precedence_over_env_var` may also fail since `GithubApiClient::new` now requires two arguments (compile error).

- [ ] **Step 3: Implement**

Replace the full contents of `crates/cli/src/main.rs`:

```rust
use std::{path::PathBuf, process, sync::Arc};

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use scanner_analysis::{Analyzer, NodeJsAnalyzer, VsCodeAnalyzer, Verdict};
use scanner_report::render;
use scanner_repository::{GithubApiClient, LocalCloneClient, RepositoryPort};
use scanner_rules::loader::load;

#[derive(Parser)]
#[command(
    name = "repo-scanner",
    about = "Scan a GitHub repo for malware patterns before cloning"
)]
struct Args {
    /// GitHub repository URL (e.g. `https://github.com/owner/repo`)
    github_url: String,

    /// Clone repo locally for deeper analysis (default: API only)
    #[arg(long)]
    clone: bool,

    /// Path to custom TOML ruleset
    #[arg(long, value_name = "FILE")]
    rules: Option<PathBuf>,

    /// GitHub personal access token (overrides GITHUB_TOKEN env var)
    #[arg(long, value_name = "TOKEN")]
    token: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let exit_code = run(args).await?;
    process::exit(exit_code);
}

async fn run(args: Args) -> anyhow::Result<i32> {
    let token = resolve_token(args.token)?;
    let (owner, name) = parse_github_url(&args.github_url)?;

    let rules_path = args.rules.unwrap_or_else(|| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../rules/nodejs.toml")
    });
    let vscode_rules_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../rules/vscode.toml");
    let ruleset = Arc::new(load(&rules_path)?);
    let vscode_ruleset = Arc::new(load(&vscode_rules_path)?);

    let snapshot = fetch_snapshot(&args.github_url, &owner, &name, args.clone, &token).await?;

    let mut findings = NodeJsAnalyzer.analyze(&snapshot, &ruleset);
    findings.extend(VsCodeAnalyzer.analyze(&snapshot, &vscode_ruleset));
    let verdict = Verdict::from_findings(&findings);

    print!("{}", render(&findings, &verdict, true));

    Ok(match verdict {
        Verdict::Safe => 0,
        Verdict::Suspicious => 1,
        Verdict::Dangerous => 2,
    })
}

fn resolve_token(flag: Option<String>) -> anyhow::Result<String> {
    if let Some(t) = flag {
        return Ok(t);
    }
    if let Ok(t) = std::env::var("GITHUB_TOKEN") {
        if !t.is_empty() {
            return Ok(t);
        }
    }
    anyhow::bail!("GitHub token is required. Set GITHUB_TOKEN or pass --token <TOKEN>.")
}

async fn fetch_snapshot(
    url: &str,
    owner: &str,
    name: &str,
    clone: bool,
    token: &str,
) -> anyhow::Result<scanner_repository::RepoSnapshot> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner} {msg}")
            .unwrap(),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    let snapshot = if clone {
        pb.set_message(format!("Cloning {owner}/{name}..."));
        LocalCloneClient::new(token).fetch_url(url)?
    } else {
        pb.set_message(format!("Fetching {owner}/{name}..."));
        GithubApiClient::new("https://api.github.com", token)
            .fetch(owner, name)
            .await?
    };
    pb.finish_and_clear();
    Ok(snapshot)
}

fn parse_github_url(url: &str) -> anyhow::Result<(String, String)> {
    let url = url.trim_end_matches('/');
    let parts: Vec<&str> = url.rsplitn(3, '/').collect();
    if parts.len() < 3 || !parts[2].contains("github.com") {
        anyhow::bail!("invalid GitHub URL — expected https://github.com/owner/repo");
    }
    Ok((parts[1].to_owned(), parts[0].to_owned()))
}
```

- [ ] **Step 4: Run tests to verify they pass**
```bash
cargo build && cargo test -p cli
```
Expected: all tests pass including the 3 new token-resolution tests.

- [ ] **Step 5: Lint and format**
```bash
cargo fmt && cargo clippy -- -D warnings
```

- [ ] **Step 6: Commit**
```bash
git add crates/cli/src/main.rs \
        crates/cli/tests/integration_test.rs
git commit -m "feat(cli): add --token flag and GITHUB_TOKEN env var resolution"
```
