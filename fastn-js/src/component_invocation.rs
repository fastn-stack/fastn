#[derive(Clone, Debug)]
pub struct Kernel {
    pub element_kind: ElementKind,
    pub name: String,
    pub parent: String,
}

impl Kernel {
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        pretty::RcDoc::text("let")
            .append(pretty::RcDoc::space())
            .append(pretty::RcDoc::text(self.name.clone()))
            .append(pretty::RcDoc::space())
            .append(pretty::RcDoc::text("="))
            .append(pretty::RcDoc::space())
            .append(pretty::RcDoc::text("fastn_dom.createKernel("))
            .append(pretty::RcDoc::text(format!("{},", self.parent.clone())))
            .append(pretty::RcDoc::space())
            .append(pretty::RcDoc::text(self.element_kind.to_js()))
            .append(pretty::RcDoc::text(");"))
    }

    pub fn from_component(component_name: &str, parent: &str, index: usize) -> Kernel {
        let element_kind = fastn_js::ElementKind::from_component_name(component_name);
        Kernel {
            element_kind,
            name: format!("i{index}"),
            parent: parent.to_string(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ElementKind {
    Row,
    Column,
    Integer,
    Decimal,
    Boolean,
    Text,
    Image,
    IFrame,
}

impl ElementKind {
    pub(crate) fn from_component_name(name: &str) -> ElementKind {
        match name {
            "ftd#text" => ElementKind::Text,
            "ftd#row" => ElementKind::Row,
            "ftd#column" => ElementKind::Column,
            _ => todo!(),
        }
    }

    fn to_js(&self) -> &'static str {
        match self {
            ElementKind::Row => "fastn_dom.ElementKind.Row",
            ElementKind::Column => "fastn_dom.ElementKind.Column",
            ElementKind::Integer => "fastn_dom.ElementKind.Integer",
            ElementKind::Decimal => "fastn_dom.ElementKind.Decimal",
            ElementKind::Boolean => "fastn_dom.ElementKind.Boolean",
            ElementKind::Text => "fastn_dom.ElementKind.Text",
            ElementKind::Image => "fastn_dom.ElementKind.Image",
            ElementKind::IFrame => "fastn_dom.ElementKind.IFrame",
        }
    }
}

/*pub struct ComponentInvocation {
    pub name: String,
    pub arguments: Vec<String>,
    pub parent: String,
}*/

/*impl ComponentInvocation {
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        let mut js = pretty::RcDoc::text(format!("{}(", self.name))
            .append(pretty::RcDoc::text(self.parent.clone()));
        for argument in self.arguments.iter() {
            js = js
                .append(pretty::RcDoc::text(","))
                .append(pretty::RcDoc::space())
                .append(pretty::RcDoc::text(argument));
        }
        js
    }
}*/
