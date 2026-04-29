use mockito::Server;
use scanner_repository::{GithubApiClient, RepositoryPort};

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

    let client = GithubApiClient::new(server.url());
    let snapshot = client
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

    let client = GithubApiClient::new(server.url());
    let err = client.fetch("alice", "missing").await.unwrap_err();
    assert!(matches!(
        err,
        scanner_repository::RepositoryError::NotFound { .. }
    ));
}

#[tokio::test]
async fn returns_rate_limited_on_403() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("GET", "/repos/alice/repo/git/trees/HEAD?recursive=1")
        .with_status(403)
        .create_async()
        .await;

    let client = GithubApiClient::new(server.url());
    let err = client.fetch("alice", "repo").await.unwrap_err();
    assert!(matches!(
        err,
        scanner_repository::RepositoryError::RateLimited
    ));
}
