#[allow(clippy::too_many_arguments)]
pub fn arguments(
    _arguments: &[fastn_resolved::Argument],
    _caption: &mut fastn_unresolved::UR<Option<fastn_section::HeaderValue>, ()>,
    _properties: &mut [fastn_unresolved::UR<
        fastn_unresolved::Property,
        fastn_resolved::Property,
    >],
    _body: &mut fastn_unresolved::UR<Vec<fastn_section::Tes>, ()>,
    _children: &[fastn_unresolved::UR<
        fastn_unresolved::ComponentInvocation,
        fastn_resolved::ComponentInvocation,
    >],
    _definitions: &std::collections::HashMap<String, fastn_unresolved::URD>,
    _modules: &std::collections::HashMap<fastn_unresolved::Module, bool>,
    _arena: &mut fastn_unresolved::Arena,
    _output: &mut fastn_unresolved::resolver::Output,
) {
}
