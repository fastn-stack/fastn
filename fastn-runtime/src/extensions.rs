pub trait ComponentDefinitionExt {
    fn to_ast(
        &self,
        doc: &dyn fastn_resolved::tdoc::TDoc,
        has_rive_components: &mut bool,
    ) -> fastn_js::Ast;
}

pub trait FunctionExt {
    fn to_ast(&self, doc: &dyn fastn_resolved::tdoc::TDoc) -> fastn_js::Ast;
}

pub trait ComponentExt {
    fn to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &dyn fastn_resolved::tdoc::TDoc,
        rdata: &fastn_runtime::ResolverData,
        should_return: bool,
        has_rive_components: &mut bool,
    ) -> Vec<fastn_js::ComponentStatement>;
    fn to_component_statements_(
        &self,
        parent: &str,
        index: usize,
        doc: &dyn fastn_resolved::tdoc::TDoc,
        rdata: &fastn_runtime::ResolverData,
        should_return: bool,
        has_rive_components: &mut bool,
    ) -> Vec<fastn_js::ComponentStatement>;
    fn kernel_to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &dyn fastn_resolved::tdoc::TDoc,
        rdata: &fastn_runtime::ResolverData,
        should_return: bool,
        has_rive_components: &mut bool,
    ) -> Option<Vec<fastn_js::ComponentStatement>>;
    fn defined_component_to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &dyn fastn_resolved::tdoc::TDoc,
        rdata: &fastn_runtime::ResolverData,
        should_return: bool,
        has_rive_components: &mut bool,
    ) -> Option<Vec<fastn_js::ComponentStatement>>;
    fn header_defined_component_to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &dyn fastn_resolved::tdoc::TDoc,
        rdata: &fastn_runtime::ResolverData,
        should_return: bool,
        has_rive_components: &mut bool,
    ) -> Option<Vec<fastn_js::ComponentStatement>>;
    fn variable_defined_component_to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &dyn fastn_resolved::tdoc::TDoc,
        rdata: &fastn_runtime::ResolverData,
        should_return: bool,
        has_rive_components: &mut bool,
    ) -> Option<Vec<fastn_js::ComponentStatement>>;
    fn is_loop(&self) -> bool;
}

pub(crate) trait EventNameExt {
    fn to_js_event_name(&self) -> Option<fastn_js::Event>;
}

pub(crate) trait EventExt {
    fn to_event_handler_js(
        &self,
        element_name: &str,
        doc: &dyn fastn_resolved::tdoc::TDoc,
        rdata: &fastn_runtime::ResolverData,
    ) -> Option<fastn_js::EventHandler>;
}

pub(crate) trait ValueExt {
    fn to_fastn_js_value(
        &self,
        doc: &dyn fastn_resolved::tdoc::TDoc,
        rdata: &fastn_runtime::ResolverData,
        has_rive_components: &mut bool,
        should_return: bool,
    ) -> fastn_js::SetPropertyValue;
}

pub trait PropertyValueExt {
    fn get_deps(&self, rdata: &fastn_runtime::ResolverData) -> Vec<String>;

    fn to_fastn_js_value_with_none(
        &self,
        doc: &dyn fastn_resolved::tdoc::TDoc,
        has_rive_components: &mut bool,
    ) -> fastn_js::SetPropertyValue;

    fn to_fastn_js_value(
        &self,
        doc: &dyn fastn_resolved::tdoc::TDoc,
        rdata: &fastn_runtime::ResolverData,
        should_return: bool,
    ) -> fastn_js::SetPropertyValue;

    fn to_fastn_js_value_with_ui(
        &self,
        doc: &dyn fastn_resolved::tdoc::TDoc,
        rdata: &fastn_runtime::ResolverData,
        has_rive_components: &mut bool,
        is_ui_component: bool,
    ) -> fastn_js::SetPropertyValue;

    fn to_value(&self) -> fastn_runtime::Value;
}

pub(crate) trait FunctionCallExt {
    fn to_js_function(
        &self,
        doc: &dyn fastn_resolved::tdoc::TDoc,
        rdata: &fastn_runtime::ResolverData,
    ) -> fastn_js::Function;
}

pub(crate) trait ExpressionExt {
    fn get_deps(&self, rdata: &fastn_runtime::ResolverData) -> Vec<String>;
    fn update_node_with_variable_reference_js(
        &self,
        rdata: &fastn_runtime::ResolverData,
    ) -> fastn_resolved::evalexpr::ExprNode;
}

pub(crate) trait ArgumentExt {
    fn get_default_value(&self) -> Option<fastn_runtime::Value>;
    fn get_optional_value(
        &self,
        properties: &[fastn_resolved::Property],
        // doc_name: &str,
        // line_number: usize
    ) -> Option<fastn_runtime::Value>;
}

pub trait WebComponentDefinitionExt {
    fn to_ast(&self, doc: &dyn fastn_resolved::tdoc::TDoc) -> fastn_js::Ast;
}

pub trait VariableExt {
    fn to_ast(
        &self,
        doc: &dyn fastn_resolved::tdoc::TDoc,
        prefix: Option<String>,
        has_rive_components: &mut bool,
    ) -> fastn_js::Ast;
}
