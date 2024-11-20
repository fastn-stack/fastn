#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct OrType {
    pub name: String,
    pub variants: Vec<fastn_resolved::OrTypeVariant>,
    pub line_number: usize,
}

impl fastn_resolved::OrType {
    pub fn new(
        name: &str,
        variants: Vec<fastn_resolved::OrTypeVariant>,
        line_number: usize,
    ) -> fastn_resolved::OrType {
        fastn_resolved::OrType {
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
    AnonymousRecord(fastn_resolved::Record),
    Regular(fastn_resolved::Field),
    Constant(fastn_resolved::Field),
}

impl fastn_resolved::OrTypeVariant {
    pub fn new_record(record: fastn_resolved::Record) -> fastn_resolved::OrTypeVariant {
        fastn_resolved::OrTypeVariant::AnonymousRecord(record)
    }

    pub fn new_constant(variant: fastn_resolved::Field) -> fastn_resolved::OrTypeVariant {
        fastn_resolved::OrTypeVariant::Constant(variant)
    }

    pub fn new_regular(variant: fastn_resolved::Field) -> fastn_resolved::OrTypeVariant {
        fastn_resolved::OrTypeVariant::Regular(variant)
    }

    pub fn is_constant(&self) -> bool {
        matches!(self, fastn_resolved::OrTypeVariant::Constant(_))
    }

    pub fn name(&self) -> String {
        match self {
            fastn_resolved::OrTypeVariant::AnonymousRecord(ar) => ar.name.to_string(),
            fastn_resolved::OrTypeVariant::Regular(r) => r.name.to_string(),
            fastn_resolved::OrTypeVariant::Constant(c) => c.name.to_string(),
        }
    }

    pub fn line_number(&self) -> usize {
        match self {
            fastn_resolved::OrTypeVariant::AnonymousRecord(ar) => ar.line_number,
            fastn_resolved::OrTypeVariant::Regular(r) => r.line_number,
            fastn_resolved::OrTypeVariant::Constant(c) => c.line_number,
        }
    }

    pub fn fields(&self) -> Vec<&fastn_resolved::Field> {
        match self {
            fastn_resolved::OrTypeVariant::AnonymousRecord(r) => r.fields.iter().collect(),
            fastn_resolved::OrTypeVariant::Regular(r) => vec![r],
            fastn_resolved::OrTypeVariant::Constant(c) => vec![c],
        }
    }
}
