pub(crate) trait PropertyValueExt {
    fn into_property(&self, source: ftd::interpreter::PropertySource)
        -> ftd::interpreter::Property;
}

impl PropertyValueExt for fastn_type::PropertyValue {
    fn into_property(
        &self,
        source: ftd::interpreter::PropertySource,
    ) -> ftd::interpreter::Property {
        ftd::interpreter::Property {
            value: self.clone(),
            source,
            condition: None,
            line_number: self.line_number(),
        }
    }
}
