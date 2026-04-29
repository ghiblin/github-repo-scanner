use crate::{error::RepositoryError, models::RepoSnapshot};

pub trait RepositoryPort {
    fn fetch(
        &self,
        owner: &str,
        name: &str,
    ) -> impl std::future::Future<Output = Result<RepoSnapshot, RepositoryError>> + Send;
}
