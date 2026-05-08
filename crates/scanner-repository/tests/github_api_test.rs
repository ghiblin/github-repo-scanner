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
