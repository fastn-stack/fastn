pub struct Component;

pub const COMPONENT: &str = "component";

impl Component {
    pub fn is_component(section: &ftd::p11::Section) -> bool {
        section.kind.as_ref().map_or(false, |s| s.eq(COMPONENT))
    }
}
