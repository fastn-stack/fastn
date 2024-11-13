pub(crate) trait RecordExt {
    fn scan_ast(
        ast: ftd_ast::Ast,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()>;
    fn scan_record(
        record: ftd_ast::Record,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()>;
    fn from_ast(
        ast: ftd_ast::Ast,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_type::Record>>;
    fn from_record(
        record: ftd_ast::Record,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_type::Record>>;
    fn get_field(
        &self,
        name: &str,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<&fastn_type::Field>;
}

impl RecordExt for fastn_type::Record {
    fn scan_ast(
        ast: ftd_ast::Ast,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()> {
        let record = ast.get_record(doc.name)?;
        fastn_type::Record::scan_record(record, doc)
    }

    fn scan_record(
        record: ftd_ast::Record,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()> {
        let name = doc.resolve_name(record.name.as_str());
        let known_kinds = std::iter::IntoIterator::into_iter([(
            record.name.to_string(),
            fastn_type::Kind::record(name.as_str()),
        )])
        .collect::<ftd::Map<fastn_type::Kind>>();
        fastn_type::Field::scan_ast_fields(record.fields, doc, &known_kinds)
    }

    fn from_ast(
        ast: ftd_ast::Ast,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_type::Record>> {
        let record = ast.get_record(doc.name)?;
        fastn_type::Record::from_record(record, doc)
    }

    fn from_record(
        record: ftd_ast::Record,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_type::Record>> {
        let name = doc.resolve_name(record.name.as_str());
        let known_kinds = std::iter::IntoIterator::into_iter([(
            record.name.to_string(),
            fastn_type::Kind::Record {
                name: name.to_string(),
            },
        )])
        .collect::<ftd::Map<fastn_type::Kind>>();
        let fields = try_ok_state!(fastn_type::Field::from_ast_fields(
            record.name.as_str(),
            record.fields,
            doc,
            &known_kinds
        )?);
        validate_record_fields(name.as_str(), &fields, doc.name)?;
        Ok(ftd::interpreter::StateWithThing::new_thing(
            fastn_type::Record::new(name.as_str(), fields, record.line_number),
        ))
    }

    fn get_field(
        &self,
        name: &str,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<&fastn_type::Field> {
        use itertools::Itertools;

        let field = self.fields.iter().filter(|v| v.name.eq(name)).collect_vec();
        if field.is_empty() {
            return ftd::interpreter::utils::e2(
                format!(
                    "Cannot find the field `{}` for record `{}`",
                    name, self.name
                )
                .as_str(),
                doc_id,
                line_number,
            );
        }

        if field.len() > 1 {
            return ftd::interpreter::utils::e2(
                format!(
                    "Multiple fields `{}` for record `{}` found",
                    name, self.name
                )
                .as_str(),
                doc_id,
                line_number,
            );
        }

        Ok(field.first().unwrap())
    }
}

pub(crate) trait FieldExt {
    fn get_default_interpreter_property_value(
        &self,
        properties: &[fastn_type::Property],
    ) -> ftd::interpreter::Result<Option<fastn_type::PropertyValue>>;
    fn get_default_interpreter_value(
        &self,
        doc: &ftd::interpreter::TDoc,
        properties: &[fastn_type::Property],
    ) -> ftd::interpreter::Result<Option<fastn_type::Value>>;
    fn scan_ast_fields(
        fields: Vec<ftd_ast::Field>,
        doc: &mut ftd::interpreter::TDoc,
        known_kinds: &ftd::Map<fastn_type::Kind>,
    ) -> ftd::interpreter::Result<()>;
    fn resolve_kinds_from_ast_fields(
        ast_fields: Vec<ftd_ast::Field>,
        doc: &mut ftd::interpreter::TDoc,
        known_kinds: &ftd::Map<fastn_type::Kind>,
    ) -> ftd::interpreter::Result<
        ftd::interpreter::StateWithThing<Vec<ftd::executor::FieldWithValue>>,
    >;
    fn resolve_values_from_ast_fields(
        definition_name: &str,
        fields_with_resolved_kinds: Vec<(fastn_type::Field, Option<ftd_ast::VariableValue>)>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<Vec<fastn_type::Field>>>;
    fn from_ast_fields(
        name: &str,
        fields: Vec<ftd_ast::Field>,
        doc: &mut ftd::interpreter::TDoc,
        known_kinds: &ftd::Map<fastn_type::Kind>,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<Vec<fastn_type::Field>>>;
    fn scan_ast_field(
        field: ftd_ast::Field,
        doc: &mut ftd::interpreter::TDoc,
        known_kinds: &ftd::Map<fastn_type::Kind>,
    ) -> ftd::interpreter::Result<()>;
    fn from_ast_field(
        field: ftd_ast::Field,
        doc: &mut ftd::interpreter::TDoc,
        known_kinds: &ftd::Map<fastn_type::Kind>,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_type::Field>>;
    fn from_ast_field_kind(
        field: ftd_ast::Field,
        doc: &mut ftd::interpreter::TDoc,
        known_kinds: &ftd::Map<fastn_type::Kind>,
    ) -> ftd::interpreter::Result<
        ftd::interpreter::StateWithThing<(fastn_type::Field, Option<ftd_ast::VariableValue>)>,
    >;
    fn for_component_or_web_component(
        component_name: &str,
        definition_name_with_arguments: &Option<(&str, &mut [fastn_type::Field])>,
        doc: &mut ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<Vec<fastn_type::Field>>>;
    fn for_component(
        component_name: &str,
        definition_name_with_arguments: &Option<(&str, &mut [fastn_type::Field])>,
        doc: &mut ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<Vec<fastn_type::Field>>>;
    fn for_web_component(
        component_name: &str,
        definition_name_with_arguments: &Option<(&str, &mut [fastn_type::Field])>,
        doc: &mut ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<Vec<fastn_type::Field>>>;
    fn update_with_or_type_variant(
        &mut self,
        doc: &mut ftd::interpreter::TDoc,
        variant: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<()>>;
}

impl FieldExt for fastn_type::Field {
    fn get_default_interpreter_property_value(
        &self,
        properties: &[fastn_type::Property],
    ) -> ftd::interpreter::Result<Option<fastn_type::PropertyValue>> {
        let sources = self.to_sources();
        let properties = ftd::interpreter::utils::find_properties_by_source(
            sources.as_slice(),
            properties,
            "", // doc_name
            self,
            0, // line_number
        )?;

        for property in properties {
            if property.condition.is_none() {
                return Ok(Some(property.value));
            }
        }

        Ok(None)
    }

    fn get_default_interpreter_value(
        &self,
        doc: &ftd::interpreter::TDoc,
        properties: &[fastn_type::Property],
    ) -> ftd::interpreter::Result<Option<fastn_type::Value>> {
        use ftd::interpreter::PropertyValueExt;

        let property_value = self.get_default_interpreter_property_value(properties)?;
        if let Some(property_value) = property_value {
            return Ok(property_value.resolve(doc, 0).ok());
        }
        Ok(None)
    }

    fn scan_ast_fields(
        fields: Vec<ftd_ast::Field>,
        doc: &mut ftd::interpreter::TDoc,
        known_kinds: &ftd::Map<fastn_type::Kind>,
    ) -> ftd::interpreter::Result<()> {
        for field in fields {
            fastn_type::Field::scan_ast_field(field, doc, known_kinds)?;
        }
        Ok(())
    }

    fn resolve_kinds_from_ast_fields(
        ast_fields: Vec<ftd_ast::Field>,
        doc: &mut ftd::interpreter::TDoc,
        known_kinds: &ftd::Map<fastn_type::Kind>,
    ) -> ftd::interpreter::Result<
        ftd::interpreter::StateWithThing<Vec<ftd::executor::FieldWithValue>>,
    > {
        let mut fields_with_resolved_kinds = vec![];
        for field in ast_fields {
            fields_with_resolved_kinds.push(try_ok_state!(fastn_type::Field::from_ast_field_kind(
                field,
                doc,
                known_kinds
            )?));
        }
        Ok(ftd::interpreter::StateWithThing::new_thing(
            fields_with_resolved_kinds,
        ))
    }

    fn resolve_values_from_ast_fields(
        definition_name: &str,
        mut fields_with_resolved_kinds: Vec<(fastn_type::Field, Option<ftd_ast::VariableValue>)>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<Vec<fastn_type::Field>>> {
        use ftd::interpreter::PropertyValueExt;
        use itertools::Itertools;

        let mut fields = fields_with_resolved_kinds
            .iter()
            .map(|v| v.0.clone())
            .collect_vec();
        for (field, value) in fields_with_resolved_kinds.iter_mut() {
            let value = if let Some(value) = value {
                Some(try_ok_state!(
                    fastn_type::PropertyValue::from_ast_value_with_argument(
                        value.to_owned(),
                        doc,
                        field.mutable,
                        Some(&field.kind),
                        &mut Some((definition_name, fields.as_mut_slice())),
                        &None
                    )?
                ))
            } else {
                None
            };

            field.value = value;
        }
        let resolved_fields = fields_with_resolved_kinds
            .into_iter()
            .map(|v| v.0)
            .collect_vec();

        Ok(ftd::interpreter::StateWithThing::new_thing(resolved_fields))
    }

    fn from_ast_fields(
        name: &str,
        fields: Vec<ftd_ast::Field>,
        doc: &mut ftd::interpreter::TDoc,
        known_kinds: &ftd::Map<fastn_type::Kind>,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<Vec<fastn_type::Field>>> {
        // First resolve all kinds from ast fields
        let partial_resolved_fields = try_ok_state!(
            fastn_type::Field::resolve_kinds_from_ast_fields(fields, doc, known_kinds)?
        );

        // Once ast kinds are resolved, then try resolving ast values
        let resolved_fields =
            fastn_type::Field::resolve_values_from_ast_fields(name, partial_resolved_fields, doc)?;

        Ok(resolved_fields)
    }

    fn scan_ast_field(
        field: ftd_ast::Field,
        doc: &mut ftd::interpreter::TDoc,
        known_kinds: &ftd::Map<fastn_type::Kind>,
    ) -> ftd::interpreter::Result<()> {
        use ftd::interpreter::{KindDataExt, PropertyValueExt};

        fastn_type::KindData::scan_ast_kind(field.kind, known_kinds, doc, field.line_number)?;

        if let Some(value) = field.value {
            fastn_type::PropertyValue::scan_ast_value(value, doc)?;
        }

        Ok(())
    }

    fn from_ast_field(
        field: ftd_ast::Field,
        doc: &mut ftd::interpreter::TDoc,
        known_kinds: &ftd::Map<fastn_type::Kind>,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_type::Field>> {
        use ftd::interpreter::{KindDataExt, PropertyValueExt};

        let kind = try_ok_state!(fastn_type::KindData::from_ast_kind(
            field.kind,
            known_kinds,
            doc,
            field.line_number,
        )?);

        let value = if let Some(value) = field.value {
            Some(try_ok_state!(fastn_type::PropertyValue::from_ast_value(
                value,
                doc,
                field.mutable,
                Some(&kind),
            )?))
        } else {
            None
        };

        Ok(ftd::interpreter::StateWithThing::new_thing(
            fastn_type::Field {
                name: field.name.to_string(),
                kind,
                mutable: field.mutable,
                value,
                line_number: field.line_number,
                access_modifier: access_modifier(field.access_modifier),
            },
        ))
    }

    fn from_ast_field_kind(
        field: ftd_ast::Field,
        doc: &mut ftd::interpreter::TDoc,
        known_kinds: &ftd::Map<fastn_type::Kind>,
    ) -> ftd::interpreter::Result<
        ftd::interpreter::StateWithThing<(fastn_type::Field, Option<ftd_ast::VariableValue>)>,
    > {
        use ftd::interpreter::KindDataExt;

        let kind = try_ok_state!(fastn_type::KindData::from_ast_kind(
            field.kind,
            known_kinds,
            doc,
            field.line_number,
        )?);

        Ok(ftd::interpreter::StateWithThing::new_thing((
            fastn_type::Field {
                name: field.name.to_string(),
                kind,
                mutable: field.mutable,
                value: None,
                line_number: field.line_number,
                access_modifier: access_modifier(field.access_modifier),
            },
            field.value,
        )))
    }

    fn for_component_or_web_component(
        component_name: &str,
        definition_name_with_arguments: &Option<(&str, &mut [fastn_type::Field])>,
        doc: &mut ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<Vec<fastn_type::Field>>> {
        match Self::for_component(
            component_name,
            definition_name_with_arguments,
            doc,
            line_number,
        ) {
            Ok(swt) => Ok(swt),
            Err(e1) => match Self::for_web_component(
                component_name,
                definition_name_with_arguments,
                doc,
                line_number,
            ) {
                Ok(swt) => Ok(swt),
                Err(e2) => {
                    ftd::interpreter::utils::e2(format!("{:?} {:?}", e1, e2), doc.name, line_number)
                }
            },
        }
    }
    fn for_component(
        component_name: &str,
        definition_name_with_arguments: &Option<(&str, &mut [fastn_type::Field])>,
        doc: &mut ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<Vec<fastn_type::Field>>> {
        Ok(ftd::interpreter::StateWithThing::new_thing(
            match definition_name_with_arguments {
                Some((name, arg)) if name.eq(&component_name) => arg.to_vec(),
                _ => try_ok_state!(doc.search_component(component_name, line_number)?).arguments,
            },
        ))
    }

    fn for_web_component(
        component_name: &str,
        definition_name_with_arguments: &Option<(&str, &mut [fastn_type::Field])>,
        doc: &mut ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<Vec<fastn_type::Field>>> {
        Ok(ftd::interpreter::StateWithThing::new_thing(
            match definition_name_with_arguments {
                Some((name, arg)) if name.eq(&component_name) => arg.to_vec(),
                _ => {
                    try_ok_state!(doc.search_web_component(component_name, line_number)?).arguments
                }
            },
        ))
    }

    fn update_with_or_type_variant(
        &mut self,
        doc: &mut ftd::interpreter::TDoc,
        variant: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<()>> {
        match self.kind.kind.mut_inner() {
            fastn_type::Kind::OrType {
                name,
                variant: v,
                full_variant,
            } => {
                let or_type = try_ok_state!(doc.search_or_type(name, self.line_number)?);
                let (variant_name, remaining) =
                    ftd::ftd2021::p2::utils::get_doc_name_and_remaining(variant)?;
                let or_variant = or_type
                    .variants
                    .iter()
                    .find(|v| {
                        v.name()
                            .trim_start_matches(format!("{}.", name).as_str())
                            .eq(variant_name.as_str())
                    })
                    .ok_or(ftd::interpreter::Error::ParseError {
                        message: format!(
                            "Cannot find variant `{}` for or-type `{}`",
                            variant, name
                        ),
                        doc_id: doc.name.to_string(),
                        line_number,
                    })?;

                check_variant_if_constant(or_variant, remaining, doc)?;
                let variant = Some(format!("{}.{}", name, variant));

                v.clone_from(&variant);
                *full_variant = variant;
                Ok(ftd::interpreter::StateWithThing::new_thing(()))
            }
            t => ftd::interpreter::utils::e2(
                format!(
                    "Expected or-type for variant `{}`, found: `{:?}`",
                    variant, t
                ),
                doc.name,
                line_number,
            ),
        }
    }
}

fn access_modifier(am: ftd_p1::AccessModifier) -> fastn_type::AccessModifier {
    match am {
        ftd_p1::AccessModifier::Private => fastn_type::AccessModifier::Private,
        ftd_p1::AccessModifier::Public => fastn_type::AccessModifier::Public,
    }
}

fn validate_record_fields(
    rec_name: &str,
    fields: &[fastn_type::Field],
    doc_id: &str,
) -> ftd::interpreter::Result<()> {
    if let Some(field) = fields.iter().find(|v| v.mutable) {
        return ftd::interpreter::utils::e2(
            format!(
                "Currently, mutable field `{}` in record `{}` is not supported.",
                field.name, rec_name
            )
            .as_str(),
            doc_id,
            field.line_number,
        );
    }
    Ok(())
}

fn check_variant_if_constant(
    or_variant: &ftd::interpreter::OrTypeVariant,
    _remaining: Option<String>,
    doc: &ftd::interpreter::TDoc,
) -> ftd::interpreter::Result<()> {
    match or_variant {
        ftd::interpreter::OrTypeVariant::AnonymousRecord(_r) => {} // Todo: check on remaining for constant and throw error if found
        ftd::interpreter::OrTypeVariant::Regular(_r) => {} // Todo: check on remaining for constant and throw error if found
        ftd::interpreter::OrTypeVariant::Constant(c) => {
            return ftd::interpreter::utils::e2(
                format!("Cannot pass deconstructed constant variant `{}`", c.name),
                doc.name,
                c.line_number,
            );
        }
    }
    Ok(())
}
