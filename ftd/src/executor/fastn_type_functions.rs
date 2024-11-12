pub(crate) trait PropertyValueExt {
    fn into_property(&self, source: fastn_type::PropertySource) -> fastn_type::Property;
}

impl PropertyValueExt for fastn_type::PropertyValue {
    fn into_property(&self, source: fastn_type::PropertySource) -> fastn_type::Property {
        fastn_type::Property {
            value: self.clone(),
            source,
            condition: None,
            line_number: self.line_number(),
        }
    }
}
