/// Authentication and password management for FASTN accounts
///
/// Generate a secure random password
pub fn generate_password() -> String {
    use rand::Rng;

    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                             abcdefghijklmnopqrstuvwxyz\
                             0123456789!@#$%^&*";
    const PASSWORD_LEN: usize = 16;

    let mut rng = rand::thread_rng();

    (0..PASSWORD_LEN)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Hash a password using Argon2
pub fn hash_password(password: &str) -> Result<String, crate::HashPasswordError> {
    use argon2::{
        Argon2,
        password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
    };

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| crate::HashPasswordError::HashingFailed { 
            message: e.to_string() 
        })?
        .to_string();

    Ok(password_hash)
}

/// Verify a password against a hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, crate::VerifyPasswordError> {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};

    let parsed_hash = PasswordHash::new(hash).map_err(|e| {
        crate::VerifyPasswordError::HashParsingFailed { 
            message: e.to_string() 
        }
    })?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_generation() {
        let password1 = generate_password();
        let password2 = generate_password();

        assert_eq!(password1.len(), 16);
        assert_eq!(password2.len(), 16);
        assert_ne!(password1, password2); // Should be different
    }

    #[test]
    fn test_password_hashing() {
        let password = "test_password_123";
        let hash = hash_password(password).unwrap();

        // Hash should be a valid Argon2 hash
        assert!(hash.starts_with("$argon2"));

        // Same password should verify
        assert!(verify_password(password, &hash).unwrap());

        // Different password should not verify
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }
}
