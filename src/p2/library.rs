#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait Library: Sync {
    async fn get(&self, name: &str) -> Option<String>;
    async fn get_with_result(&self, name: &str) -> crate::p1::Result<String> {
        match self.get(name).await {
            Some(v) => Ok(v),
            None => ftd::e2(format!("library not found: {}", name), "TODO", 0), // TODO
        }
    }
    async fn process(
        &self,
        section: &ftd::p1::Section,
        doc: &ftd::p2::TDoc,
    ) -> crate::p1::Result<ftd::Value> {
        crate::unknown_processor_error(
            format!("unimplemented for section {:?} and doc {:?}", section, doc),
            doc.name.to_string(),
            section.line_number,
        )
    }
}

#[cfg(not(feature = "async"))]
pub trait Library {
    fn get(&self, name: &str) -> Option<String>;
    fn get_with_result(&self, name: &str) -> crate::p1::Result<String> {
        match self.get(name) {
            Some(v) => Ok(v),
            None => ftd::e2(format!("library not found: {}", name), "", 0),
        }
    }
    fn process(
        &self,
        section: &ftd::p1::Section,
        doc: &ftd::p2::TDoc,
    ) -> crate::p1::Result<ftd::Value> {
        crate::unknown_processor_error(
            format!("unimplemented for section {:?} and doc {:?}", section, doc),
            doc.name.to_string(),
            section.line_number,
        )
    }
}

pub struct TestLibrary {}

fn read_version() -> crate::p1::Result<ftd::Value> {
    let get =
        std::fs::read_to_string("./Cargo.toml").map_err(|e| crate::p1::Error::ParseError {
            message: e.to_string(),
            doc_id: "".to_string(),
            line_number: 0,
        })?;

    let version_string = "version";
    for line in get.split('\n') {
        if line.starts_with(version_string) {
            let mut part = line.splitn(2, '=');
            let _part_1 = part.next().unwrap().trim();
            let part_2 = part.next().unwrap().trim();
            return Ok(ftd::Value::String {
                text: part_2.to_string(),
                source: crate::TextSource::Header,
            });
        }
    }

    crate::unknown_processor_error("version not found", "".to_string(), 0)
}

fn read_package(section: &ftd::p1::Section, doc: &ftd::p2::TDoc) -> ftd::p1::Result<ftd::Value> {
    let var = crate::Variable::list_from_p1(section, doc)?;
    let get =
        std::fs::read_to_string("./Cargo.toml").map_err(|e| crate::p1::Error::ParseError {
            message: e.to_string(),
            doc_id: doc.name.to_string(),
            line_number: section.line_number,
        })?;
    if let crate::Value::List {
        kind:
            crate::p2::Kind::String {
                caption,
                body,
                default,
            },
        ..
    } = var.value
    {
        let mut data = vec![];
        for line in get.split('\n') {
            if line.is_empty() {
                break;
            }
            if line.contains('=') {
                let mut part = line.splitn(2, '=');
                let _part_1 = part.next().unwrap().trim();
                let part_2 = part.next().unwrap().trim();
                data.push(ftd::Value::String {
                    text: part_2.to_string(),
                    source: crate::TextSource::Header,
                });
            }
        }
        Ok(ftd::Value::List {
            data,
            kind: crate::p2::Kind::String {
                caption,
                body,
                default,
            },
        })
    } else {
        crate::unknown_processor_error(
            format!(
                "list should have 'string' kind, found {:?}",
                var.value.kind()
            ),
            doc.name.to_string(),
            section.line_number,
        )
    }
}

fn read_records(section: &ftd::p1::Section, doc: &ftd::p2::TDoc) -> ftd::p1::Result<ftd::Value> {
    let var = crate::Variable::list_from_p1(section, doc)?;
    let get =
        std::fs::read_to_string("./Cargo.toml").map_err(|e| crate::p1::Error::ParseError {
            message: e.to_string(),
            doc_id: doc.name.to_string(),
            line_number: section.line_number,
        })?;
    if let crate::Value::List {
        kind: crate::p2::Kind::Record { name },
        ..
    } = var.value.clone()
    {
        let rec = doc.get_record(section.line_number, name.as_str())?;
        let mut data = vec![];
        for line in get.split('\n') {
            if line.is_empty() {
                break;
            }
            if line.contains('=') {
                let mut fields: std::collections::BTreeMap<String, crate::PropertyValue> =
                    Default::default();
                let mut parts = line.splitn(2, '=');
                for (k, v) in &rec.fields {
                    let part = parts.next().unwrap().trim();
                    match v {
                        ftd::p2::Kind::String { .. } => {
                            fields.insert(
                                k.to_string(),
                                crate::PropertyValue::Value {
                                    value: crate::variable::Value::String {
                                        text: part.to_string(),
                                        source: crate::TextSource::Header,
                                    },
                                },
                            );
                        }
                        _ => unimplemented!(),
                    }
                }
                data.push(ftd::Value::Record {
                    name: rec.name.clone(),
                    fields,
                });
            }
        }
        Ok(ftd::Value::List {
            data,
            kind: ftd::p2::Kind::Record { name },
        })
    } else {
        crate::unknown_processor_error(
            format!(
                "list should have 'string' kind, found {:?}",
                var.value.kind()
            ),
            doc.name.to_string(),
            section.line_number,
        )
    }
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
impl Library for TestLibrary {
    async fn get(&self, name: &str) -> Option<String> {
        std::fs::read_to_string(format!("./tests/{}.ftd", name)).ok()
    }

    async fn process(
        &self,
        section: &ftd::p1::Section,
        doc: &ftd::p2::TDoc,
    ) -> crate::p1::Result<ftd::Value> {
        match section
            .header
            .str(doc.name, section.line_number, "$processor$")?
        {
            "read_version_from_cargo_toml" => read_version(),
            "read_package_from_cargo_toml" => read_package(section, doc),
            "read_package_records_from_cargo_toml" => read_records(section, doc),
            t => ftd::e2(
                format!("unknown processor: {}", t),
                doc.name,
                section.line_number.to_owned(),
            ),
        }
    }
}

#[cfg(not(feature = "async"))]
impl Library for TestLibrary {
    fn get(&self, name: &str) -> Option<String> {
        std::fs::read_to_string(format!("./tests/{}.ftd", name)).ok()
    }

    fn process(
        &self,
        section: &ftd::p1::Section,
        doc: &ftd::p2::TDoc,
    ) -> crate::p1::Result<ftd::Value> {
        match section
            .header
            .str(doc.name, section.line_number, "$processor$")?
        {
            "read_version_from_cargo_toml" => read_version(),
            "read_package_from_cargo_toml" => read_package(section, doc),
            "read_package_records_from_cargo_toml" => read_records(section, doc),
            t => ftd::e2(
                format!("unknown processor: {}", t),
                doc.name,
                section.line_number.to_owned(),
            ),
        }
    }
}
