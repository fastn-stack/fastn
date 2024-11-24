slotmap::new_key_type! { pub struct NodeKey; }

/// node_key_to_id converts a given slotmap key to a stable id. Each key in each slot map starts
/// with a value like 1v1. This 1v1 is stable contract is the assumption we are working with. This
/// should be stable as the first 1 refers to the index where we are adding the first element, and
/// second 1 refers to the version number, eg if we remove the element at first index, and add
/// another element, it would be 1v2 and so on. This should should be stable. Our entire design
/// of generating pointers on server side and using them on browser side will break if this was not
/// stable.
///
/// See also: node_key_ffi_is_stable() test in this file.
pub fn node_key_to_id(node_key: fastn_runtime::NodeKey) -> String {
    format!("{}", slotmap::Key::data(&node_key).as_ffi())
}

pub trait DomT {
    fn create_kernel(
        &mut self,
        parent: fastn_runtime::NodeKey,
        _k: fastn_runtime::ElementKind,
    ) -> fastn_runtime::NodeKey;
    fn add_child(
        &mut self,
        parent_key: fastn_runtime::NodeKey,
        child_key: fastn_runtime::NodeKey,
    );
}

#[cfg(not(feature = "browser"))]
pub struct Dom {
    pub width: u32,
    pub height: u32,
    pub(crate) last_mouse: fastn_runtime::MouseState,
    pub(crate) has_focus: bool,
    pub(crate) modifiers: fastn_runtime::event::ModifiersState,
    pub(crate) taffy: taffy::Taffy,
    pub(crate) nodes: slotmap::SlotMap<fastn_runtime::NodeKey, fastn_runtime::Element>,
    pub(crate) children: slotmap::SecondaryMap<fastn_runtime::NodeKey, Vec<fastn_runtime::NodeKey>>,
    pub(crate) root: fastn_runtime::NodeKey,
    pub(crate) memory: fastn_runtime::memory::Memory,
}

#[cfg(not(feature = "browser"))]
impl Dom {
    pub fn new(width: u32, height: u32) -> Self {
        let mut nodes = slotmap::SlotMap::with_key();
        let mut taffy = taffy::Taffy::new();
        let mut children = slotmap::SecondaryMap::new();
        let root = nodes.insert(fastn_runtime::Container::outer_column(&mut taffy));
        children.insert(root, vec![]);

        Dom {
            width,
            height,
            taffy,
            nodes,
            root,
            children,
            memory: Default::default(),
            last_mouse: Default::default(),
            has_focus: false,
            modifiers: Default::default(),
        }
    }
    pub fn register_memory_functions(&self, linker: &mut wasmtime::Linker<fastn_runtime::Dom>) {
        self.memory.register(linker)
    }

    pub fn root(&self) -> fastn_runtime::NodeKey {
        self.root
    }

    pub fn memory(&self) -> &fastn_runtime::Memory {
        &self.memory
    }

    pub fn memory_mut(&mut self) -> &mut fastn_runtime::Memory {
        &mut self.memory
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
}

// functions used by wasm
#[cfg(not(feature = "browser"))]
impl Dom {
    pub fn create_kernel(
        &mut self,
        parent: fastn_runtime::NodeKey,
        _k: fastn_runtime::ElementKind,
    ) -> fastn_runtime::NodeKey {
        let taffy_key = self
            .taffy
            .new_leaf(taffy::style::Style::default())
            .expect("this should never fail");

        // TODO: based on k, create different elements
        let c = fastn_runtime::Element::Container(fastn_runtime::Container {
            taffy_key,
            style: fastn_runtime::CommonStyle {
                // background_color: Some(
                //     fastn_runtime::Color {
                //         red: self.memory.create_i32(0),
                //         green: self.memory.create_i32(100),
                //         blue: self.memory.create_i32(0),
                //         alpha: self.memory.create_f32(1.0),
                //     }
                //     .into(),
                // ),
                background_color: None,
                padding: None,
                align: None,
            },
        });

        let key = self.nodes.insert(c);
        self.children.insert(key, vec![]);
        self.add_child(parent, key);
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

    pub fn set_element_background_solid(
        &mut self,
        _key: fastn_runtime::NodeKey,
        _color: fastn_runtime::NodeKey,
    ) {
        // let common_styles = self.nodes[key].common_styles();
        // common_styles.background_color = Some(color);
    }

    pub fn set_element_width_px(&mut self, key: fastn_runtime::NodeKey, width: i32) {
        let taffy_key = self.nodes[key].taffy();
        let mut style = self.taffy.style(taffy_key).unwrap().to_owned();
        dbg!("start", &style.size.width);
        style.size.width = taffy::prelude::points(width as f32);
        dbg!("end", &style.size.width);
        self.taffy.set_style(taffy_key, style).unwrap();
    }

    pub fn set_element_height_px(&mut self, key: fastn_runtime::NodeKey, height: i32) {
        let taffy_key = self.nodes[key].taffy();
        let mut style = self.taffy.style(taffy_key).unwrap().to_owned();
        style.size.height = taffy::prelude::points(height as f32);
        self.taffy.set_style(taffy_key, style).unwrap();
    }

    pub fn set_element_spacing_px(&mut self, key: fastn_runtime::NodeKey, spacing: i32) {
        let taffy_key = self.nodes[key].taffy();
        let mut style = self.taffy.style(taffy_key).unwrap().to_owned();
        style.gap.height = taffy::prelude::points(spacing as f32);
        self.taffy.set_style(taffy_key, style).unwrap();
    }

    pub fn set_element_margin_px(&mut self, key: fastn_runtime::NodeKey, margin: i32) {
        let taffy_key = self.nodes[key].taffy();
        let mut style = self.taffy.style(taffy_key).unwrap().to_owned();
        style.margin = taffy::prelude::points(margin as f32);
        self.taffy.set_style(taffy_key, style).unwrap();
    }

    fn set_element_height_percent(&mut self, key: fastn_runtime::NodeKey, height: f32) {
        let taffy_key = self.nodes[key].taffy();
        let mut style = self.taffy.style(taffy_key).unwrap().to_owned();
        style.size.height = taffy::prelude::points(height);
        self.taffy.set_style(taffy_key, style).unwrap();
    }

    pub fn set_property(
        &mut self,
        key: fastn_runtime::NodeKey,
        property_kind: fastn_runtime::UIProperty,
        value: Value,
    ) {
        match property_kind {
            fastn_runtime::UIProperty::WidthFixedPx => self.set_element_width_px(key, value.i32()),
            fastn_runtime::UIProperty::HeightFixedPx => {
                self.set_element_height_px(key, value.i32())
            }
            fastn_runtime::UIProperty::HeightFixedPercentage => {
                self.set_element_height_percent(key, value.f32())
            }
            fastn_runtime::UIProperty::BackgroundSolid => {
                // self.set_element_background_solid(key, value.rgba())
                todo!()
            }
            fastn_runtime::UIProperty::SpacingFixedPx => {
                self.set_element_spacing_px(key, value.i32())
            }
            fastn_runtime::UIProperty::MarginFixedPx => {
                self.set_element_margin_px(key, value.i32())
            }
            fastn_runtime::UIProperty::Event => {}
        }
    }

    pub fn set_dynamic_property(
        &mut self,
        node_key: fastn_runtime::NodeKey,
        ui_property: fastn_runtime::UIProperty,
        table_index: i32,
        func_arg: fastn_runtime::PointerKey,
        current_value_of_dynamic_property: Value,
    ) {
        self.set_property(node_key, ui_property, current_value_of_dynamic_property);

        let func_arg = func_arg.into_list_pointer();

        let mem = self.memory_mut();
        let closure_key = mem.create_closure(fastn_runtime::Closure {
            function: table_index,
            captured_variables: func_arg,
        });

        mem.add_dynamic_property_dependency(
            func_arg,
            ui_property.into_dynamic_property(node_key, closure_key),
        );
    }
}

pub enum Value {
    I32(i32),
    F32(f32),
    Vec(Vec<Value>),
    Color(i32, i32, i32, f32),
}

impl From<i32> for Value {
    fn from(i: i32) -> Value {
        Value::I32(i)
    }
}

impl From<f32> for Value {
    fn from(i: f32) -> Value {
        Value::F32(i)
    }
}

impl From<(i32, i32, i32, f32)> for Value {
    fn from(i: (i32, i32, i32, f32)) -> Value {
        Value::Color(i.0, i.1, i.2, i.3)
    }
}

impl From<Vec<Value>> for Value {
    fn from(i: Vec<Value>) -> Value {
        Value::Vec(i)
    }
}

impl Value {
    fn i32(&self) -> i32 {
        if let Value::I32(i) = self {
            *i
        } else {
            panic!("Expected i32 value")
        }
    }

    fn f32(&self) -> f32 {
        if let Value::F32(i) = self {
            *i
        } else {
            panic!("Expected f32 value")
        }
    }

    fn rgba(&self) -> (i32, i32, i32, f32) {
        if let Value::Color(r, g, b, a) = self {
            (*r, *g, *b, *a)
        } else {
            panic!("Expected vec value")
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn ui_dependency() {
        let mut d = super::Dom::default();
        println!("1** {:#?}", d.memory());
        d.memory().assert_empty();
        d.memory_mut().create_frame();

        let i32_pointer = d.memory_mut().create_i32(200);
        let i32_pointer2 = d.memory_mut().create_i32(100);
        let arr_ptr = d
            .memory_mut()
            .create_list_1(fastn_runtime::PointerKind::Integer, i32_pointer);
        let column_node = d.create_kernel(d.root, fastn_runtime::ElementKind::Column);

        let closure_key = d.memory_mut().create_closure(fastn_runtime::Closure {
            function: 0,
            captured_variables: arr_ptr.into_list_pointer(),
        });
        d.memory_mut().add_dynamic_property_dependency(
            i32_pointer.into_integer_pointer(),
            fastn_runtime::UIProperty::WidthFixedPx.into_dynamic_property(column_node, closure_key),
        );
        d.memory_mut().end_frame();

        // i32_pointer should still be live as its attached as a dynamic property
        assert!(d
            .memory
            .is_pointer_valid(i32_pointer.into_integer_pointer()));
        // i32_pointer2 should go away as its not needed anywhere
        assert!(!d
            .memory
            .is_pointer_valid(i32_pointer2.into_integer_pointer()));
    }
}
