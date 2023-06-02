impl fastn_wasm::StoreExtractor for fastn_runtime::Memory {
    type Parent = fastn_runtime::Dom;
    fn extract<'a>(store: &'a mut wasmtime::Caller<fastn_runtime::Dom>) -> &'a mut Self {
        store.data_mut().memory_mut()
    }
}

impl fastn_wasm::StoreExtractor for fastn_runtime::Dom {
    type Parent = fastn_runtime::Dom;
    fn extract<'a>(store: &'a mut wasmtime::Caller<fastn_runtime::Dom>) -> &'a mut Self {
        store.data_mut()
    }
}

impl fastn_wasm::FromToI32 for fastn_runtime::DomEventKind {}
impl fastn_wasm::FromToI32 for fastn_runtime::ElementKind {}
impl fastn_wasm::FromToI32 for fastn_runtime::PointerKind {}
impl fastn_wasm::FromToI32 for fastn_runtime::UIProperty {}

impl fastn_wasm::WasmType for fastn_runtime::NodeKey {
    fn extract(idx: usize, vals: &[wasmtime::Val]) -> Self {
        vals.key(idx)
    }
    fn the_type() -> wasmtime::ValType {
        wasmtime::ValType::ExternRef
    }
    fn to_wasm(&self) -> wasmtime::Val {
        wasmtime::Val::ExternRef(Some(wasmtime::ExternRef::new(*self)))
    }
}

impl fastn_wasm::WasmType for fastn_runtime::PointerKey {
    fn extract(idx: usize, vals: &[wasmtime::Val]) -> Self {
        vals.ptr(idx)
    }
    fn the_type() -> wasmtime::ValType {
        wasmtime::ValType::ExternRef
    }
    fn to_wasm(&self) -> wasmtime::Val {
        wasmtime::Val::ExternRef(Some(wasmtime::ExternRef::new(*self)))
    }
}

pub trait Params {
    fn i32(&self, idx: usize) -> i32;
    fn f32(&self, idx: usize) -> f32;
    fn externref(&self, idx: usize) -> Option<wasmtime::ExternRef>;
    fn key(&self, idx: usize) -> fastn_runtime::NodeKey;
    fn ptr(&self, idx: usize) -> fastn_runtime::PointerKey;
    fn boolean(&self, idx: usize) -> bool;
}

impl Params for [wasmtime::Val] {
    fn i32(&self, idx: usize) -> i32 {
        self[idx].i32().unwrap()
    }

    fn f32(&self, idx: usize) -> f32 {
        self[idx].f32().unwrap()
    }

    fn externref(&self, idx: usize) -> Option<wasmtime::ExternRef> {
        self[idx].externref().unwrap()
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

pub trait CallerExt {
    fn memory(&self) -> &fastn_runtime::Memory;
    fn memory_mut(&mut self) -> &mut fastn_runtime::Memory;
}

impl CallerExt for wasmtime::Caller<'_, fastn_runtime::Dom> {
    fn memory(&self) -> &fastn_runtime::Memory {
        self.data().memory()
    }
    fn memory_mut(&mut self) -> &mut fastn_runtime::Memory {
        self.data_mut().memory_mut()
    }
}
