#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("HttpError: {}", _0)]
    HttpError(#[from] reqwest::Error),

    #[error("IoError: {}", _0)]
    IoError(#[from] std::io::Error),

    #[error("ZipError: {}", _0)]
    ZipError(#[from] zip::result::ZipError),

    #[error("SerdeJsonError: {}", _0)]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("FTDError: {}", _0)]
    FTDError(#[from] ftd::p1::Error),

    #[error("FTDP1Error: {}", _0)]
    FTDP1Error(#[from] ftd::p11::Error),

    #[error("FTDExecError: {}", _0)]
    FTDExecError(#[from] ftd::executor::Error),

    #[error("FTDInterpreterError: {}", _0)]
    FTDInterpreterError(#[from] ftd::interpreter2::Error),

    #[error("FTDHtmlError: {}", _0)]
    FTDHtmlError(#[from] ftd::html1::Error),

    #[error("IgnoreError: {}", _0)]
    IgnoreError(#[from] ignore::Error),

    #[error("FromPathBufError: {}", _0)]
    FromPathBufError(#[from] camino::FromPathBufError),

    #[error("StripPrefixError: {}", _0)]
    StripPrefixError(#[from] std::path::StripPrefixError),

    #[error("SitemapParseError: {}", _0)]
    SitemapParseError(#[from] fpm::sitemap::ParseError),

    #[error("URLParseError: {}", _0)]
    UrlParseError(#[from] url::ParseError),

    #[error("UTF8Error: {}", _0)]
    UTF8Error(#[from] std::string::FromUtf8Error),

    #[error("ParseIntError: {}", _0)]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("ParseFloatError: {}", _0)]
    ParseFloatError(#[from] std::num::ParseFloatError),

    #[error("ParseBoolError: {}", _0)]
    ParseBoolError(#[from] std::str::ParseBoolError),

    #[error("APIResponseError: {}", _0)]
    APIResponseError(String),

    #[error("PackageError: {message}")]
    PackageError { message: String },

    #[error("UsageError: {message}")]
    UsageError { message: String },

    #[error("GenericError: {}", _0)]
    GenericError(String),

    #[error("GroupNotFound: id: {id}, {message}")]
    GroupNotFound { id: String, message: String },

    #[error("CRAboutNotFound CR#{cr_number}: {message}")]
    CRAboutNotFound { message: String, cr_number: usize },

    #[error("QueryPayloadError: {}", _0)]
    QueryPayloadError(#[from] actix_web::error::QueryPayloadError),

    #[error("TokioMPSCError1: {}", _0)]
    TokioMPSCError1(#[from] tokio::sync::mpsc::error::SendError<fpm::watcher::WatcherSender>),

    #[error("TokioMPSCError2: {}", _0)]
    TokioMPSCError2(#[from] tokio::sync::mpsc::error::SendError<usize>),
}
