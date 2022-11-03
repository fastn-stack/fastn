#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Record {
    pub name: String,
    pub fields: Vec<Field>,
    pub line_number: usize,
}

impl Record {
    fn new(name: &str, fields: Vec<Field>, line_number: usize) -> Record {
        Record {
            name: name.to_string(),
            fields,
            line_number,
        }
    }
    pub(crate) fn from_ast(
        ast: ftd::ast::AST,
        doc: &ftd::interpreter2::TDoc,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::Record> {
        let record = ast.get_record(doc.name)?;
        let name = doc.resolve_name(record.name.as_str());
        let known_kinds = std::iter::IntoIterator::into_iter([(
            record.name.to_string(),
            ftd::interpreter2::Kind::Record {
                name: name.to_string(),
            },
        )])
        .collect::<ftd::Map<ftd::interpreter2::Kind>>();
        let fields = Field::from_ast_fields(record.fields, doc, &known_kinds)?;
        Ok(Record::new(name.as_str(), fields, record.line_number))
    }

    pub(crate) fn get_field(
        &self,
        name: &str,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter2::Result<&Field> {
        use itertools::Itertools;

        let field = self.fields.iter().filter(|v| v.name.eq(name)).collect_vec();
        if field.is_empty() {
            return ftd::interpreter2::utils::e2(
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
            return ftd::interpreter2::utils::e2(
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

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Field {
    pub name: String,
    pub kind: ftd::interpreter2::KindData,
    pub mutable: bool,
    pub value: Option<ftd::interpreter2::PropertyValue>,
    pub line_number: usize,
}

impl Field {
    pub fn new(
        name: &str,
        kind: ftd::interpreter2::KindData,
        mutable: bool,
        value: Option<ftd::interpreter2::PropertyValue>,
        line_number: usize,
    ) -> Field {
        Field {
            name: name.to_string(),
            kind,
            mutable,
            value,
            line_number,
        }
    }

    pub fn to_sources(&self) -> Vec<ftd::interpreter2::PropertySource> {
        let mut sources = vec![ftd::interpreter2::PropertySource::Header {
            name: self.name.to_string(),
            mutable: self.mutable,
        }];
        if self.is_caption() {
            sources.push(ftd::interpreter2::PropertySource::Caption);
        }

        if self.is_body() {
            sources.push(ftd::interpreter2::PropertySource::Body);
        }

        if self.is_subsection_ui() {
            sources.push(ftd::interpreter2::PropertySource::Subsection);
        }

        sources
    }

    pub fn default(name: &str, kind: ftd::interpreter2::KindData) -> Field {
        Field {
            name: name.to_string(),
            kind,
            mutable: false,
            value: None,
            line_number: 0,
        }
    }

    pub(crate) fn from_ast_fields(
        fields: Vec<ftd::ast::Field>,
        doc: &ftd::interpreter2::TDoc,
        known_kinds: &ftd::Map<ftd::interpreter2::Kind>,
    ) -> ftd::interpreter2::Result<Vec<Field>> {
        let mut result = vec![];
        for field in fields {
            result.push(Field::from_ast_field(field, doc, known_kinds)?);
        }
        Ok(result)
    }

    pub(crate) fn from_ast_field(
        field: ftd::ast::Field,
        doc: &ftd::interpreter2::TDoc,
        known_kinds: &ftd::Map<ftd::interpreter2::Kind>,
    ) -> ftd::interpreter2::Result<Field> {
        let kind = ftd::interpreter2::KindData::from_ast_kind(
            field.kind,
            known_kinds,
            doc,
            field.line_number,
        )?;
        let value = field.value.map_or(Ok(None), |v| {
            ftd::interpreter2::PropertyValue::from_ast_value(v, doc, field.mutable, Some(&kind))
                .map(Some)
        })?;
        Ok(Field {
            name: field.name.to_string(),
            kind,
            mutable: field.mutable,
            value,
            line_number: field.line_number,
        })
    }

    pub fn is_caption(&self) -> bool {
        self.kind.caption
    }

    pub fn is_subsection_ui(&self) -> bool {
        self.kind.kind.clone().inner_list().is_subsection_ui()
    }

    pub fn is_body(&self) -> bool {
        self.kind.body
    }

    pub(crate) fn for_component(
        component_name: &str,
        definition_name_with_arguments: &Option<(&str, &[Field])>,
        doc: &ftd::interpreter2::TDoc,
        line_number: usize,
    ) -> ftd::interpreter2::Result<Vec<Field>> {
        Ok(match definition_name_with_arguments {
            Some((name, arg)) if name.eq(&component_name) => arg.to_vec(),
            _ => doc.get_component(component_name, line_number)?.arguments,
        })
    }
}
