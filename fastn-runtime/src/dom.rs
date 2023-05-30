pub struct Dom {
    taffy: taffy::Taffy,
    nodes: slotmap::SlotMap<fastn_runtime::NodeKey, fastn_runtime::Element>,
    children: slotmap::SecondaryMap<fastn_runtime::NodeKey, Vec<fastn_runtime::NodeKey>>,
    root: fastn_runtime::NodeKey,
    memory: fastn_runtime::memory::Memory,
}

#[derive(Copy, Clone)]
pub enum ElementKind {
    Column,
    Row,
    Text,
    Image,
    Container,
    IFrame,
    Integer,
    Decimal,
    Boolean,
}

impl From<i32> for ElementKind {
    fn from(i: i32) -> ElementKind {
        match i {
            0 => ElementKind::Column,
            1 => ElementKind::Row,
            2 => ElementKind::Text,
            3 => ElementKind::Image,
            4 => ElementKind::Container,
            5 => ElementKind::IFrame,
            6 => ElementKind::Integer,
            7 => ElementKind::Decimal,
            8 => ElementKind::Boolean,
            _ => panic!("Unknown element kind: {}", i),
        }
    }
}

impl From<ElementKind> for i32 {
    fn from(s: ElementKind) -> i32 {
        match s {
            ElementKind::Column => 0,
            ElementKind::Row => 1,
            ElementKind::Text => 2,
            ElementKind::Image => 3,
            ElementKind::Container => 4,
            ElementKind::IFrame => 5,
            ElementKind::Integer => 6,
            ElementKind::Decimal => 7,
            ElementKind::Boolean => 8,
        }
    }
}

impl Default for Dom {
    fn default() -> Self {
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
            memory: Default::default(),
        }
    }
}

impl Dom {
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
impl Dom {
    pub fn create_kernel(
        &mut self,
        parent: fastn_runtime::NodeKey,
        _k: ElementKind,
    ) -> fastn_runtime::NodeKey {
        let taffy_key = self
            .taffy
            .new_leaf(taffy::style::Style::default())
            .expect("this should never fail");

        // TODO: based on k, create different elements
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
        key: fastn_runtime::NodeKey,
        color_pointer: (i32, i32, i32, f32),
    ) {
        let common_styles = self.nodes[key].common_styles();

        common_styles.background_color = Some(fastn_runtime::ColorValue {
            red: color_pointer.0 as u8,
            green: color_pointer.1 as u8,
            blue: color_pointer.2 as u8,
            alpha: color_pointer.3,
        });
    }

    pub fn set_element_width_px(&mut self, key: fastn_runtime::NodeKey, width: i32) {
        let taffy_key = self.nodes[key].taffy();
        let mut style = self.taffy.style(taffy_key).unwrap().to_owned();
        style.size.width = taffy::prelude::points(width as f32);
        self.taffy.set_style(taffy_key, style).unwrap();
    }

    pub fn set_element_height_px(&mut self, key: fastn_runtime::NodeKey, height: i32) {
        let taffy_key = self.nodes[key].taffy();
        let mut style = self.taffy.style(taffy_key).unwrap().to_owned();
        style.size.height = taffy::prelude::points(height as f32);
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
                self.set_element_background_solid(key, value.rgba())
            }
        }
    }
}

pub enum Value {
    I32(i32),
    F32(f32),
    Vec(Vec<Value>),
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

    fn vec(&self) -> &[Value] {
        if let Value::Vec(i) = self {
            i
        } else {
            panic!("Expected vec value")
        }
    }

    fn rgba(&self) -> (i32, i32, i32, f32) {
        if let Value::Vec(i) = self {
            (i[0].i32(), i[1].i32(), i[2].i32(), i[3].f32())
        } else {
            panic!("Expected vec value")
        }
    }
}
