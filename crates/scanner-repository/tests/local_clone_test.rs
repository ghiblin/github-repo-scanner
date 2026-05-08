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
        .args([
            "-C",
            repo_path.to_str().unwrap(),
            "config",
            "user.email",
            "test@test.com",
        ])
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args([
            "-C",
            repo_path.to_str().unwrap(),
            "config",
            "user.name",
            "Test",
        ])
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

#[test]
fn clone_error_does_not_expose_token() {
    let client = LocalCloneClient::new("super-secret-token");
    let err = client
        .fetch_url("https://github.com/nonexistent-org-xyz/no-such-repo-abc123")
        .unwrap_err();
    let msg = err.to_string();
    assert!(
        !msg.contains("super-secret-token"),
        "token must not appear in error message, got: {msg}"
    );
}
