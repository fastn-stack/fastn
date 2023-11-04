#[derive(Debug, PartialEq)]
pub struct TDoc<'a> {
    pub name: &'a str,
    pub aliases: &'a ftd::Map<String>,
    pub bag: &'a ftd::Map<ftd::ftd2021::interpreter::Thing>,
}

impl TDoc<'_> {
    pub fn resolve_name(&self, name: &str) -> String {
        ftd::ftd2021::interpreter::utils::resolve_name(name, self.name, self.aliases)
    }
}
