use thiserror::Error as Error_;

#[derive(Error_, Debug)]
pub enum Error {
    #[error("invalid header {found:?}")]
    InvalidCode { found: String },
}
