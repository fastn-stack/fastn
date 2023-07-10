#[derive(Debug, Clone, PartialEq)]
pub enum DeviceType {
    Desktop,
    Mobile,
}

impl From<&str> for DeviceType {
    fn from(s: &str) -> Self {
        match s {
            "ftd#desktop" => DeviceType::Desktop,
            "ftd#mobile" => DeviceType::Mobile,
            t => unreachable!("Unknown device {}", t),
        }
    }
}

#[derive(Debug)]
pub struct DeviceBlock {
    pub device: fastn_js::DeviceType,
    pub statements: Vec<fastn_js::ComponentStatement>,
    pub parent: String,
    pub should_return: bool,
}
