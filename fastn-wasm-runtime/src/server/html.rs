pub fn node(
    tag: &'static str,
    node_key: fastn_runtime::NodeKey,
    attrs: Option<pretty::RcDoc<'static>>,
    body: pretty::RcDoc<'static>,
) -> pretty::RcDoc<'static> {
    let g1 = pretty::RcDoc::text("<")
        .append(tag)
        .append(pretty::RcDoc::space())
        .append("data-id=")
        .append(fastn_runtime::dom::node_key_to_id(node_key));
    let attrs = match attrs {
        Some(v) => v.append(">"),
        None => pretty::RcDoc::text(">"),
    };
    pretty::RcDoc::intersperse(vec![g1, attrs, body], pretty::RcDoc::space())
        .append("</")
        .append(tag)
        .append(">")
}

pub fn leaf(
    tag: &'static str,
    node_key: fastn_runtime::NodeKey,
    attrs: Option<pretty::RcDoc<'static>>,
) -> pretty::RcDoc<'static> {
    let g1 = pretty::RcDoc::text("<")
        .append(tag)
        .append(pretty::RcDoc::space())
        .append("data-id=")
        .append(fastn_runtime::dom::node_key_to_id(node_key));
    let attrs = match attrs {
        Some(v) => v.append(">"),
        None => pretty::RcDoc::text(">"),
    };
    pretty::RcDoc::intersperse(vec![g1, attrs], pretty::RcDoc::space())
        .append("</")
        .append(tag)
        .append(">")
}

pub fn initial(dom: &fastn_runtime::Dom) -> String {
    let mut w = Vec::new();
    let o = dom.html(dom.root);
    o.render(80, &mut w).unwrap();
    String::from_utf8(w).unwrap()
}

impl fastn_runtime::Dom {
    fn html(&self, node_key: fastn_runtime::NodeKey) -> pretty::RcDoc<'static> {
        let root = self.nodes.get(node_key).unwrap();
        root.html(node_key, self)
    }
}

impl fastn_runtime::Element {
    fn html(
        &self,
        node_key: fastn_runtime::NodeKey,
        dom: &fastn_runtime::Dom,
    ) -> pretty::RcDoc<'static> {
        match self {
            fastn_runtime::Element::Container(c) => c.html(node_key, dom),
            fastn_runtime::Element::Text(t) => t.html(node_key, dom),
            fastn_runtime::Element::Image(i) => i.html(node_key, dom),
        }
    }
}

impl fastn_runtime::Container {
    fn html(
        &self,
        node_key: fastn_runtime::NodeKey,
        dom: &fastn_runtime::Dom,
    ) -> pretty::RcDoc<'static> {
        let children = dom.children[node_key]
            .iter()
            .map(|v| dom.html(*v))
            .collect::<Vec<_>>();
        if children.is_empty() {
            fastn_runtime::server::html::leaf("div", node_key, None)
        } else {
            fastn_runtime::server::html::node(
                "div",
                node_key,
                None,
                pretty::RcDoc::intersperse(children, pretty::RcDoc::line()),
            )
        }
    }
}

impl fastn_runtime::Text {
    fn html(
        &self,
        _node_key: fastn_runtime::NodeKey,
        _dom: &fastn_runtime::Dom,
    ) -> pretty::RcDoc<'static> {
        todo!()
    }
}

impl fastn_runtime::Image {
    fn html(
        &self,
        _node_key: fastn_runtime::NodeKey,
        _dom: &fastn_runtime::Dom,
    ) -> pretty::RcDoc<'static> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    fn create_columns() -> Vec<u8> {
        let mut m: Vec<fastn_wasm::Ast> = fastn_runtime::Dom::imports();

        // Note: can not add these till the functions are defined
        m.extend(fastn_wasm::table_2(
            fastn_wasm::RefType::Func,
            "product",
            "foo#on_mouse_enter",
            // "foo#on_mouse_leave",
            // "foo#background",
        ));

        m.push(fastn_wasm::func_def::func1ret(
            "return_externref",
            fastn_wasm::Type::ExternRef.into(),
            fastn_wasm::Type::ExternRef,
        ));

        m.push(fastn_wasm::func_def::func1(
            "no_return",
            fastn_wasm::Type::ExternRef.into(),
        ));

        // (func (export "call_by_index") (param $idx i32) (param $arr externref) (result externref)
        //    call_indirect (type $return_externref) (local.get 0) (local.get 1)
        // )

        // (type $return_externref (func (param externref) (result externref)))
        // (func (export "call_by_index")
        //      (param $idx i32)
        //      (param $arr externref)
        //      (result externref)
        //
        //      (call_indirect (type $return_externref) (local.get $idx) (local.get $arr))
        // )

        m.push(
            fastn_wasm::Func {
                name: None,
                export: Some("call_by_index".to_string()),
                params: vec![
                    fastn_wasm::Type::I32.to_pl("fn_idx"),
                    fastn_wasm::Type::ExternRef.to_pl("arr"),
                ],
                locals: vec![],
                result: Some(fastn_wasm::Type::ExternRef),
                body: vec![fastn_wasm::expression::call_indirect2(
                    "return_externref",
                    fastn_wasm::expression::local("arr"),
                    fastn_wasm::expression::local("fn_idx"),
                )],
            }
            .to_ast(),
        );

        m.push(
            fastn_wasm::Func {
                name: None,
                export: Some("void_by_index".to_string()),
                params: vec![
                    fastn_wasm::Type::I32.to_pl("fn_idx"),
                    fastn_wasm::Type::ExternRef.to_pl("arr"),
                ],
                locals: vec![],
                result: None,
                body: vec![fastn_wasm::expression::call_indirect2(
                    "no_return",
                    fastn_wasm::expression::local("arr"),
                    fastn_wasm::expression::local("fn_idx"),
                )],
            }
            .to_ast(),
        );

        m.push(
            fastn_wasm::Func {
                name: Some("product".to_string()),
                export: None,
                params: vec![fastn_wasm::Type::ExternRef.to_pl("func-data")],
                locals: vec![],
                result: Some(fastn_wasm::Type::ExternRef),
                body: vec![
                    fastn_wasm::expression::call("create_frame"),
                    fastn_wasm::expression::call1(
                        "return_frame",
                        fastn_wasm::expression::call3(
                            "multiply_i32",
                            fastn_wasm::expression::local("func-data"),
                            fastn_wasm::expression::i32(0),
                            fastn_wasm::expression::i32(1),
                        ),
                    ),
                ],
            }
            .to_ast(),
        );

        m.push(
            fastn_wasm::Func {
                name: None,
                export: Some("main".to_string()),
                params: vec![fastn_wasm::Type::ExternRef.to_pl("root")],
                locals: vec![fastn_wasm::Type::ExternRef.to_pl("column")],
                result: None,
                body: vec![
                    fastn_wasm::expression::call("create_frame"),
                    fastn_wasm::expression::call2(
                        "set_global",
                        fastn_wasm::expression::i32(0),
                        fastn_wasm::expression::call1(
                            "create_boolean",
                            fastn_wasm::expression::i32(0),
                        ),
                    ),
                    fastn_wasm::expression::call2(
                        "set_global",
                        fastn_wasm::expression::i32(1),
                        fastn_wasm::expression::call1(
                            "create_i32",
                            fastn_wasm::expression::i32(42),
                        ),
                    ),
                    fastn_wasm::expression::local_set(
                        "column",
                        fastn_wasm::expression::call2(
                            "create_kernel",
                            fastn_wasm::expression::local("root"),
                            fastn_wasm::expression::i32(fastn_runtime::ElementKind::Column.into()),
                        ),
                    ),
                    /* fastn_wasm::expression::call4(
                        "set_dynamic_property_i32",
                        fastn_wasm::expression::local("column"),
                        fastn_wasm::expression::i32(fastn_runtime::UIProperty::WidthFixedPx.into()),
                        fastn_wasm::expression::i32(0), // table_index
                        fastn_wasm::expression::call2(
                            "array_i32_2",
                            fastn_wasm::expression::call1(
                                "create_i32",
                                fastn_wasm::expression::i32(10),
                            ),
                            fastn_wasm::expression::call1("get_global", fastn_wasm::expression::i32(1)),
                        ),
                    ),*/
                    fastn_wasm::expression::call3(
                        "set_property_i32",
                        fastn_wasm::expression::local("column"),
                        fastn_wasm::expression::i32(
                            fastn_runtime::UIProperty::HeightFixedPx.into(),
                        ),
                        fastn_wasm::expression::i32(500),
                    ),
                    fastn_wasm::expression::call3(
                        "set_property_i32",
                        fastn_wasm::expression::local("column"),
                        fastn_wasm::expression::i32(
                            fastn_runtime::UIProperty::SpacingFixedPx.into(),
                        ),
                        fastn_wasm::expression::i32(100),
                    ),
                    fastn_wasm::expression::call3(
                        "set_property_i32",
                        fastn_wasm::expression::local("column"),
                        fastn_wasm::expression::i32(
                            fastn_runtime::UIProperty::MarginFixedPx.into(),
                        ),
                        fastn_wasm::expression::i32(140),
                    ),
                    fastn_wasm::expression::call1("foo", fastn_wasm::expression::local("column")),
                    fastn_wasm::expression::call1("foo", fastn_wasm::expression::local("column")),
                    fastn_wasm::expression::call("end_frame"),
                ],
            }
            .to_ast(),
        );

        m.push(
            fastn_wasm::Func {
                name: Some("foo".to_string()),
                export: None,
                params: vec![fastn_wasm::Type::ExternRef.to_pl("parent")],
                locals: vec![
                    fastn_wasm::Type::ExternRef.to_pl("column"),
                    fastn_wasm::Type::ExternRef.to_pl("on-hover"),
                ],
                result: None,
                body: vec![
                    fastn_wasm::expression::call("create_frame"),
                    fastn_wasm::expression::local_set(
                        "on-hover",
                        fastn_wasm::expression::call1(
                            "create_i32",
                            fastn_wasm::expression::i32(42),
                        ),
                    ),
                    fastn_wasm::expression::local_set(
                        "column",
                        fastn_wasm::expression::call2(
                            "create_kernel",
                            fastn_wasm::expression::local("parent"),
                            fastn_wasm::expression::i32(fastn_runtime::ElementKind::Column.into()),
                        ),
                    ),
                    fastn_wasm::expression::call4(
                        "attach_event_handler",
                        fastn_wasm::expression::local("column"),
                        fastn_wasm::expression::i32(
                            fastn_runtime::DomEventKind::OnGlobalKey.into(),
                        ),
                        fastn_wasm::expression::i32(1), // table index (on-mouse-enter)
                        fastn_wasm::expression::call4(
                            "create_list_2",
                            fastn_wasm::expression::i32(fastn_runtime::PointerKind::Integer.into()),
                            fastn_wasm::expression::call1(
                                "get_global",
                                fastn_wasm::expression::i32(1),
                            ),
                            fastn_wasm::expression::i32(fastn_runtime::PointerKind::Integer.into()),
                            fastn_wasm::expression::local("on-hover"),
                        ),
                    ),
                    fastn_wasm::expression::call3(
                        "set_property_i32",
                        fastn_wasm::expression::local("column"),
                        fastn_wasm::expression::i32(
                            fastn_runtime::UIProperty::HeightFixedPx.into(),
                        ),
                        fastn_wasm::expression::i32(80),
                    ),
                    fastn_wasm::expression::call4(
                        "set_dynamic_property_i32",
                        fastn_wasm::expression::local("column"),
                        fastn_wasm::expression::i32(fastn_runtime::UIProperty::WidthFixedPx.into()),
                        fastn_wasm::expression::i32(0), // table_index
                        fastn_wasm::expression::call2(
                            "array_i32_2",
                            fastn_wasm::expression::call1(
                                "create_i32",
                                fastn_wasm::expression::i32(2),
                            ),
                            fastn_wasm::expression::local("on-hover"),
                        ),
                    ),
                    fastn_wasm::expression::call("end_frame"),
                ],
            }
            .to_ast(),
        );

        m.push(
            fastn_wasm::Func {
                name: Some("foo#on_mouse_enter".to_string()),
                export: None,
                params: vec![fastn_wasm::Type::ExternRef.to_pl("func-data")],
                locals: vec![],
                result: None,
                body: vec![
                    fastn_wasm::expression::call("create_frame"),
                    fastn_wasm::expression::call2(
                        "set_i32",
                        fastn_wasm::expression::call2(
                            "get_func_arg_ref",
                            fastn_wasm::expression::local("func-data"),
                            fastn_wasm::expression::i32(1),
                        ),
                        fastn_wasm::expression::i32(80),
                    ),
                    fastn_wasm::expression::call("end_frame"),
                ],
            }
            .to_ast(),
        );

        m.push(
            fastn_wasm::Func {
                name: Some("foo#on_mouse_leave".to_string()),
                export: None,
                params: vec![fastn_wasm::Type::ExternRef.to_pl("func-data")],
                locals: vec![],
                result: None,
                body: vec![
                    fastn_wasm::expression::call("create_frame"),
                    // fastn_wasm::expression::call2(
                    //     "set_boolean",
                    // ),
                    fastn_wasm::expression::call("end_frame"),
                ],
            }
            .to_ast(),
        );

        let wat = fastn_wasm::encode(&m);
        println!("{}", wat);
        wat.into_bytes()
    }

    #[track_caller]
    fn e(d: fastn_runtime::Document, html: &str) {
        let got = d.initial_html();
        println!("got: {}", got);
        println!("exp: {}", html);
        assert_eq!(got, html)
    }

    #[test]
    fn test() {
        // write test of prime
        e(
            fastn_runtime::Document::new(create_columns()),
            indoc::indoc!(
                r#"
            <div data-id="1v1" > <div data-id=4294967298 > <div data-id=4294967299 ></div>
            <div data-id=4294967300 ></div></div></div>"#
            ),
        )
    }

    #[test]
    fn node_key_ffi_is_stable() {
        let mut i32s: slotmap::SlotMap<fastn_runtime::NodeKey, i32> = slotmap::SlotMap::with_key();
        let k1 = i32s.insert(10);
        let k2 = i32s.insert(20);
        let k3 = i32s.insert(30);
        assert_eq!(fastn_runtime::html::node_key_to_id(k1), "4294967297");
        assert_eq!(fastn_runtime::html::node_key_to_id(k2), "4294967298");
        assert_eq!(fastn_runtime::html::node_key_to_id(k3), "4294967299");
        let mut bools: slotmap::SlotMap<fastn_runtime::NodeKey, bool> =
            slotmap::SlotMap::with_key();
        assert_eq!(
            fastn_runtime::html::node_key_to_id(bools.insert(false)),
            "4294967297"
        );
    }
}
