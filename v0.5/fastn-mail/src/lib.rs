//! # fastn-mail
//!
//! Complete email handling and storage system for FASTN accounts with full SMTP/IMAP compatibility.
//!
//! This crate provides a hybrid storage system that combines database indexing with file-based
//! storage to support real-world email clients while enabling fast IMAP operations.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use fastn_mail::Store;
//! use std::path::Path;
//!
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     let account_path = Path::new("/path/to/account");
//!     
//!     // Create new email storage for an account
//!     let store = Store::create(&account_path).await?;
//!     
//!     // Load existing email storage
//!     let store = Store::load(&account_path).await?;
//!     
//!     // SMTP operations
//!     let raw_message = vec![]; // RFC 5322 email bytes
//!     let email_id = store.smtp_receive(raw_message).await?;
//!     
//!     // IMAP operations
//!     let folder_info = store.imap_select_folder("INBOX").await?;
//!     let message = store.imap_fetch("INBOX", 1).await?;
//!     
//!     // P2P delivery
//!     let pending = store.get_pending_deliveries().await?;
//!     let peer_id52 = fastn_id52::SecretKey::generate().public_key();
//!     let emails = store.get_emails_for_peer(&peer_id52).await?;
//!     
//!     Ok(())
//! }
//! ```

extern crate self as fastn_mail;

#[cfg(feature = "cli")]
pub mod cli;
mod database;
mod errors;
mod p2p_receive_email;
mod smtp_receive;
mod store;
mod types;
mod utils;

// Re-export main types
pub use errors::{
    GetEmailsForPeerError, GetPendingDeliveriesError, ImapExpungeError, ImapFetchError,
    ImapListFoldersError, ImapSearchError, ImapSelectFolderError, ImapStoreFlagsError,
    ImapThreadError, MarkDeliveredError, SmtpReceiveError, StoreCreateError, StoreLoadError,
};
pub use store::Store;
pub use types::{
    DefaultMail, EmailAddress, EmailForDelivery, Flag, FolderInfo, ParsedEmail, PendingDelivery,
    ThreadNode, ThreadTree,
};
