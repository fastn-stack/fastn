impl fastn_account::Alias {
    /// Get the ID52 string for this alias
    pub fn id52(&self) -> String {
        self.public_key.to_string()
    }

    /// Get the public key
    pub fn public_key(&self) -> &fastn_id52::PublicKey {
        &self.public_key
    }

    /// Get the secret key (use with caution)
    pub fn secret_key(&self) -> &fastn_id52::SecretKey {
        &self.secret_key
    }

    /// Get the public name visible to others
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the private reason/note for this alias
    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// Check if this is the primary alias
    pub fn is_primary(&self) -> bool {
        self.is_primary
    }

    /// Sign a message with this alias's private key
    pub fn sign(&self, message: &[u8]) -> fastn_id52::Signature {
        self.secret_key.sign(message)
    }
}
