#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct OrType {
    pub name: String,
    pub variants: Vec<OrTypeVariant>,
    pub line_number: usize,
}

impl OrType {
    fn new(
        name: &str,
        variants: Vec<ftd::interpreter::OrTypeVariant>,
        line_number: usize,
    ) -> OrType {
        OrType {
            name: name.to_string(),
            variants,
            line_number,
        }
    }

    pub fn or_type_name(name: &str) -> String {
        if name.starts_with("ftd") {
            return name.to_string();
        }
        if let Some((_, last)) = name.rsplit_once("#") {
            return last.to_string();
        }
        name.to_string()
    }

    pub(crate) fn scan_ast(
        ast: ftd::ast::AST,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()> {
        let or_type = ast.get_or_type(doc.name)?;
        for mut variant in or_type.variants {
            variant.set_name(format!("{}.{}", or_type.name, variant.name()).as_str());
            ftd::interpreter::OrTypeVariant::scan_ast(variant, doc)?;
        }
        Ok(())
    }

    pub(crate) fn from_ast(
        ast: ftd::ast::AST,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<OrType>> {
        let or_type = ast.get_or_type(doc.name)?;
        let name = doc.resolve_name(or_type.name.as_str());
        let line_number = or_type.line_number();
        let mut variants = vec![];
        for mut variant in or_type.variants {
            variant.set_name(format!("{}.{}", or_type.name, variant.name()).as_str());
            variants.push(try_ok_state!(ftd::interpreter::OrTypeVariant::from_ast(
                variant, doc
            )?))
        }
        Ok(ftd::interpreter::StateWithThing::new_thing(OrType::new(
            name.as_str(),
            variants,
            line_number,
        )))
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum OrTypeVariant {
    AnonymousRecord(ftd::interpreter::Record),
    Regular(ftd::interpreter::Field),
    Constant(ftd::interpreter::Field),
}

impl OrTypeVariant {
    pub fn new_record(record: ftd::interpreter::Record) -> OrTypeVariant {
        OrTypeVariant::AnonymousRecord(record)
    }

    pub fn new_constant(variant: ftd::interpreter::Field) -> OrTypeVariant {
        OrTypeVariant::Constant(variant)
    }

    pub fn new_regular(variant: ftd::interpreter::Field) -> OrTypeVariant {
        OrTypeVariant::Regular(variant)
    }

    pub fn is_constant(&self) -> bool {
        matches!(self, ftd::interpreter::OrTypeVariant::Constant(_))
    }

    pub fn name(&self) -> String {
        match self {
            OrTypeVariant::AnonymousRecord(ar) => ar.name.to_string(),
            OrTypeVariant::Regular(r) => r.name.to_string(),
            OrTypeVariant::Constant(c) => c.name.to_string(),
        }
    }

    pub fn ok_constant(&self, doc_id: &str) -> ftd::interpreter::Result<&ftd::interpreter::Field> {
        match self {
            ftd::interpreter::OrTypeVariant::Constant(c) => Ok(c),
            t => ftd::interpreter::utils::e2(
                format!("Expected constant, found: {:?}", t),
                doc_id,
                t.line_number(),
            ),
        }
    }

    pub fn line_number(&self) -> usize {
        match self {
            OrTypeVariant::AnonymousRecord(ar) => ar.line_number,
            OrTypeVariant::Regular(r) => r.line_number,
            OrTypeVariant::Constant(c) => c.line_number,
        }
    }

    pub fn scan_ast(
        ast_variant: ftd::ast::OrTypeVariant,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()> {
        match ast_variant {
            ftd::ast::OrTypeVariant::AnonymousRecord(record) => {
                ftd::interpreter::Record::scan_record(record, doc)
            }
            ftd::ast::OrTypeVariant::Regular(variant) => {
                ftd::interpreter::Field::scan_ast_field(variant, doc, &Default::default())
            }
            ftd::ast::OrTypeVariant::Constant(variant) => {
                ftd::interpreter::Field::scan_ast_field(variant, doc, &Default::default())
            }
        }
    }

    pub fn from_ast(
        ast_variant: ftd::ast::OrTypeVariant,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<OrTypeVariant>> {
        match ast_variant {
            ftd::ast::OrTypeVariant::AnonymousRecord(record) => {
                Ok(ftd::interpreter::StateWithThing::new_thing(
                    ftd::interpreter::OrTypeVariant::new_record(try_ok_state!(
                        ftd::interpreter::Record::from_record(record, doc)?
                    )),
                ))
            }
            ftd::ast::OrTypeVariant::Regular(variant) => {
                Ok(ftd::interpreter::StateWithThing::new_thing(
                    ftd::interpreter::OrTypeVariant::new_regular(try_ok_state!(
                        ftd::interpreter::Field::from_ast_field(variant, doc, &Default::default())?
                    )),
                ))
            }
            ftd::ast::OrTypeVariant::Constant(variant) => {
                let variant = try_ok_state!(ftd::interpreter::Field::from_ast_field(
                    variant,
                    doc,
                    &Default::default()
                )?);
                validate_constant_variant(&variant, doc)?;
                Ok(ftd::interpreter::StateWithThing::new_thing(
                    ftd::interpreter::OrTypeVariant::new_constant(variant),
                ))
            }
        }
    }

    pub fn fields(&self) -> Vec<&ftd::interpreter::Field> {
        match self {
            OrTypeVariant::AnonymousRecord(r) => r.fields.iter().collect(),
            OrTypeVariant::Regular(r) => vec![r],
            OrTypeVariant::Constant(c) => vec![c],
        }
    }
}

fn validate_constant_variant(
    variant: &ftd::interpreter::Field,
    doc: &ftd::interpreter::TDoc,
) -> ftd::interpreter::Result<()> {
    if variant.value.is_none()
        && !(variant.kind.is_void() || variant.kind.is_optional() || variant.kind.is_list())
    {
        return ftd::interpreter::utils::e2(
            format!("The constant variant `{}` can't be empty", variant.name),
            doc.name,
            variant.line_number,
        );
    }
    Ok(())
}
