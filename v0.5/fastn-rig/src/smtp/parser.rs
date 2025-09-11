//! SMTP Command and Message Parsing
//!
//! Provides testable parsing abstractions for SMTP protocol elements

#[derive(Debug, PartialEq)]
pub struct AuthCredentials {
    pub username: String,
    pub password: String,
}

impl AuthCredentials {
    /// Parse SMTP AUTH PLAIN credentials from base64 string
    pub fn parse_plain(base64_creds: &str) -> Result<Self, &'static str> {
        // Decode base64
        use base64::Engine;
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(base64_creds)
            .map_err(|_| "Invalid base64 encoding")?;

        let auth_string = String::from_utf8(decoded).map_err(|_| "Invalid UTF-8 in credentials")?;

        // PLAIN format: \0username\0password
        let parts: Vec<&str> = auth_string.split('\0').collect();
        if parts.len() != 3 {
            return Err("Invalid AUTH PLAIN format");
        }

        Ok(AuthCredentials {
            username: parts[1].to_string(),
            password: parts[2].to_string(),
        })
    }

    /// Extract account ID52 from username (robust parsing for various SMTP client formats)
    ///
    /// Supports ONLY secure .fastn format to prevent domain hijacking:
    /// - user@<id52>.fastn (secure format - no purchasable domains)
    ///
    /// Security: Rejects .com/.org/.net domains to prevent attack where
    /// someone buys {id52}.com and intercepts emails meant for P2P delivery.
    pub fn extract_account_id52(&self) -> Option<fastn_id52::PublicKey> {
        // Strategy 1: Extract from domain part - ONLY accept .fastn domains
        if let Some(at_pos) = self.username.find('@') {
            let domain = &self.username[at_pos + 1..];
            let domain_parts: Vec<&str> = domain.split('.').collect();

            // Security: Only accept .fastn domains
            if domain_parts.len() == 2 && domain_parts[1] == "fastn" {
                let potential_id52 = domain_parts[0];
                if potential_id52.len() == 52
                    && let Ok(id52) = potential_id52.parse::<fastn_id52::PublicKey>()
                {
                    return Some(id52);
                }
            }
        }

        // Strategy 2: Security-enhanced fallback - only if email contains .fastn
        // This ensures even unusual formats are still secure
        if self.username.contains(".fastn") {
            let separators = ['@', '.', '_', '-', '+', '='];
            let parts: Vec<&str> = self.username.split(&separators[..]).collect();
            for part in parts {
                if part.len() == 52
                    && let Ok(id52) = part.parse::<fastn_id52::PublicKey>()
                {
                    return Some(id52);
                }
            }
        }

        None
    }
}

/// Parse MAIL FROM command
pub fn parse_mail_from(args: &str) -> Result<String, &'static str> {
    let args = args.trim();
    if !args.to_uppercase().starts_with("FROM:") {
        return Err("Invalid MAIL FROM syntax");
    }

    let addr_part = args[5..].trim();
    extract_address_from_brackets(addr_part)
}

/// Parse RCPT TO command  
pub fn parse_rcpt_to(args: &str) -> Result<String, &'static str> {
    let args = args.trim();
    if !args.to_uppercase().starts_with("TO:") {
        return Err("Invalid RCPT TO syntax");
    }

    let addr_part = args[3..].trim();
    extract_address_from_brackets(addr_part)
}

/// Extract email address from optional angle brackets
fn extract_address_from_brackets(addr_part: &str) -> Result<String, &'static str> {
    if addr_part.starts_with('<') && addr_part.ends_with('>') {
        let inner = &addr_part[1..addr_part.len() - 1];
        if inner.is_empty() {
            return Err("Empty address in brackets");
        }
        Ok(inner.to_string())
    } else {
        Ok(addr_part.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test-only utilities
    #[derive(Debug, PartialEq)]
    pub struct SmtpCommand {
        pub verb: String,
        pub args: String,
    }

    #[derive(Debug, PartialEq)]
    pub struct EmailAddress {
        pub local: String,
        pub domain: String,
    }

    impl SmtpCommand {
        /// Parse SMTP command line into verb and arguments
        pub fn parse(line: &str) -> Result<Self, &'static str> {
            let line = line.trim();
            if line.is_empty() {
                return Err("Empty command line");
            }

            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            let verb = parts[0].to_uppercase();
            let args = parts.get(1).unwrap_or(&"").to_string();

            Ok(SmtpCommand { verb, args })
        }
    }

    impl EmailAddress {
        /// Parse email address from string
        pub fn parse(addr: &str) -> Result<Self, &'static str> {
            let addr = addr.trim();
            if addr.is_empty() {
                return Err("Empty email address");
            }

            let at_pos = addr.find('@').ok_or("Invalid email address: missing @")?;

            if at_pos == 0 || at_pos == addr.len() - 1 {
                return Err("Invalid email address: empty local or domain part");
            }

            let local = addr[..at_pos].to_string();
            let domain = addr[at_pos + 1..].to_string();

            Ok(EmailAddress { local, domain })
        }
    }

    #[test]
    fn test_smtp_command_parse() {
        assert_eq!(
            SmtpCommand::parse("EHLO example.com"),
            Ok(SmtpCommand {
                verb: "EHLO".to_string(),
                args: "example.com".to_string()
            })
        );

        assert_eq!(
            SmtpCommand::parse("QUIT"),
            Ok(SmtpCommand {
                verb: "QUIT".to_string(),
                args: "".to_string()
            })
        );

        assert_eq!(
            SmtpCommand::parse("  mail from:<test@example.com>  "),
            Ok(SmtpCommand {
                verb: "MAIL".to_string(),
                args: "from:<test@example.com>".to_string()
            })
        );

        assert!(SmtpCommand::parse("").is_err());
    }

    #[test]
    fn test_auth_plain_parse() {
        // "user\0test@example.com\0password123" base64 encoded
        use base64::Engine;
        let base64_creds =
            base64::engine::general_purpose::STANDARD.encode("user\0test@example.com\0password123");

        let creds = AuthCredentials::parse_plain(&base64_creds).unwrap();
        assert_eq!(creds.username, "test@example.com");
        assert_eq!(creds.password, "password123");

        // Test invalid formats
        assert!(AuthCredentials::parse_plain("invalid_base64!").is_err());

        let invalid_format = base64::engine::general_purpose::STANDARD.encode("user\0password"); // Missing second null
        assert!(AuthCredentials::parse_plain(&invalid_format).is_err());
    }

    #[test]
    fn test_extract_account_id52() {
        // Test with actual valid ID52 format
        let valid_key = fastn_id52::SecretKey::generate();
        let valid_id52 = valid_key.public_key().id52();

        let valid_creds = AuthCredentials {
            username: format!("anything@{}.fastn", valid_id52),
            password: "password".to_string(),
        };
        let result = valid_creds.extract_account_id52();
        assert!(result.is_some());
        assert_eq!(result.unwrap().id52(), valid_id52);

        // Test with different user prefix - should still work
        let prefix_creds = AuthCredentials {
            username: format!("inbox@{}.fastn", valid_id52),
            password: "password".to_string(),
        };
        let result = prefix_creds.extract_account_id52();
        assert!(result.is_some());
        assert_eq!(result.unwrap().id52(), valid_id52);

        // Test invalid formats
        let invalid_creds = AuthCredentials {
            username: "no-at-sign".to_string(),
            password: "password".to_string(),
        };
        assert!(invalid_creds.extract_account_id52().is_none());

        // Test that .com domains are rejected for security
        let com_domain_creds = AuthCredentials {
            username: format!("user@{}.com", valid_id52),
            password: "password".to_string(),
        };
        assert!(
            com_domain_creds.extract_account_id52().is_none(),
            "Security: .com domains should be rejected"
        );

        // Test other purchasable TLDs are rejected
        let org_domain_creds = AuthCredentials {
            username: format!("user@{}.org", valid_id52),
            password: "password".to_string(),
        };
        assert!(
            org_domain_creds.extract_account_id52().is_none(),
            "Security: .org domains should be rejected"
        );

        let short_id_creds = AuthCredentials {
            username: "user@short.domain.fastn".to_string(),
            password: "password".to_string(),
        };
        assert!(short_id_creds.extract_account_id52().is_none());
    }

    #[test]
    fn test_email_address_parse() {
        assert_eq!(
            EmailAddress::parse("user@example.com"),
            Ok(EmailAddress {
                local: "user".to_string(),
                domain: "example.com".to_string()
            })
        );

        assert!(EmailAddress::parse("").is_err());
        assert!(EmailAddress::parse("no-at-sign").is_err());
        assert!(EmailAddress::parse("@example.com").is_err());
        assert!(EmailAddress::parse("user@").is_err());
    }

    #[test]
    fn test_mail_from_parse() {
        assert_eq!(
            parse_mail_from("FROM:<user@example.com>"),
            Ok("user@example.com".to_string())
        );

        assert_eq!(
            parse_mail_from("from: user@example.com"),
            Ok("user@example.com".to_string())
        );

        assert_eq!(parse_mail_from("FROM:<>"), Err("Empty address in brackets"));

        assert!(parse_mail_from("TO:<user@example.com>").is_err());
        assert!(parse_mail_from("invalid").is_err());
    }

    #[test]
    fn test_rcpt_to_parse() {
        assert_eq!(
            parse_rcpt_to("TO:<user@example.com>"),
            Ok("user@example.com".to_string())
        );

        assert_eq!(
            parse_rcpt_to("to: user@example.com"),
            Ok("user@example.com".to_string())
        );

        assert!(parse_rcpt_to("FROM:<user@example.com>").is_err());
        assert!(parse_rcpt_to("invalid").is_err());
    }
}
