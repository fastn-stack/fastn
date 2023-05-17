pub struct Document {
    pub taffy: taffy::Taffy,
    pub nodes: slotmap::SlotMap<fastn_runtime::NodeKey, fastn_runtime::Element>,
    pub root: fastn_runtime::NodeKey,
    pub width: u32,
    pub height: u32,
    // variables, bindings
    pub wasm: fastn_runtime::wasm::Wasm,
}


/*

// global doc

fn add_wrapper(a: Ref, b: Ref) -> Ref {
    let a = doc.get(a);
    let b = doc.get(b);
    let c = add(a, b);
    Ref::new(c)
}

fn add(a: u32, b: u32) -> u32 {
    a + b
}

fn handle_event_wrapper(a: Ref, b: Ref) {
    let a = doc.get(a);
    handle_event(a, b);
}

fn handle_event(a: u32, b: Mut<u32>) {
    b.set(a + 1)
}

-- boolean f: true
-- boolean g: hello(v=$f)

-- boolean hello(v):
boolean v:

!v

-- ftd.boolean: $g



let rt = Runtime::new();

let f = rt.create_boolean_id_ref(true);
let hello = rt.create_function(boolean, [Ref::new(boolean)], [], "return !$0");
let g = rt.create_boolean_ref(hello, [f]); // internally update dependency graph

rt.create_boolean_kernel(g); // internally update dependency graph

rt.show();

 */

impl Document {
    pub fn new(wat: impl AsRef<[u8]>) -> Document {
        let mut nodes = slotmap::SlotMap::with_key();
        let mut taffy = taffy::Taffy::new();
        let root = nodes.insert(fastn_runtime::Container::outer_column(&mut taffy));

        Document {
            root,
            taffy,
            nodes,
            width: 0,
            height: 0,
            wasm: fastn_runtime::wasm::Wasm::new(wat),
        }
    }

    // initial_html() -> server side HTML
    // hydrate() -> client side
    // event_with_target() -> Vec<DomMutation>

    // pub fn create_string_ref(&mut self) -> fastn_runtime::Ref {
    //     todo!()
    // }
    //
    // pub fn add_text(&mut self, text: fastn_runtime::Callable<String>) -> fastn_runtime::NodeKey {
    //     let text_style = fastn_runtime::TextStyle::default();
    //     let taffy = self.taffy.new_leaf(text_style.taffy()).unwrap();
    //     let t = fastn_runtime::Text {
    //         taffy,
    //         text,
    //         style: text_style,
    //     };
    //     self.nodes.insert(fastn_runtime::Element::Text(t))
    // }

    // if not wasm
    pub fn initial_layout(
        &mut self,
        width: u32,
        height: u32,
    ) -> (fastn_runtime::ControlFlow, Vec<fastn_runtime::Operation>) {
        let taffy_root = self.nodes[self.root].taffy();
        self.taffy
            .compute_layout(
                taffy_root,
                taffy::prelude::Size {
                    width: taffy::prelude::points(width as f32),
                    height: taffy::prelude::points(height as f32),
                },
            )
            .unwrap();
        self.width = width;
        self.height = height;
        dbg!(self.taffy.layout(taffy_root).unwrap());

        (fastn_runtime::ControlFlow::WaitForEvent, self.wasm.run())
    }

    // if not wasm
    pub async fn event(
        &mut self,
        _e: fastn_runtime::Event,
    ) -> (fastn_runtime::ControlFlow, Vec<fastn_runtime::Operation>) {
        // find the event target based on current layout and event coordinates
        // handle event, which will update the dom tree
        // compute layout
        (fastn_runtime::ControlFlow::WaitForEvent, vec![])
    }
}



