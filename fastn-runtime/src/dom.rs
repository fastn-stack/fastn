pub struct Dom {
    pub taffy: taffy::Taffy,
    pub nodes: slotmap::SlotMap<fastn_runtime::NodeKey, fastn_runtime::Element>,
    pub children: slotmap::SecondaryMap<fastn_runtime::NodeKey, Vec<fastn_runtime::NodeKey>>,
    pub root: fastn_runtime::NodeKey,
    pub store: fastn_runtime::runtime_store::Memory,
}

impl Dom {
    pub fn new() -> Dom {
        let mut nodes = slotmap::SlotMap::with_key();
        let mut taffy = taffy::Taffy::new();
        let mut children = slotmap::SecondaryMap::new();
        let root = nodes.insert(fastn_runtime::Container::outer_column(&mut taffy));
        children.insert(root, vec![]);
        println!("root: {:?}", &root);

        Dom {
            taffy,
            nodes,
            root,
            children,
            store: Default::default(),
        }
    }

    pub fn compute_layout(&mut self, width: u32, height: u32) -> Vec<fastn_runtime::Operation> {
        let taffy_root = self.nodes[self.root].taffy();
        self.taffy
            .compute_layout(
                taffy_root,
                taffy::prelude::Size {
                    width: taffy::prelude::points(dbg!(width) as f32),
                    height: taffy::prelude::points(dbg!(height) as f32),
                },
            )
            .unwrap();

        dbg!(self.layout_to_operations(self.root))
    }

    fn layout_to_operations(&self, key: fastn_runtime::NodeKey) -> Vec<fastn_runtime::Operation> {
        let node = self.nodes.get(key).unwrap();
        match node {
            fastn_runtime::Element::Container(c) => {
                let mut operations = vec![];

                // no need to draw a rectangle if there is no color or border
                if let Some(o) = c.operation(&self.taffy) {
                    operations.push(o);
                }

                for child in self.children.get(key).unwrap() {
                    operations.extend(self.layout_to_operations(*child));
                }
                operations
            }
            fastn_runtime::Element::Text(_t) => todo!(),
            fastn_runtime::Element::Image(_i) => todo!(),
        }
    }

    pub fn create_column(&mut self) -> fastn_runtime::NodeKey {
        let taffy_key = self
            .taffy
            .new_leaf(taffy::style::Style::default())
            .expect("this should never fail");

        let c = fastn_runtime::Element::Container(fastn_runtime::Container {
            taffy_key,
            style: fastn_runtime::CommonStyleMinusTaffy {
                background_color: Some(fastn_runtime::ColorValue {
                    red: 0,
                    green: 100,
                    blue: 0,
                    alpha: 1.0,
                }),
            },
        });

        let key = self.nodes.insert(c);
        self.children.insert(key, vec![]);
        println!("column: {:?}", &key);
        key
    }

    pub fn add_child(
        &mut self,
        parent_key: fastn_runtime::NodeKey,
        child_key: fastn_runtime::NodeKey,
    ) {
        let parent = self.nodes.get(parent_key).unwrap();
        let child = self.nodes.get(child_key).unwrap();
        self.taffy.add_child(parent.taffy(), child.taffy()).unwrap();
        self.children
            .entry(parent_key)
            .unwrap()
            .or_default()
            .push(child_key);
        println!("add_child: {:?} -> {:?}", &parent_key, &child_key);
    }

    fn set_column_width_px(&mut self, key: fastn_runtime::NodeKey, width: i32) {
        let taffy_key = self.nodes[key].taffy();
        let mut style = self.taffy.style(taffy_key).unwrap().to_owned();
        style.size.width = taffy::prelude::points(width as f32);
        self.taffy.set_style(taffy_key, style).unwrap();
    }

    fn set_column_height_px(&mut self, key: fastn_runtime::NodeKey, width: i32) {
        let taffy_key = self.nodes[key].taffy();
        let mut style = self.taffy.style(taffy_key).unwrap().to_owned();
        style.size.height = taffy::prelude::points(width as f32);
        self.taffy.set_style(taffy_key, style).unwrap();
    }

    pub fn create_instance(
        wat: impl AsRef<[u8]>,
    ) -> (wasmtime::Store<fastn_runtime::Dom>, wasmtime::Instance) {
        let engine = wasmtime::Engine::new(wasmtime::Config::new().async_support(false))
            .expect("cant create engine");
        let module = wasmtime::Module::new(&engine, wat).expect("cant parse module");
        let dom = fastn_runtime::Dom::new();

        let mut linker = wasmtime::Linker::new(&engine);

        dom.store.register(&mut linker);

        // this is quite tedious boilerplate, maybe we can write some macro to generate it
        linker
            .func_new(
                "fastn",
                "create_column",
                wasmtime::FuncType::new(
                    [].iter().cloned(),
                    [wasmtime::ValType::ExternRef].iter().cloned(),
                ),
                |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, _params, results| {
                    // ExternRef is a reference-counted pointer to a host-defined object. We mut not
                    // deallocate it on Rust side unless it's .strong_count() is 0. Not sure how it
                    // affects us yet.
                    results[0] = wasmtime::Val::ExternRef(Some(wasmtime::ExternRef::new(
                        caller.data_mut().create_column(),
                    )));
                    Ok(())
                },
            )
            .unwrap();
        linker
            .func_new(
                "fastn",
                "root_container",
                wasmtime::FuncType::new(
                    [].iter().cloned(),
                    [wasmtime::ValType::ExternRef].iter().cloned(),
                ),
                |caller: wasmtime::Caller<'_, fastn_runtime::Dom>, _params, results| {
                    // ExternRef is a reference-counted pointer to a host-defined object. We mut not
                    // deallocate it on Rust side unless it's .strong_count() is 0. Not sure how it
                    // affects us yet.
                    results[0] = wasmtime::Val::ExternRef(Some(wasmtime::ExternRef::new(
                        caller.data().root,
                    )));
                    Ok(())
                },
            )
            .unwrap();
        linker
            .func_new(
                "fastn",
                "add_child",
                wasmtime::FuncType::new(
                    [wasmtime::ValType::ExternRef, wasmtime::ValType::ExternRef]
                        .iter()
                        .cloned(),
                    [].iter().cloned(),
                ),
                |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, params, _results| {
                    // ExternRef is a reference-counted pointer to a host-defined object. We mut not
                    // deallocate it on Rust side unless it's .strong_count() is 0. Not sure how it
                    // affects us yet.
                    caller.data_mut().add_child(params.key(0), params.key(1));
                    Ok(())
                },
            )
            .unwrap();
        linker
            .func_new(
                "fastn",
                "set_column_width_px",
                wasmtime::FuncType::new(
                    [wasmtime::ValType::ExternRef, wasmtime::ValType::I32]
                        .iter()
                        .cloned(),
                    [].iter().cloned(),
                ),
                |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, p, _results| {
                    // ExternRef is a reference-counted pointer to a host-defined object. We mut not
                    // deallocate it on Rust side unless it's .strong_count() is 0. Not sure how it
                    // affects us yet.
                    caller.data_mut().set_column_width_px(p.key(0), p.i32(1));
                    Ok(())
                },
            )
            .unwrap();
        linker
            .func_new(
                "fastn",
                "set_column_height_px",
                wasmtime::FuncType::new(
                    [wasmtime::ValType::ExternRef, wasmtime::ValType::I32]
                        .iter()
                        .cloned(),
                    [].iter().cloned(),
                ),
                |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, p, _results| {
                    // ExternRef is a reference-counted pointer to a host-defined object. We mut not
                    // deallocate it on Rust side unless it's .strong_count() is 0. Not sure how it
                    // affects us yet.
                    caller.data_mut().set_column_height_px(p.key(0), p.i32(1));
                    Ok(())
                },
            )
            .unwrap();
        let mut store = wasmtime::Store::new(&engine, fastn_runtime::Dom::new());
        let instance = linker
            .instantiate(&mut store, &module)
            .expect("cant create instance");

        let root = Some(wasmtime::ExternRef::new(store.data().root));

        let wasm_main = instance
            .get_typed_func::<(Option<wasmtime::ExternRef>,), ()>(&mut store, "main")
            .unwrap();
        wasm_main.call(&mut store, (root, )).unwrap();

        (store, instance)
    }
}

pub trait Params {
    fn i32(&self, idx: usize) -> i32;
    fn key(&self, idx: usize) -> fastn_runtime::NodeKey;
    fn ptr(&self, idx: usize) -> fastn_runtime::PointerKey;
    fn boolean(&self, idx: usize) -> bool;
}

impl Params for [wasmtime::Val] {
    fn i32(&self, idx: usize) -> i32 {
        self[idx].i32().unwrap()
    }

    fn key(&self, idx: usize) -> fastn_runtime::NodeKey {
        *self[idx]
            .externref()
            .unwrap()
            .expect("externref gone?")
            .data()
            .downcast_ref()
            .unwrap()
    }
    fn ptr(&self, idx: usize) -> fastn_runtime::PointerKey {
        *self[idx]
            .externref()
            .unwrap()
            .expect("externref gone?")
            .data()
            .downcast_ref()
            .unwrap()
    }

    fn boolean(&self, idx: usize) -> bool {
        self.i32(idx) != 0
    }
}

#[cfg(test)]
mod test {
    fn assert_import(name: &str, type_: &str) {
        fastn_runtime::Dom::create_instance(format!(
            r#"
                (module (import "fastn" "{}" (func {}))
                    (func (export "main"))
                )
            "#,
            name, type_
        ));
    }

    #[test]
    fn test() {
        assert_import("create_column", "(result externref)");
        assert_import("root_container", "(result externref)");
        assert_import("add_child", "(param externref externref)");
        assert_import("set_column_width_px", "(param externref i32)");
        assert_import("set_column_height_px", "(param externref i32)");
    }
}
