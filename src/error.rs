pub type InvmstResult<T> = Result<T, InvmstError>;

#[derive(Debug, thiserror::Error)]
pub enum InvmstError {
    #[error("[Concurrent Error] {0}")]
    ConcurrentError(#[from] ::tokio::task::JoinError),

    #[error("[Config Error] {0}")]
    ConfigError(#[from] ::confy::ConfyError),

    #[error("[Dataframe Error] {0}")]
    DataframeError(#[from] ::polars::error::PolarsError),

    #[error("[HTTP Request Error] {0}")]
    HttpRequestError(#[from] ::reqwest::Error),

    #[error("[HTTP Status Error] {0}")]
    HttpStatusError(String),

    #[error("[Invalid] {1}")]
    Invalid(&'static str, String),

    #[error("[IO Error] {0}")]
    IoError(#[from] std::io::Error),

    #[error("[No Data] {1}")]
    NoData(&'static str, String),

    #[error("[Not Exists] {1}")]
    NotExists(&'static str, String),

    #[error("[Parse Enum Error] {0}")]
    ParseEnumError(#[from] ::strum::ParseError),

    #[error("[Required] {1}")]
    Required(&'static str, String),

    #[error("[Serde JSON Error] {0}")]
    SerdeJsonError(#[from] ::serde_json::Error),

    #[error("[URL Parse Error] {0}")]
    UrlParseError(#[from] url::ParseError),
}
