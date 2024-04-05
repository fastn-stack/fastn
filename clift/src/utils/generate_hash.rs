// Warning: this function is used in `ft` too which checks the changes in the
// file content and hence ensures that only diff file is uploaded.
// If this function ever needed to be changed then make sure to change the
// corresponding function in `ft` too.
// https://github.com/FifthTry/ft/blob/main/ft-db/src/utils.rs
// This hash can be created using cli command:
// `shasum -a 256 <file-path>` or `echo -n "<string>" | shasum -a 256`
pub fn generate_hash(content: impl AsRef<[u8]>) -> String {
    use sha2::digest::FixedOutput;
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(content);
    format!("{:X}", hasher.finalize_fixed())
}
