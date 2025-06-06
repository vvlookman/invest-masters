pub type InvmstResult<T> = Result<T, InvmstError>;

#[derive(Debug, thiserror::Error)]
pub enum InvmstError {
    #[error("[IO Error] {0}")]
    IoError(#[from] std::io::Error),

    #[error("[Not Exists] {0}")]
    NotExists(String),
}
