pub struct SiteToken(pub String);

impl SiteToken {
    pub fn from_env() -> Result<Self, std::env::VarError> {
        Ok(Self(std::env::var("FIFTHTRY_SITE_WRITE_TOKEN")?))
    }
}
