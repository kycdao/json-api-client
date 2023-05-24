use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("client error: '{0}'")]
    ClientError(String),

    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),

    #[error(transparent)]
    UrlError(#[from] url::ParseError),

    #[error(transparent)]
    JsonError(#[from] serde_json::Error),

    #[error(transparent)]
    Oauth2ExecuteError(#[from] oauth2::ExecuteError),
}

pub type Result<T> = std::result::Result<T, Error>;
