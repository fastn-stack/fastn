#[derive(Debug, PartialEq)]
pub struct TDoc<'a> {
    pub name: &'a str,
    pub aliases: &'a ftd::Map<String>,
    pub bag: &'a ftd::Map<ftd::interpreter::Thing>,
}

impl<'a> TDoc<'a> {
    pub fn resolve_name(&self, name: &str) -> String {
        ftd::interpreter::utils::resolve_name(name, self.name, self.aliases)
    }
}
