pub(crate) trait PropertyValueExt {
    fn to_property(&self, source: fastn_type::PropertySource) -> fastn_type::Property;
}

impl PropertyValueExt for fastn_type::PropertyValue {
    fn to_property(&self, source: fastn_type::PropertySource) -> fastn_type::Property {
        fastn_type::Property {
            value: self.clone(),
            source,
            condition: None,
            line_number: self.line_number(),
        }
    }
}

pub(crate) trait ComponentExt {
    fn get_children(
        &self,
        doc: &ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<Vec<fastn_type::ComponentInvocation>>;
    fn get_children_property(&self) -> Option<fastn_type::Property>;
    fn get_children_properties(&self) -> Vec<fastn_type::Property>;
    fn is_variable(&self) -> bool;
}

impl ComponentExt for fastn_type::ComponentInvocation {
    fn get_children(
        &self,
        doc: &ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<Vec<fastn_type::ComponentInvocation>> {
        use ftd::interpreter::PropertyValueExt;

        let property = if let Some(property) = self.get_children_property() {
            property
        } else {
            return Ok(vec![]);
        };

        let value = property.value.clone().resolve(doc, property.line_number)?;
        if let fastn_type::Value::UI { component, .. } = value {
            return Ok(vec![component]);
        }
        if let fastn_type::Value::List { data, kind } = value {
            if kind.is_ui() {
                let mut children = vec![];
                for value in data {
                    let value = value.resolve(doc, property.line_number)?;
                    if let fastn_type::Value::UI { component, .. } = value {
                        children.push(component);
                    }
                }
                return Ok(children);
            }
        }

        Ok(vec![])
    }

    fn get_children_property(&self) -> Option<fastn_type::Property> {
        self.get_children_properties().first().map(|v| v.to_owned())
    }

    fn get_children_properties(&self) -> Vec<fastn_type::Property> {
        ftd::interpreter::utils::get_children_properties_from_properties(&self.properties)
    }

    fn is_variable(&self) -> bool {
        self.source.eq(&fastn_type::ComponentSource::Variable)
    }
}

pub(crate) trait PropertySourceExt {
    fn header(name: &str) -> fastn_type::PropertySource;
}

impl PropertySourceExt for fastn_type::PropertySource {
    fn header(name: &str) -> fastn_type::PropertySource {
        fastn_type::PropertySource::Header {
            name: name.to_string(),
            mutable: false,
        }
    }
}
