pub type InvmstResult<T> = Result<T, InvmstError>;

#[derive(Debug, thiserror::Error)]
pub enum InvmstError {
    #[error("[Dataframe Error] {0}")]
    DataframeError(#[from] polars::error::PolarsError),

    #[error("[Invalid] {1}")]
    Invalid(&'static str, String),

    #[error("[IO Error] {0}")]
    IoError(#[from] std::io::Error),

    #[error("[Not Exists] {1}")]
    NotExists(&'static str, String),

    #[error("[Required] {1}")]
    Required(&'static str, String),
}
