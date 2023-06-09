// methods exposed to wasm, methods we are relatively confident are correct and needed.
impl fastn_runtime::Memory {
    pub fn create_frame(&mut self) {
        self.stack.push(fastn_runtime::Frame::default());
    }

    pub fn end_frame(&mut self) {
        // using .unwrap() so we crash on a bug instead of silently ignoring it
        for pointer in self.stack.pop().unwrap().pointers.iter() {
            self.drop_pointer(*pointer, &mut vec![], None);
        }
    }

    pub fn return_frame(&mut self, keep: fastn_runtime::PointerKey) -> fastn_runtime::PointerKey {
        let mut k: Option<fastn_runtime::Pointer> = None;
        let mut v = vec![];

        for pointer in self.stack.pop().unwrap().pointers.iter() {
            if pointer.pointer == keep {
                k = Some(pointer.to_owned());
            } else {
                self.drop_pointer(*pointer, &mut v, None);
            }
        }

        let k = k.unwrap();
        self.insert_in_frame(k.pointer, k.kind);
        keep
    }

    pub fn get_global(&self, idx: i32) -> fastn_runtime::PointerKey {
        self.global[idx as usize]
    }

    pub fn set_global(&mut self, idx: i32, ptr: fastn_runtime::PointerKey) {
        let idx = idx as usize;

        if idx < self.global.len() {
            println!("updated global: idx={}, ptr={:?}", idx, ptr);
            self.global[idx] = ptr;
            return;
        }

        if idx == self.global.len() {
            println!("created global: idx={}, ptr={:?}", idx, ptr);
            self.global.push(ptr);
            return;
        }

        // the way things are either this global variables are sequentially initialised at the start
        // of the program. If a jump happens it means our generated wasm file is incorrect.
        unreachable!()
    }
}
