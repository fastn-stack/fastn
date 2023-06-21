impl fastn_js::Ast {
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        match self {
            fastn_js::Ast::Component(f) => f.to_js(),
            fastn_js::Ast::UDF(f) => f.to_js(),
        }
    }
}

impl fastn_js::Kernel {
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
}

impl fastn_js::ElementKind {
    pub fn to_js(&self) -> &'static str {
        match self {
            fastn_js::ElementKind::Row => "fastn_dom.ElementKind.Row",
            fastn_js::ElementKind::Column => "fastn_dom.ElementKind.Column",
            fastn_js::ElementKind::Integer => "fastn_dom.ElementKind.Integer",
            fastn_js::ElementKind::Decimal => "fastn_dom.ElementKind.Decimal",
            fastn_js::ElementKind::Boolean => "fastn_dom.ElementKind.Boolean",
            fastn_js::ElementKind::Text => "fastn_dom.ElementKind.Text",
            fastn_js::ElementKind::Image => "fastn_dom.ElementKind.Image",
            fastn_js::ElementKind::IFrame => "fastn_dom.ElementKind.IFrame",
        }
    }
}
impl fastn_js::ComponentStatement {
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        match self {
            fastn_js::ComponentStatement::StaticVariable(f) => f.to_js(),
            fastn_js::ComponentStatement::MutableVariable(f) => f.to_js(),
            fastn_js::ComponentStatement::CreateKernel(kernel) => kernel.to_js(),
            fastn_js::ComponentStatement::Done { component_name } => {
                pretty::RcDoc::text(format!("{component_name}.done();"))
            }
        }
    }
}

impl fastn_js::MutableVariable {
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        pretty::RcDoc::text("let")
            .append(pretty::RcDoc::space())
            .append(pretty::RcDoc::text(self.name.clone()))
            .append(pretty::RcDoc::space())
            .append(pretty::RcDoc::text("="))
            .append(pretty::RcDoc::space())
            .append(pretty::RcDoc::text("fastn.mutable("))
            .append(if self.is_quoted {
                pretty::RcDoc::text("\"")
                    .append(pretty::RcDoc::text(self.value.replace("\n", "\\n")))
                    .append(pretty::RcDoc::text("\""))
            } else {
                pretty::RcDoc::text(self.value.clone())
            })
            .append(pretty::RcDoc::text(");"))
    }
}
