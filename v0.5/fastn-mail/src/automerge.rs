//! Automerge document definitions for mail functionality

#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    fastn_automerge::Reconcile,
    fastn_automerge::Hydrate,
    fastn_automerge::Document,
)]
#[document_path("/-/mails/default")]
pub struct DefaultMail {
    /// Hashed password for authentication
    pub password_hash: String,
    /// Whether the mail service is active
    pub is_active: bool,
    /// Unix timestamp when created
    pub created_at: i64,
}
