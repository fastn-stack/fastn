#[derive(serde::Deserialize, Clone, Debug, PartialEq, serde::Serialize)]
pub struct RiveData {
    pub id: String,
    pub src: String,
    pub state_machine: Vec<String>,
    pub artboard: Option<String>,
    pub autoplay: bool,
    pub events: Vec<fastn_type::Event>,
}
