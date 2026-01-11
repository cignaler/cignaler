use thiserror::Error;

#[derive(Error, Debug)]
pub enum CignalerError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Connection pool error: {0}")]
    Pool(#[from] r2d2::Error),

    #[error("GitLab API error: {0}")]
    GitLabApi(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unsupported provider: {0}")]
    UnsupportedProvider(String),

    #[error("Rate limited, retry after {0} seconds")]
    RateLimited(u64),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),
}

impl From<CignalerError> for String {
    fn from(err: CignalerError) -> Self {
        err.to_string()
    }
}

pub type Result<T> = std::result::Result<T, CignalerError>;
