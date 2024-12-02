use ftd::interpreter::things::record::RecordExt;
use ftd::interpreter::FieldExt;

pub trait OrTypeExt {
    fn scan_ast(
        ast: ftd_ast::Ast,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()>;
    fn from_ast(
        ast: ftd_ast::Ast,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_resolved::OrType>>;
}

impl OrTypeExt for fastn_resolved::OrType {
    fn scan_ast(
        ast: ftd_ast::Ast,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()> {
        let or_type = ast.get_or_type(doc.name)?;
        for mut variant in or_type.variants {
            variant.set_name(format!("{}.{}", or_type.name, variant.name()).as_str());
            fastn_resolved::OrTypeVariant::scan_ast(variant, doc)?;
        }
        Ok(())
    }

    fn from_ast(
        ast: ftd_ast::Ast,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_resolved::OrType>> {
        let or_type = ast.get_or_type(doc.name)?;
        let name = doc.resolve_name(or_type.name.as_str());
        let line_number = or_type.line_number();
        let mut variants = vec![];
        for mut variant in or_type.variants {
            variant.set_name(format!("{}.{}", or_type.name, variant.name()).as_str());
            variants.push(try_ok_state!(fastn_resolved::OrTypeVariant::from_ast(
                variant, doc
            )?))
        }
        Ok(ftd::interpreter::StateWithThing::new_thing(
            fastn_resolved::OrType::new(name.as_str(), variants, line_number),
        ))
    }
}

pub trait OrTypeVariantExt {
    fn ok_constant(&self, doc_id: &str) -> ftd::interpreter::Result<&fastn_resolved::Field>;
    fn scan_ast(
        ast_variant: ftd_ast::OrTypeVariant,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()>;
    fn from_ast(
        ast_variant: ftd_ast::OrTypeVariant,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_resolved::OrTypeVariant>>;
    fn to_thing(
        &self,
        doc_name: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::Thing>;
}

impl OrTypeVariantExt for fastn_resolved::OrTypeVariant {
    fn ok_constant(&self, doc_id: &str) -> ftd::interpreter::Result<&fastn_resolved::Field> {
        match self {
            fastn_resolved::OrTypeVariant::Constant(c) => Ok(c),
            t => ftd::interpreter::utils::e2(
                format!("Expected constant, found: {:?}", t),
                doc_id,
                t.line_number(),
            ),
        }
    }

    fn scan_ast(
        ast_variant: ftd_ast::OrTypeVariant,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()> {
        match ast_variant {
            ftd_ast::OrTypeVariant::AnonymousRecord(record) => {
                fastn_resolved::Record::scan_record(record, doc)
            }
            ftd_ast::OrTypeVariant::Regular(variant) => {
                fastn_resolved::Field::scan_ast_field(variant, doc, &Default::default())
            }
            ftd_ast::OrTypeVariant::Constant(variant) => {
                fastn_resolved::Field::scan_ast_field(variant, doc, &Default::default())
            }
        }
    }

    fn from_ast(
        ast_variant: ftd_ast::OrTypeVariant,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_resolved::OrTypeVariant>>
    {
        match ast_variant {
            ftd_ast::OrTypeVariant::AnonymousRecord(record) => {
                Ok(ftd::interpreter::StateWithThing::new_thing(
                    fastn_resolved::OrTypeVariant::new_record(try_ok_state!(
                        fastn_resolved::Record::from_record(record, doc)?
                    )),
                ))
            }
            ftd_ast::OrTypeVariant::Regular(variant) => {
                Ok(ftd::interpreter::StateWithThing::new_thing(
                    fastn_resolved::OrTypeVariant::new_regular(try_ok_state!(
                        fastn_resolved::Field::from_ast_field(variant, doc, &Default::default())?
                    )),
                ))
            }
            ftd_ast::OrTypeVariant::Constant(variant) => {
                let variant = try_ok_state!(fastn_resolved::Field::from_ast_field(
                    variant,
                    doc,
                    &Default::default()
                )?);
                validate_constant_variant(&variant, doc)?;
                Ok(ftd::interpreter::StateWithThing::new_thing(
                    fastn_resolved::OrTypeVariant::new_constant(variant),
                ))
            }
        }
    }

    fn to_thing(
        &self,
        doc_name: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::Thing> {
        match self {
            fastn_resolved::OrTypeVariant::AnonymousRecord(r) => {
                Ok(ftd::interpreter::Thing::Record(r.clone()))
            }
            fastn_resolved::OrTypeVariant::Constant(_)
            | fastn_resolved::OrTypeVariant::Regular(_) => {
                Err(ftd::interpreter::Error::ParseError {
                    message: format!("Can't convert the or-type-variant to thing `{self:?}`"),
                    doc_id: doc_name.to_string(),
                    line_number,
                })
            }
        }
    }
}

fn validate_constant_variant(
    variant: &fastn_resolved::Field,
    doc: &ftd::interpreter::TDoc,
) -> ftd::interpreter::Result<()> {
    if variant.default.is_none()
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
