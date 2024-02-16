#[derive(serde::Deserialize)]
pub struct UserData {
    pub id: i32,
    pub username: String,
    pub name: String,
    pub email: String,
    pub verified_email: bool,
}
