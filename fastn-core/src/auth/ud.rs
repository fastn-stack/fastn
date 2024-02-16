#[derive(serde::Deserialize, Clone)]
pub struct UserData {
    pub username: String,
    pub name: String,
    pub email: String,
    pub verified_email: bool,
}
