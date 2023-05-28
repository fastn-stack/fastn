pub trait ParamExtractor<T> {
    fn extract(&self, idx: usize) -> T;
}

impl ParamExtractor<i32> for [wasmtime::Val] {
    fn extract(&self, idx: usize) -> i32 {
        self.i32(idx)
    }
}

pub trait Params {
    fn i32(&self, idx: usize) -> i32;
    fn f32(&self, idx: usize) -> f32;
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

pub trait LinkerExt {
    fn func0(
        &mut self,
        name: &str,
        func: impl Fn(&mut fastn_runtime::Memory) + Send + Sync + 'static,
    );
    // fn func1<T: ParamExtractor<T>>(&mut self, name: &str, func: impl Fn(&mut fastn_runtime::Memory, T) + Send + Sync + 'static);
    fn func1(
        &mut self,
        name: &str,
        arg1: wasmtime::ValType,
        func: impl Fn(&mut fastn_runtime::Memory, &wasmtime::Val) + Send + Sync + 'static,
    );
    fn func2(
        &mut self,
        name: &str,
        arg1: wasmtime::ValType,
        arg2: wasmtime::ValType,
        func: impl Fn(&mut fastn_runtime::Memory, &wasmtime::Val, &wasmtime::Val)
        + Send
        + Sync
        + 'static,
    );
    fn func0ret<O: Into<wasmtime::Val>>(
        &mut self,
        name: &str,
        func: impl Fn(&mut fastn_runtime::Memory) -> O + Send + Sync + 'static,
    );
    fn func1ret<O: Into<wasmtime::Val>>(
        &mut self,
        name: &str,
        arg: wasmtime::ValType,
        func: impl Fn(&mut fastn_runtime::Memory, &wasmtime::Val) -> O
        + Send
        + Sync
        + 'static,
    );
    fn func2ret<O: Into<wasmtime::Val>>(
        &mut self,
        name: &str,
        arg1: wasmtime::ValType,
        arg2: wasmtime::ValType,
        func: impl Fn(&mut fastn_runtime::Memory, &wasmtime::Val, &wasmtime::Val) -> O
        + Send
        + Sync
        + 'static,
    );
}

impl LinkerExt for wasmtime::Linker<fastn_runtime::Dom> {
    fn func0(
        &mut self,
        name: &str,
        func: impl Fn(&mut fastn_runtime::Memory) + Send + Sync + 'static,
    ) {
        self.func_new(
            "fastn",
            name,
            wasmtime::FuncType::new([].iter().cloned(), [].iter().cloned()),
            move |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, _params, _results| {
                func(caller.memory_mut());
                Ok(())
            },
        )
            .unwrap();
    }
    fn func1(
        &mut self,
        name: &str,
        arg1: wasmtime::ValType,
        func: impl Fn(&mut fastn_runtime::Memory, &wasmtime::Val) + Send + Sync + 'static,
    ) {
        self.func_new(
            "fastn",
            name,
            wasmtime::FuncType::new([arg1].iter().cloned(), [].iter().cloned()),
            move |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, params, _results| {
                func(caller.memory_mut(), &params[0]);
                Ok(())
            },
        )
            .unwrap();
    }
    fn func2(
        &mut self,
        name: &str,
        arg1: wasmtime::ValType,
        arg2: wasmtime::ValType,
        func: impl Fn(&mut fastn_runtime::Memory, &wasmtime::Val, &wasmtime::Val)
        + Send
        + Sync
        + 'static,
    ) {
        self.func_new(
            "fastn",
            name,
            wasmtime::FuncType::new([arg1, arg2].iter().cloned(), [].iter().cloned()),
            move |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, params, _results| {
                func(caller.memory_mut(), &params[0], &params[1]);
                Ok(())
            },
        )
            .unwrap();
    }
    fn func0ret<O: Into<wasmtime::Val>>(
        &mut self,
        name: &str,
        func: impl Fn(&mut fastn_runtime::Memory) -> O + Send + Sync + 'static,
    ) {
        self.func_new(
            "fastn",
            name,
            wasmtime::FuncType::new([].iter().cloned(), [].iter().cloned()),
            move |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, _params, results| {
                results[0] = func(caller.memory_mut()).into();
                Ok(())
            },
        )
            .unwrap();
    }
    fn func1ret<O: Into<wasmtime::Val>>(
        &mut self,
        name: &str,
        arg: wasmtime::ValType,
        func: impl Fn(&mut fastn_runtime::Memory, &wasmtime::Val) -> O
        + Send
        + Sync
        + 'static,
    ) {
        self.func_new(
            "fastn",
            name,
            wasmtime::FuncType::new([arg].iter().cloned(), [].iter().cloned()),
            move |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, params, results| {
                results[0] = func(caller.memory_mut(), &params[0]).into();
                Ok(())
            },
        )
            .unwrap();
    }
    fn func2ret<O: Into<wasmtime::Val>>(
        &mut self,
        name: &str,
        arg1: wasmtime::ValType,
        arg2: wasmtime::ValType,
        func: impl Fn(&mut fastn_runtime::Memory, &wasmtime::Val, &wasmtime::Val) -> O
        + Send
        + Sync
        + 'static,
    ) {
        self.func_new(
            "fastn",
            name,
            wasmtime::FuncType::new([arg1, arg2].iter().cloned(), [].iter().cloned()),
            move |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, params, results| {
                results[0] = func(caller.memory_mut(), &params[0], &params[1]).into();
                Ok(())
            },
        )
            .unwrap();
    }
}

impl From<fastn_runtime::PointerKey> for wasmtime::Val {
    fn from(value: fastn_runtime::PointerKey) -> Self {
        wasmtime::Val::ExternRef(Some(wasmtime::ExternRef::new(value)))
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