#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
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
        let fields = Field::from_ast_fields(record.fields, doc)?;
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
    fn from_ast_fields(
        fields: Vec<ftd::ast::Field>,
        doc: &ftd::interpreter2::TDoc,
    ) -> ftd::interpreter2::Result<Vec<Field>> {
        let mut result = vec![];
        for field in fields {
            result.push(Field::from_ast_field(field, doc)?);
        }
        Ok(result)
    }

    fn from_ast_field(
        field: ftd::ast::Field,
        doc: &ftd::interpreter2::TDoc,
    ) -> ftd::interpreter2::Result<Field> {
        let kind = ftd::interpreter2::KindData::from_ast_kind(field.kind, doc, field.line_number)?;
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

    pub fn is_body(&self) -> bool {
        self.kind.body
    }
}
