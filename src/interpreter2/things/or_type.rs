#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct OrType {
    pub name: String,
    pub variants: Vec<ftd::interpreter2::Record>,
    pub line_number: usize,
}

impl OrType {
    fn new(name: &str, variants: Vec<ftd::interpreter2::Record>, line_number: usize) -> OrType {
        OrType {
            name: name.to_string(),
            variants,
            line_number,
        }
    }

    pub(crate) fn from_ast(
        ast: ftd::ast::AST,
        doc: &ftd::interpreter2::TDoc,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::StateWithThing<OrType>> {
        let or_type = ast.get_or_type(doc.name)?;
        let name = doc.resolve_name(or_type.name.as_str());
        let line_number = or_type.line_number();
        let mut variants = vec![];
        for variant in or_type.variants {
            variants.push(try_ok_state!(ftd::interpreter2::Record::from_record(
                variant, doc
            )?));
        }
        Ok(ftd::interpreter2::StateWithThing::new_thing(OrType::new(
            name.as_str(),
            variants,
            line_number,
        )))
    }
}
