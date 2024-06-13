#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct OrType {
    pub name: String,
    pub variants: Vec<OrTypeVariant>,
    pub line_number: usize,
}

pub const ORTYPE: &str = "or-type";

impl OrType {
    fn new(name: &str, variants: Vec<ftd_ast::OrTypeVariant>, line_number: usize) -> OrType {
        OrType {
            name: name.to_string(),
            variants,
            line_number,
        }
    }

    pub(crate) fn is_or_type(section: &ftd_p1::Section) -> bool {
        section.kind.as_ref().map_or(false, |s| s.eq(ORTYPE))
    }

    pub(crate) fn from_p1(section: &ftd_p1::Section, doc_id: &str) -> ftd_ast::Result<OrType> {
        if !Self::is_or_type(section) {
            return ftd_ast::parse_error(
                format!("Section is not or-type section, found `{:?}`", section),
                doc_id,
                section.line_number,
            );
        }
        let mut variants = vec![];
        for section in section.sub_sections.iter() {
            variants.push(OrTypeVariant::from_p1(section, doc_id)?);
        }

        Ok(OrType::new(
            section.name.as_str(),
            variants,
            section.line_number,
        ))
    }

    pub fn line_number(&self) -> usize {
        self.line_number
    }
}

impl ftd_ast::Field {
    pub(crate) fn from_p1(
        section: &ftd_p1::Section,
        doc_id: &str,
    ) -> ftd_ast::Result<ftd_ast::Field> {
        if !ftd_ast::VariableDefinition::is_variable_definition(section) {
            return ftd_ast::parse_error(
                format!(
                    "Section is not or-type variant section, found `{:?}`",
                    section
                ),
                doc_id,
                section.line_number,
            );
        }

        let kind = ftd_ast::VariableKind::get_kind(
            section.kind.as_ref().unwrap().as_str(),
            doc_id,
            section.line_number,
        )?;

        let value =
            ftd_ast::VariableValue::from_p1_with_modifier(section, doc_id, &kind, false)?.inner();

        Ok(ftd_ast::Field::new(
            section.name.trim_start_matches(ftd_ast::utils::REFERENCE),
            kind,
            ftd_ast::utils::is_variable_mutable(section.name.as_str()),
            value,
            section.line_number,
            Default::default(),
        ))
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum OrTypeVariant {
    AnonymousRecord(ftd_ast::Record),
    Regular(ftd_ast::Field),
    Constant(ftd_ast::Field),
}

impl OrTypeVariant {
    pub fn new_record(record: ftd_ast::Record) -> OrTypeVariant {
        OrTypeVariant::AnonymousRecord(record)
    }

    pub fn new_variant(variant: ftd_ast::Field) -> OrTypeVariant {
        OrTypeVariant::Regular(variant)
    }

    pub fn new_constant(variant: ftd_ast::Field) -> OrTypeVariant {
        OrTypeVariant::Constant(variant)
    }

    pub fn set_name(&mut self, name: &str) {
        let variant_name = match self {
            OrTypeVariant::AnonymousRecord(r) => &mut r.name,
            OrTypeVariant::Regular(f) => &mut f.name,
            OrTypeVariant::Constant(f) => &mut f.name,
        };
        *variant_name = name.to_string();
    }

    pub fn name(&self) -> String {
        match self {
            OrTypeVariant::AnonymousRecord(r) => r.name.to_string(),
            OrTypeVariant::Regular(f) => f.name.to_string(),
            OrTypeVariant::Constant(f) => f.name.to_string(),
        }
    }

    pub(crate) fn is_constant(section: &ftd_p1::Section) -> bool {
        section
            .name
            .starts_with(format!("{} ", ftd_ast::constants::CONSTANT).as_str())
    }

    pub fn from_p1(section: &ftd_p1::Section, doc_id: &str) -> ftd_ast::Result<OrTypeVariant> {
        if ftd_ast::Record::is_record(section) {
            Ok(OrTypeVariant::new_record(ftd_ast::Record::from_p1(
                section, doc_id,
            )?))
        } else if OrTypeVariant::is_constant(section) {
            let mut section = section.to_owned();
            section.name = section
                .name
                .trim_start_matches(ftd_ast::constants::CONSTANT)
                .trim()
                .to_string();
            Ok(OrTypeVariant::new_constant(ftd_ast::Field::from_p1(
                &section, doc_id,
            )?))
        } else {
            Ok(OrTypeVariant::new_constant(ftd_ast::Field::from_p1(
                section, doc_id,
            )?))
        }
    }
}
