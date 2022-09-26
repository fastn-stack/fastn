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
        let fields = Field::from_ast_fields(&record.fields, doc)?;
        Ok(Record::new(name.as_str(), fields, record.line_number))
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Field {
    name: String,
    kind: ftd::interpreter2::KindData,
    mutable: bool,
    value: Option<ftd::interpreter2::PropertyValue>,
    line_number: usize,
}

impl Field {
    fn from_ast_fields(
        fields: &[ftd::ast::Field],
        doc: &ftd::interpreter2::TDoc,
    ) -> ftd::interpreter2::Result<Vec<Field>> {
        let mut result = vec![];
        for field in fields {
            result.push(Field::from_ast_field(field, doc)?);
        }
        Ok(result)
    }

    fn from_ast_field(
        field: &ftd::ast::Field,
        doc: &ftd::interpreter2::TDoc,
    ) -> ftd::interpreter2::Result<Field> {
        let kind = ftd::interpreter2::KindData::from_ast_kind(&field.kind, doc, field.line_number)?;
        let value = field.value.as_ref().map_or(Ok(None), |v| {
            ftd::interpreter2::PropertyValue::from_ast_value_with_kind(v, doc, &kind).map(Some)
        })?;
        Ok(Field {
            name: field.name.to_string(),
            kind,
            mutable: field.mutable,
            value,
            line_number: field.line_number,
        })
    }
}
