//! Main Store implementation for email storage operations

use crate::errors::*;
use crate::types::*;

/// Email storage system with hybrid database/file design
#[derive(Debug, Clone)]
pub struct Store {
    /// Path to the account directory
    account_path: std::path::PathBuf,
    /// Mail database connection
    connection: std::sync::Arc<tokio::sync::Mutex<rusqlite::Connection>>,
}

impl Store {
    /// Create new email storage for an account
    pub async fn create(account_path: &std::path::Path) -> Result<Self, StoreCreateError> {
        let mail_db_path = account_path.join("mail.sqlite");
        
        // Create mail directory structure
        crate::database::create_directories(account_path).map_err(|e| {
            StoreCreateError::DirectoryCreationFailed {
                path: account_path.join("mails"),
                source: e,
            }
        })?;

        // Create and connect to database
        let connection = rusqlite::Connection::open(&mail_db_path).map_err(|e| {
            StoreCreateError::DatabaseCreationFailed {
                path: mail_db_path,
                source: e,
            }
        })?;
        
        // Create schema
        crate::database::create_schema(&connection).map_err(|e| {
            StoreCreateError::MigrationFailed { source: e }
        })?;

        Ok(Self {
            account_path: account_path.to_path_buf(),
            connection: std::sync::Arc::new(tokio::sync::Mutex::new(connection)),
        })
    }

    /// Load existing email storage for an account
    pub async fn load(account_path: &std::path::Path) -> Result<Self, StoreLoadError> {
        let mail_db_path = account_path.join("mail.sqlite");
        
        // Check if database exists
        if !mail_db_path.exists() {
            return Err(StoreLoadError::DatabaseNotFound {
                path: mail_db_path,
            });
        }

        // Connect to existing database
        let connection = rusqlite::Connection::open(&mail_db_path).map_err(|e| {
            StoreLoadError::DatabaseOpenFailed {
                path: mail_db_path,
                source: e,
            }
        })?;
        
        // Run migrations (idempotent)
        crate::database::create_schema(&connection).map_err(|e| {
            StoreLoadError::MigrationFailed { source: e }
        })?;

        Ok(Self {
            account_path: account_path.to_path_buf(),
            connection: std::sync::Arc::new(tokio::sync::Mutex::new(connection)),
        })
    }

    /// Create a test store in memory (for testing only)
    pub fn create_test() -> Self {
        let connection = rusqlite::Connection::open_in_memory().unwrap();
        crate::database::create_schema(&connection).unwrap();
        
        Self {
            account_path: std::path::PathBuf::from(":memory:"),
            connection: std::sync::Arc::new(tokio::sync::Mutex::new(connection)),
        }
    }

    // === SMTP Operations ===

    /// SMTP server receives an email and handles delivery (local storage or P2P queuing)
    pub async fn smtp_receive(&self, _raw_message: Vec<u8>) -> Result<String, SmtpReceiveError> {
        // TODO: Parse message headers to extract:
        // - message_id, from_addr, to_addr, cc_addr, bcc_addr, subject
        // - date_sent, content_type, has_attachments
        // - our_alias_used, our_username, their_alias, their_username
        
        // TODO: Generate unique email_id
        // TODO: Determine target folder (INBOX for inbound, Sent for outbound)
        // TODO: Generate file_path (timestamp + message_id based filename)
        // TODO: Store raw_message as .eml file
        // TODO: Insert parsed data into fastn_emails table
        // TODO: If external recipient, insert into fastn_email_delivery table
        
        // For now, return a proper error indicating this is not yet implemented
        // This prevents catastrophic panics while indicating the feature is incomplete
        Err(SmtpReceiveError::MessageParsingFailed {
            message: "SMTP message parsing and storage not yet implemented".to_string(),
        })
    }

    // === P2P Delivery Queue Management ===

    /// Called by periodic task to check outbound queue
    pub async fn get_pending_deliveries(&self) -> Result<Vec<PendingDelivery>, GetPendingDeliveriesError> {
        let conn = self.connection.lock().await;
        
        // Query delivery table for queued emails grouped by recipient
        let mut stmt = conn.prepare(
            "SELECT recipient_id52, COUNT(*) as email_count, MIN(last_attempt) as oldest_date
             FROM fastn_email_delivery 
             WHERE delivery_status = 'queued' OR (delivery_status = 'failed' AND next_retry <= ?)
             GROUP BY recipient_id52"
        ).map_err(|e| GetPendingDeliveriesError::DatabaseQueryFailed { source: e })?;

        let now = chrono::Utc::now().timestamp();
        let rows = stmt.query_map([now], |row| {
            let peer_id52_str: String = row.get(0)?;
            let peer_id52 = std::str::FromStr::from_str(&peer_id52_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "peer_id52".to_string(), rusqlite::types::Type::Text))?;
            
            Ok(PendingDelivery {
                peer_id52,
                email_count: row.get(1)?,
                oldest_email_date: row.get(2)?,
            })
        }).map_err(|e| GetPendingDeliveriesError::DatabaseQueryFailed { source: e })?;

        let mut deliveries = Vec::new();
        for row in rows {
            deliveries.push(row.map_err(|e| GetPendingDeliveriesError::DatabaseQueryFailed { source: e })?);
        }

        Ok(deliveries)
    }

    /// Called when peer contacts us requesting their emails
    pub async fn get_emails_for_peer(&self, peer_id52: &fastn_id52::PublicKey) -> Result<Vec<EmailForDelivery>, GetEmailsForPeerError> {
        let conn = self.connection.lock().await;
        let peer_id52_str = peer_id52.id52();
        
        // Get emails queued for this peer
        let mut stmt = conn.prepare(
            "SELECT e.email_id, e.file_path, e.size_bytes, d.last_attempt
             FROM fastn_emails e
             JOIN fastn_email_delivery d ON e.email_id = d.email_id
             WHERE d.recipient_id52 = ? AND d.delivery_status IN ('queued', 'failed')
             ORDER BY e.date_received ASC"
        ).map_err(|e| GetEmailsForPeerError::DatabaseQueryFailed { source: e })?;

        let rows = stmt.query_map([&peer_id52_str], |row| {
            Ok((
                row.get::<_, String>(0)?,      // email_id
                row.get::<_, String>(1)?,      // file_path
                row.get::<_, usize>(2)?,       // size_bytes
                row.get::<_, Option<i64>>(3)?, // last_attempt
            ))
        }).map_err(|e| GetEmailsForPeerError::DatabaseQueryFailed { source: e })?;

        let mut emails = Vec::new();
        for row in rows {
            let (email_id, file_path, size_bytes, last_attempt) = 
                row.map_err(|e| GetEmailsForPeerError::DatabaseQueryFailed { source: e })?;
            
            // Read the email file
            let full_path = self.account_path.join(&file_path);
            let raw_message = std::fs::read(&full_path).map_err(|e| {
                GetEmailsForPeerError::FileReadFailed {
                    path: full_path,
                    source: e,
                }
            })?;

            emails.push(EmailForDelivery {
                email_id,
                raw_message,
                size_bytes,
                date_queued: last_attempt.unwrap_or(0),
            });
        }

        Ok(emails)
    }

    /// Mark email as delivered to peer
    pub async fn mark_delivered_to_peer(&self, email_id: &str, peer_id52: &fastn_id52::PublicKey) -> Result<(), MarkDeliveredError> {
        let conn = self.connection.lock().await;
        let peer_id52_str = peer_id52.id52();
        
        let updated = conn.execute(
            "UPDATE fastn_email_delivery 
             SET delivery_status = 'delivered', last_attempt = ?
             WHERE email_id = ? AND recipient_id52 = ?",
            rusqlite::params![chrono::Utc::now().timestamp(), email_id, &peer_id52_str]
        ).map_err(|e| MarkDeliveredError::DatabaseUpdateFailed { source: e })?;

        if updated == 0 {
            return Err(MarkDeliveredError::EmailNotFound {
                email_id: email_id.to_string(),
            });
        }

        Ok(())
    }

    // === IMAP Operations ===

    /// List available folders
    pub async fn imap_list_folders(&self) -> Result<Vec<String>, ImapListFoldersError> {
        let mails_path = self.account_path.join("mails/default");
        
        let mut folders = Vec::new();
        let entries = std::fs::read_dir(&mails_path).map_err(|e| {
            ImapListFoldersError::DirectoryScanFailed { source: e }
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                ImapListFoldersError::DirectoryScanFailed { source: e }
            })?;
            
            if entry.path().is_dir()
                && let Some(folder_name) = entry.file_name().to_str() {
                    folders.push(folder_name.to_string());
                }
        }

        Ok(folders)
    }

    /// Select folder and return folder information
    pub async fn imap_select_folder(&self, folder: &str) -> Result<FolderInfo, ImapSelectFolderError> {
        let conn = self.connection.lock().await;
        
        // Check if folder exists
        let folder_path = self.account_path.join("mails/default").join(folder);
        if !folder_path.exists() {
            return Err(ImapSelectFolderError::FolderNotFound {
                folder: folder.to_string(),
            });
        }

        // Query folder statistics
        let exists: u32 = conn.query_row(
            "SELECT COUNT(*) FROM fastn_emails WHERE folder = ? AND is_deleted = 0",
            [folder],
            |row| row.get(0)
        ).map_err(|e| ImapSelectFolderError::DatabaseQueryFailed { source: e })?;

        let recent: u32 = conn.query_row(
            "SELECT COUNT(*) FROM fastn_emails WHERE folder = ? AND is_deleted = 0 AND date_received > ?",
            rusqlite::params![folder, chrono::Utc::now().timestamp() - 86400], // Last 24 hours
            |row| row.get(0)
        ).map_err(|e| ImapSelectFolderError::DatabaseQueryFailed { source: e })?;

        let unseen: Option<u32> = conn.query_row(
            "SELECT COUNT(*) FROM fastn_emails WHERE folder = ? AND is_deleted = 0 AND is_seen = 0",
            [folder],
            |row| row.get(0)
        ).ok();

        Ok(FolderInfo {
            flags: vec![Flag::Seen, Flag::Answered, Flag::Flagged, Flag::Deleted, Flag::Draft],
            exists,
            recent,
            unseen,
            permanent_flags: vec![Flag::Seen, Flag::Answered, Flag::Flagged, Flag::Deleted, Flag::Draft],
            uid_next: Some(exists + 1),
            uid_validity: Some(1), // TODO: Implement proper UID validity
        })
    }

    /// Fetch email message by UID
    pub async fn imap_fetch(&self, folder: &str, uid: u32) -> Result<Vec<u8>, ImapFetchError> {
        let conn = self.connection.lock().await;
        
        // Get file path for the UID
        let file_path: String = conn.query_row(
            "SELECT file_path FROM fastn_emails WHERE folder = ? AND rowid = ? AND is_deleted = 0",
            [folder, &uid.to_string()],
            |row| row.get(0)
        ).map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => ImapFetchError::EmailNotFound { uid },
            _ => ImapFetchError::DatabaseQueryFailed { source: e },
        })?;

        // Read the email file
        let full_path = self.account_path.join(&file_path);
        let raw_message = std::fs::read(&full_path).map_err(|e| {
            ImapFetchError::FileReadFailed {
                path: full_path,
                source: e,
            }
        })?;

        Ok(raw_message)
    }

    /// Search for emails matching criteria
    pub async fn imap_search(&self, folder: &str, criteria: &str) -> Result<Vec<u32>, ImapSearchError> {
        let conn = self.connection.lock().await;
        
        // Basic search implementation - TODO: Parse IMAP search syntax properly
        let sql = match criteria {
            "ALL" => "SELECT rowid FROM fastn_emails WHERE folder = ? AND is_deleted = 0",
            "UNSEEN" => "SELECT rowid FROM fastn_emails WHERE folder = ? AND is_deleted = 0 AND is_seen = 0",
            "SEEN" => "SELECT rowid FROM fastn_emails WHERE folder = ? AND is_deleted = 0 AND is_seen = 1",
            "FLAGGED" => "SELECT rowid FROM fastn_emails WHERE folder = ? AND is_deleted = 0 AND is_flagged = 1",
            _ => return Err(ImapSearchError::InvalidSearchCriteria {
                criteria: criteria.to_string(),
            }),
        };

        let mut stmt = conn.prepare(sql).map_err(|e| {
            ImapSearchError::DatabaseQueryFailed { source: e }
        })?;

        let rows = stmt.query_map([folder], |row| {
            row.get::<_, u32>(0)
        }).map_err(|e| ImapSearchError::DatabaseQueryFailed { source: e })?;

        let mut uids = Vec::new();
        for row in rows {
            uids.push(row.map_err(|e| ImapSearchError::DatabaseQueryFailed { source: e })?);
        }

        Ok(uids)
    }

    /// Store flags for an email
    pub async fn imap_store_flags(&self, folder: &str, uid: u32, flags: &[Flag]) -> Result<(), ImapStoreFlagsError> {
        let conn = self.connection.lock().await;
        
        // Convert flags to boolean fields
        let is_seen = flags.contains(&Flag::Seen);
        let is_flagged = flags.contains(&Flag::Flagged);
        let is_answered = flags.contains(&Flag::Answered);
        let is_deleted = flags.contains(&Flag::Deleted);
        let is_draft = flags.contains(&Flag::Draft);
        
        // Collect custom flags
        let custom_flags: Vec<String> = flags.iter()
            .filter_map(|f| match f {
                Flag::Custom(name) => Some(name.clone()),
                _ => None,
            })
            .collect();
        let custom_flags_json = serde_json::to_string(&custom_flags).unwrap_or_default();

        let updated = conn.execute(
            "UPDATE fastn_emails 
             SET is_seen = ?, is_flagged = ?, is_answered = ?, is_deleted = ?, is_draft = ?, custom_flags = ?
             WHERE folder = ? AND rowid = ?",
            rusqlite::params![is_seen, is_flagged, is_answered, is_deleted, is_draft, custom_flags_json, folder, uid]
        ).map_err(|e| ImapStoreFlagsError::DatabaseUpdateFailed { source: e })?;

        if updated == 0 {
            return Err(ImapStoreFlagsError::EmailNotFound { uid });
        }

        Ok(())
    }

    /// Expunge deleted messages
    pub async fn imap_expunge(&self, folder: &str) -> Result<Vec<u32>, ImapExpungeError> {
        let conn = self.connection.lock().await;
        
        // Get UIDs of deleted messages
        let mut stmt = conn.prepare(
            "SELECT rowid, file_path FROM fastn_emails WHERE folder = ? AND is_deleted = 1"
        ).map_err(|e| ImapExpungeError::DatabaseOperationFailed { source: e })?;

        let rows = stmt.query_map([folder], |row| {
            Ok((row.get::<_, u32>(0)?, row.get::<_, String>(1)?))
        }).map_err(|e| ImapExpungeError::DatabaseOperationFailed { source: e })?;

        let mut expunged_uids = Vec::new();
        for row in rows {
            let (uid, file_path) = row.map_err(|e| {
                ImapExpungeError::DatabaseOperationFailed { source: e }
            })?;
            
            // Delete the email file
            let full_path = self.account_path.join(&file_path);
            if full_path.exists() {
                std::fs::remove_file(&full_path).map_err(|e| {
                    ImapExpungeError::FileDeleteFailed {
                        path: full_path,
                        source: e,
                    }
                })?;
            }
            
            expunged_uids.push(uid);
        }

        // Remove from database
        conn.execute(
            "DELETE FROM fastn_emails WHERE folder = ? AND is_deleted = 1",
            [folder]
        ).map_err(|e| ImapExpungeError::DatabaseOperationFailed { source: e })?;

        Ok(expunged_uids)
    }

    /// Generate email thread tree
    pub async fn imap_thread(&self, _folder: &str, algorithm: &str) -> Result<ThreadTree, ImapThreadError> {
        if algorithm != "REFERENCES" && algorithm != "ORDEREDSUBJECT" {
            return Err(ImapThreadError::UnsupportedAlgorithm {
                algorithm: algorithm.to_string(),
            });
        }

        // TODO: Implement proper threading algorithm
        // For now, return empty thread tree
        Ok(ThreadTree {
            root_message_id: "".to_string(),
            children: Vec::new(),
        })
    }
}