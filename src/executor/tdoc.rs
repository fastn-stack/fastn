#[derive(Debug, PartialEq)]
pub struct TDoc<'a> {
    pub name: &'a str,
    pub aliases: &'a ftd::Map<String>,
    pub bag: &'a mut ftd::Map<ftd::interpreter2::Thing>,
}

impl<'a> TDoc<'a> {
    pub(crate) fn itdoc(&self) -> ftd::interpreter2::TDoc {
        ftd::interpreter2::TDoc {
            name: self.name,
            aliases: self.aliases,
            bag: self.bag,
        }
    }
}
