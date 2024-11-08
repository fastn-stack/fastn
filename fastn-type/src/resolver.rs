#[derive(Debug, Clone)]
pub struct ResolverData<'a> {
    pub component_definition_name: &'a Option<String>,
    pub record_definition_name: &'a Option<String>,
    pub component_name: Option<String>,
    pub loop_alias: &'a Option<String>,
    pub loop_counter_alias: &'a Option<String>,
    pub inherited_variable_name: &'a str,
    pub device: &'a Option<fastn_js::DeviceType>,
    pub doc_name: Option<String>,
}

impl<'a> ResolverData<'a> {
    pub(crate) fn none() -> ResolverData<'a> {
        ResolverData {
            component_definition_name: &None,
            record_definition_name: &None,
            component_name: None,
            loop_alias: &None,
            loop_counter_alias: &None,
            inherited_variable_name: fastn_js::INHERITED_VARIABLE,
            device: &None,
            doc_name: None,
        }
    }

    pub(crate) fn new_with_component_definition_name(
        component_definition_name: &'a Option<String>,
    ) -> ResolverData<'a> {
        let mut rdata = ResolverData::none();
        rdata.component_definition_name = component_definition_name;
        rdata
    }

    pub(crate) fn clone_with_default_inherited_variable(&self) -> ResolverData<'a> {
        ResolverData {
            component_definition_name: self.component_definition_name,
            record_definition_name: self.record_definition_name,
            component_name: self.component_name.clone(),
            loop_alias: self.loop_alias,
            loop_counter_alias: self.loop_counter_alias,
            inherited_variable_name: fastn_js::INHERITED_VARIABLE,
            device: self.device,
            doc_name: self.doc_name.clone(),
        }
    }

    pub(crate) fn clone_with_new_inherited_variable(
        &self,
        inherited_variable_name: &'a str,
    ) -> ResolverData<'a> {
        ResolverData {
            component_definition_name: self.component_definition_name,
            record_definition_name: self.record_definition_name,
            component_name: self.component_name.clone(),
            loop_alias: self.loop_alias,
            loop_counter_alias: self.loop_counter_alias,
            inherited_variable_name,
            device: self.device,
            doc_name: self.doc_name.clone(),
        }
    }

    pub(crate) fn clone_with_new_component_name(
        &self,
        component_name: Option<String>,
    ) -> ResolverData<'a> {
        ResolverData {
            component_definition_name: self.component_definition_name,
            record_definition_name: self.record_definition_name,
            component_name,
            loop_alias: self.loop_alias,
            loop_counter_alias: self.loop_counter_alias,
            inherited_variable_name: self.inherited_variable_name,
            device: self.device,
            doc_name: self.doc_name.clone(),
        }
    }

    pub(crate) fn clone_with_new_device(
        &self,
        device: &'a Option<fastn_js::DeviceType>,
    ) -> ResolverData<'a> {
        ResolverData {
            component_definition_name: self.component_definition_name,
            record_definition_name: self.record_definition_name,
            component_name: self.component_name.clone(),
            loop_alias: self.loop_alias,
            loop_counter_alias: self.loop_counter_alias,
            inherited_variable_name: self.inherited_variable_name,
            device,
            doc_name: self.doc_name.clone(),
        }
    }

    pub(crate) fn clone_with_new_loop_alias(
        &self,
        loop_alias: &'a Option<String>,
        loop_counter_alias: &'a Option<String>,
        doc_name: String,
    ) -> ResolverData<'a> {
        ResolverData {
            component_definition_name: self.component_definition_name,
            record_definition_name: self.record_definition_name,
            component_name: self.component_name.clone(),
            loop_alias,
            loop_counter_alias,
            inherited_variable_name: self.inherited_variable_name,
            device: self.device,
            doc_name: Some(doc_name),
        }
    }

    pub(crate) fn clone_with_new_record_definition_name(
        &self,
        record_definition_name: &'a Option<String>,
    ) -> ResolverData<'a> {
        ResolverData {
            component_definition_name: self.component_definition_name,
            record_definition_name,
            component_name: self.component_name.clone(),
            loop_alias: self.loop_alias,
            loop_counter_alias: self.loop_counter_alias,
            inherited_variable_name: self.inherited_variable_name,
            device: self.device,
            doc_name: self.doc_name.clone(),
        }
    }
}
