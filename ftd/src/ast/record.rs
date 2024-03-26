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

    pub(crate) fn is_record(section: &ftd_p1::Section) -> bool {
        section
            .kind
            .as_ref()
            .map_or(false, |s| s.eq(ftd::ast::constants::RECORD))
    }

    pub(crate) fn from_p1(section: &ftd_p1::Section, doc_id: &str) -> ftd::ast::Result<Record> {
        if !Self::is_record(section) {
            return ftd::ast::parse_error(
                format!("Section is not record section, found `{:?}`", section),
                doc_id,
                section.line_number,
            );
        }

        let fields = get_fields_from_headers(&section.headers, doc_id)?;
        Ok(Record::new(
            section.name.as_str(),
            fields,
            section.line_number,
        ))
    }

    pub fn line_number(&self) -> usize {
        self.line_number
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Field {
    pub name: String,
    pub kind: ftd::ast::VariableKind,
    pub mutable: bool,
    pub value: Option<ftd::ast::VariableValue>,
    pub line_number: usize,
    pub access_modifier: ftd_p1::AccessModifier,
}

impl Field {
    fn is_field(header: &ftd_p1::Header) -> bool {
        header.get_kind().is_some()
    }

    pub(crate) fn from_header(header: &ftd_p1::Header, doc_id: &str) -> ftd::ast::Result<Field> {
        if !Self::is_field(header) {
            return ftd::ast::parse_error(
                format!("Header is not argument, found `{:?}`", header),
                doc_id,
                header.get_line_number(),
            );
        }

        let kind = ftd::ast::VariableKind::get_kind(
            header.get_kind().as_ref().unwrap().as_str(),
            doc_id,
            header.get_line_number(),
        )?;

        let value =
            ftd::ast::VariableValue::from_header_with_modifier(header, doc_id, &kind)?.inner();

        let name = header.get_key();

        Ok(Field::new(
            name.trim_start_matches(ftd::ast::utils::REFERENCE),
            kind,
            ftd::ast::utils::is_variable_mutable(name.as_str()),
            value,
            header.get_line_number(),
            header.get_access_modifier(),
        ))
    }

    pub(crate) fn new(
        name: &str,
        kind: ftd::ast::VariableKind,
        mutable: bool,
        value: Option<ftd::ast::VariableValue>,
        line_number: usize,
        access_modifier: ftd_p1::AccessModifier,
    ) -> Field {
        Field {
            name: name.to_string(),
            kind,
            mutable,
            value,
            line_number,
            access_modifier,
        }
    }
}

pub(crate) fn get_fields_from_headers(
    headers: &ftd_p1::Headers,
    doc_id: &str,
) -> ftd::ast::Result<Vec<Field>> {
    let mut fields: Vec<Field> = Default::default();
    for header in headers.0.iter() {
        fields.push(Field::from_header(header, doc_id)?);
    }
    Ok(fields)
}
