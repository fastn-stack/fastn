pub(crate) trait PropertyValueExt {
    fn to_property(&self, source: fastn_resolved::PropertySource) -> fastn_resolved::Property;
}

impl PropertyValueExt for fastn_resolved::PropertyValue {
    fn to_property(&self, source: fastn_resolved::PropertySource) -> fastn_resolved::Property {
        fastn_resolved::Property {
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
    ) -> ftd::interpreter::Result<Vec<fastn_resolved::ComponentInvocation>>;
    fn get_children_property(&self) -> Option<fastn_resolved::Property>;
    fn get_children_properties(&self) -> Vec<fastn_resolved::Property>;
    fn is_variable(&self) -> bool;
}

impl ComponentExt for fastn_resolved::ComponentInvocation {
    fn get_children(
        &self,
        doc: &ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<Vec<fastn_resolved::ComponentInvocation>> {
        use ftd::interpreter::PropertyValueExt;

        let property = if let Some(property) = self.get_children_property() {
            property
        } else {
            return Ok(vec![]);
        };

        let value = property.value.clone().resolve(doc, property.line_number)?;
        if let fastn_resolved::Value::UI { component, .. } = value {
            return Ok(vec![component]);
        }
        if let fastn_resolved::Value::List { data, kind } = value
            && kind.is_ui()
        {
            let mut children = vec![];
            for value in data {
                let value = value.resolve(doc, property.line_number)?;
                if let fastn_resolved::Value::UI { component, .. } = value {
                    children.push(component);
                }
            }
            return Ok(children);
        }

        Ok(vec![])
    }

    fn get_children_property(&self) -> Option<fastn_resolved::Property> {
        self.get_children_properties().first().map(|v| v.to_owned())
    }

    fn get_children_properties(&self) -> Vec<fastn_resolved::Property> {
        ftd::interpreter::utils::get_children_properties_from_properties(&self.properties)
    }

    fn is_variable(&self) -> bool {
        self.source.eq(&fastn_resolved::ComponentSource::Variable)
    }
}

pub(crate) trait PropertySourceExt {
    fn header(name: &str) -> fastn_resolved::PropertySource;
}

impl PropertySourceExt for fastn_resolved::PropertySource {
    fn header(name: &str) -> fastn_resolved::PropertySource {
        fastn_resolved::PropertySource::Header {
            name: name.to_string(),
            mutable: false,
        }
    }
}
