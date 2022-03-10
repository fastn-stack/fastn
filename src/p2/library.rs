#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait Library: Sync {
    async fn get(&self, name: &str, doc: &ftd::p2::TDoc) -> Option<String>;
    async fn get_with_result(&self, name: &str, doc: &ftd::p2::TDoc) -> ftd::p1::Result<String> {
        match self.get(name, doc).await {
            Some(v) => Ok(v),
            None => ftd::e2(format!("library not found: {}", name), "", 0),
        }
    }
    async fn process(
        &self,
        section: &ftd::p1::Section,
        doc: &ftd::p2::TDoc,
    ) -> ftd::p1::Result<ftd::Value> {
        ftd::unknown_processor_error(
            format!("unimplemented for section {:?} and doc {:?}", section, doc),
            doc.name.to_string(),
            section.line_number,
        )
    }
}

#[cfg(not(feature = "async"))]
pub trait Library {
    fn get(&self, name: &str, doc: &ftd::p2::TDoc) -> Option<String>;
    fn get_with_result(&self, name: &str, doc: &ftd::p2::TDoc) -> ftd::p1::Result<String> {
        match self.get(name, doc) {
            Some(v) => Ok(v),
            None => ftd::e2(format!("library not found: {}", name), "", 0),
        }
    }
    fn process(
        &self,
        section: &ftd::p1::Section,
        doc: &ftd::p2::TDoc,
    ) -> ftd::p1::Result<ftd::Value> {
        ftd::unknown_processor_error(
            format!("unimplemented for section {:?} and doc {:?}", section, doc),
            doc.name.to_string(),
            section.line_number,
        )
    }
}

pub struct TestLibrary {}

fn read_version() -> ftd::p1::Result<ftd::Value> {
    let get = std::fs::read_to_string("./Cargo.toml").map_err(|e| ftd::p1::Error::ParseError {
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
                source: ftd::TextSource::Header,
            });
        }
    }

    ftd::unknown_processor_error("version not found", "".to_string(), 0)
}

fn read_package(section: &ftd::p1::Section, doc: &ftd::p2::TDoc) -> ftd::p1::Result<ftd::Value> {
    let var = ftd::Variable::list_from_p1(section, doc)?;
    let get = std::fs::read_to_string("./Cargo.toml").map_err(|e| ftd::p1::Error::ParseError {
        message: e.to_string(),
        doc_id: doc.name.to_string(),
        line_number: section.line_number,
    })?;
    if let Ok(ftd::Value::List {
        kind:
            ftd::p2::Kind::String {
                caption,
                body,
                default,
            },
        ..
    }) = var.value.resolve(section.line_number, doc)
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
                data.push(ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: part_2.to_string(),
                        source: ftd::TextSource::Header,
                    },
                });
            }
        }
        Ok(ftd::Value::List {
            data,
            kind: ftd::p2::Kind::String {
                caption,
                body,
                default,
            },
        })
    } else {
        ftd::unknown_processor_error(
            format!(
                "list should have 'string' kind, found {:?}",
                var.value.kind()
            ),
            doc.name.to_string(),
            section.line_number,
        )
    }
}

fn text_component() -> ftd::p1::Result<ftd::Value> {
    let mut v: std::collections::BTreeMap<String, ftd::PropertyValue> = Default::default();
    v.insert(
        "$caption$".to_string(),
        ftd::PropertyValue::Value {
            value: ftd::variable::Value::String {
                text: "Hello from text-component processor".to_string(),
                source: ftd::TextSource::Header,
            },
        },
    );
    v.insert(
        "line-clamp".to_string(),
        ftd::PropertyValue::Value {
            value: ftd::variable::Value::Integer { value: 40 },
        },
    );
    Ok(ftd::Value::Object { values: v })
}

fn read_records(section: &ftd::p1::Section, doc: &ftd::p2::TDoc) -> ftd::p1::Result<ftd::Value> {
    let var = ftd::Variable::list_from_p1(section, doc)?;
    let get = std::fs::read_to_string("./Cargo.toml").map_err(|e| ftd::p1::Error::ParseError {
        message: e.to_string(),
        doc_id: doc.name.to_string(),
        line_number: section.line_number,
    })?;
    if let Ok(ftd::Value::List {
        kind: ftd::p2::Kind::Record { name, .. },
        ..
    }) = var.value.resolve(section.line_number, doc)
    {
        let rec = doc.get_record(section.line_number, name.as_str())?;
        let mut data = vec![];
        for line in get.split('\n') {
            if line.is_empty() {
                break;
            }
            if line.contains('=') {
                let mut fields: std::collections::BTreeMap<String, ftd::PropertyValue> =
                    Default::default();
                let mut parts = line.splitn(2, '=');
                for (k, v) in &rec.fields {
                    let part = parts.next().unwrap().trim();
                    match v {
                        ftd::p2::Kind::String { .. } => {
                            fields.insert(
                                k.to_string(),
                                ftd::PropertyValue::Value {
                                    value: ftd::variable::Value::String {
                                        text: part.to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            );
                        }
                        _ => unimplemented!(),
                    }
                }
                data.push(ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: rec.name.clone(),
                        fields,
                    },
                });
            }
        }
        Ok(ftd::Value::List {
            data,
            kind: ftd::p2::Kind::Record {
                name,
                default: None,
            },
        })
    } else {
        ftd::unknown_processor_error(
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
    async fn get(&self, name: &str, _doc: &ftd::p2::TDoc) -> Option<String> {
        std::fs::read_to_string(format!("./tests/{}.ftd", name)).ok()
    }

    async fn process(
        &self,
        section: &ftd::p1::Section,
        doc: &ftd::p2::TDoc,
    ) -> ftd::p1::Result<ftd::Value> {
        match section
            .header
            .str(doc.name, section.line_number, "$processor$")?
        {
            "read_version_from_cargo_toml" => read_version(),
            "read_package_from_cargo_toml" => read_package(section, doc),
            "read_package_records_from_cargo_toml" => read_records(section, doc),
            "text-component-processor" => text_component(),
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
    fn get(&self, name: &str, _doc: &ftd::p2::TDoc) -> Option<String> {
        std::fs::read_to_string(format!("./tests/{}.ftd", name)).ok()
    }

    fn process(
        &self,
        section: &ftd::p1::Section,
        doc: &ftd::p2::TDoc,
    ) -> ftd::p1::Result<ftd::Value> {
        match section
            .header
            .str(doc.name, section.line_number, "$processor$")?
        {
            "read_version_from_cargo_toml" => read_version(),
            "read_package_from_cargo_toml" => read_package(section, doc),
            "read_package_records_from_cargo_toml" => read_records(section, doc),
            "text-component-processor" => text_component(),
            t => ftd::e2(
                format!("unknown processor: {}", t),
                doc.name,
                section.line_number.to_owned(),
            ),
        }
    }
}
