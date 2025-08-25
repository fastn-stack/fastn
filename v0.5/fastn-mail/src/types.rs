//! Type definitions for SMTP/IMAP operations aligned with established Rust crates

use serde::{Deserialize, Serialize};

/// IMAP flags aligned with async-imap standards
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Flag {
    /// Message has been read (\Seen)
    Seen,
    /// Message has been answered (\Answered)
    Answered,
    /// Message is flagged for urgent/special attention (\Flagged)
    Flagged,
    /// Message is marked for removal (\Deleted)
    Deleted,
    /// Message has not completed composition (\Draft)
    Draft,
    /// Message is recent (\Recent)
    Recent,
    /// Custom flag
    Custom(String),
}

/// Folder information aligned with async-imap Mailbox struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderInfo {
    /// Defined flags in the mailbox
    pub flags: Vec<Flag>,
    /// Number of messages in mailbox
    pub exists: u32,
    /// Number of messages with \Recent flag
    pub recent: u32,
    /// Sequence number of first unseen message
    pub unseen: Option<u32>,
    /// Flags that can be changed permanently
    pub permanent_flags: Vec<Flag>,
    /// Next UID to be assigned
    pub uid_next: Option<u32>,
    /// UID validity value
    pub uid_validity: Option<u32>,
}

/// Threading information for IMAP THREAD command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadTree {
    /// Root message of the thread
    pub root_message_id: String,
    /// Child threads
    pub children: Vec<ThreadNode>,
}

/// Individual node in email thread tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadNode {
    /// This message's ID
    pub message_id: String,
    /// IMAP UID
    pub uid: u32,
    /// Replies to this message
    pub children: Vec<ThreadNode>,
}

/// Summary of pending deliveries for periodic task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingDelivery {
    /// Which peer needs emails
    pub peer_id52: fastn_id52::PublicKey,
    /// How many emails pending
    pub email_count: usize,
    /// When oldest email was queued
    pub oldest_email_date: i64,
}

/// Email ready for P2P delivery to peer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailForDelivery {
    /// Internal email ID
    pub email_id: String,
    /// Complete RFC 5322 message
    pub raw_message: Vec<u8>,
    /// Message size
    pub size_bytes: usize,
    /// When queued for delivery
    pub date_queued: i64,
}

/// Mail configuration document stored in automerge
#[derive(
    Debug,
    Clone,
    PartialEq,
    Serialize,
    fastn_automerge::Reconcile,
    fastn_automerge::Hydrate,
    fastn_automerge::Document,
)]
#[document_path("/-/mails/default")]
pub struct DefaultMail {
    /// Hashed password for SMTP/IMAP authentication
    pub password_hash: String,
    /// Whether the mail service is active
    pub is_active: bool,
    /// Unix timestamp when created
    pub created_at: i64,
}

impl Flag {
    /// Convert to IMAP string representation
    pub fn to_imap_string(&self) -> String {
        match self {
            Flag::Seen => "\\Seen".to_string(),
            Flag::Answered => "\\Answered".to_string(),
            Flag::Flagged => "\\Flagged".to_string(),
            Flag::Deleted => "\\Deleted".to_string(),
            Flag::Draft => "\\Draft".to_string(),
            Flag::Recent => "\\Recent".to_string(),
            Flag::Custom(name) => name.clone(),
        }
    }

    /// Parse from IMAP string representation
    pub fn from_imap_string(s: &str) -> Self {
        match s {
            "\\Seen" => Flag::Seen,
            "\\Answered" => Flag::Answered,
            "\\Flagged" => Flag::Flagged,
            "\\Deleted" => Flag::Deleted,
            "\\Draft" => Flag::Draft,
            "\\Recent" => Flag::Recent,
            _ => Flag::Custom(s.to_string()),
        }
    }
}