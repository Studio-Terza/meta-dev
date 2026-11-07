use graph_query::QueryFailure;
use lint_layout::Failure;

#[derive(Debug, thiserror::Error)]
pub(crate) enum Refusal<'mode> {
    #[error("{0}")]
    Query(&'mode QueryFailure),
    #[error("{0}")]
    Workspace(Failure),
}
