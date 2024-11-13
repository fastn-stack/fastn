#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct OrType {
    pub name: String,
    pub variants: Vec<fastn_type::OrTypeVariant>,
    pub line_number: usize,
}

impl fastn_type::OrType {
    pub fn new(
        name: &str,
        variants: Vec<fastn_type::OrTypeVariant>,
        line_number: usize,
    ) -> fastn_type::OrType {
        fastn_type::OrType {
            name: name.to_string(),
            variants,
            line_number,
        }
    }

    pub fn or_type_name(name: &str) -> String {
        if name.starts_with("ftd") {
            return name.to_string();
        }
        if let Some((_, last)) = name.rsplit_once('#') {
            return last.to_string();
        }
        name.to_string()
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum OrTypeVariant {
    AnonymousRecord(fastn_type::Record),
    Regular(fastn_type::Field),
    Constant(fastn_type::Field),
}

impl fastn_type::OrTypeVariant {
    pub fn new_record(record: fastn_type::Record) -> fastn_type::OrTypeVariant {
        fastn_type::OrTypeVariant::AnonymousRecord(record)
    }

    pub fn new_constant(variant: fastn_type::Field) -> fastn_type::OrTypeVariant {
        fastn_type::OrTypeVariant::Constant(variant)
    }

    pub fn new_regular(variant: fastn_type::Field) -> fastn_type::OrTypeVariant {
        fastn_type::OrTypeVariant::Regular(variant)
    }

    pub fn is_constant(&self) -> bool {
        matches!(self, fastn_type::OrTypeVariant::Constant(_))
    }

    pub fn name(&self) -> String {
        match self {
            fastn_type::OrTypeVariant::AnonymousRecord(ar) => ar.name.to_string(),
            fastn_type::OrTypeVariant::Regular(r) => r.name.to_string(),
            fastn_type::OrTypeVariant::Constant(c) => c.name.to_string(),
        }
    }

    pub fn line_number(&self) -> usize {
        match self {
            fastn_type::OrTypeVariant::AnonymousRecord(ar) => ar.line_number,
            fastn_type::OrTypeVariant::Regular(r) => r.line_number,
            fastn_type::OrTypeVariant::Constant(c) => c.line_number,
        }
    }

    pub fn fields(&self) -> Vec<&fastn_type::Field> {
        match self {
            fastn_type::OrTypeVariant::AnonymousRecord(r) => r.fields.iter().collect(),
            fastn_type::OrTypeVariant::Regular(r) => vec![r],
            fastn_type::OrTypeVariant::Constant(c) => vec![c],
        }
    }
}
