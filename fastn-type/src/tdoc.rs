#[derive(Debug, PartialEq)]
pub struct TDoc<'a> {
    pub name: &'a str,
    pub aliases: &'a fastn_type::Map<String>,
    pub bag: &'a indexmap::IndexMap<String, crate::thing::Thing>,
}
