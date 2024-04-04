pub async fn str(
    str: &str,
    caller: &mut wasmtime::Caller<'_, fastn_ds::wasm::Store>,
) -> wasmtime::Result<i32> {
    send_bytes(str.as_bytes(), caller).await
}

pub async fn send_bytes(
    bytes: &[u8],
    caller: &mut wasmtime::Caller<'_, fastn_ds::wasm::Store>,
) -> wasmtime::Result<i32> {
    let ptr = alloc(bytes.len() as i32, caller).await?;

    let mem = caller.get_export("memory").unwrap().into_memory().unwrap();
    mem.write(caller, ptr as usize + 4, bytes)?;

    Ok(ptr)
}

pub async fn get_str(
    ptr: i32,
    len: i32,
    caller: &mut wasmtime::Caller<'_, fastn_ds::wasm::Store>,
) -> wasmtime::Result<String> {
    get_bytes(ptr, len, caller)
        .await
        .map(|v| unsafe { String::from_utf8_unchecked(v) })
}

pub async fn send_json<T: serde::Serialize>(
    t: T,
    caller: &mut wasmtime::Caller<'_, fastn_ds::wasm::Store>,
) -> wasmtime::Result<i32> {
    let bytes = serde_json::to_vec(&t).unwrap();
    send_bytes(&bytes, caller).await
}

pub async fn get_json<T: serde::de::DeserializeOwned>(
    ptr: i32,
    len: i32,
    caller: &mut wasmtime::Caller<'_, fastn_ds::wasm::Store>,
) -> wasmtime::Result<T> {
    let bytes = get_bytes(ptr, len, caller).await?;
    Ok(serde_json::from_slice(&bytes).unwrap())
}

#[allow(clippy::uninit_vec)]
pub async fn get_bytes(
    ptr: i32,
    len: i32,
    caller: &mut wasmtime::Caller<'_, fastn_ds::wasm::Store>,
) -> wasmtime::Result<Vec<u8>> {
    let mem = caller.get_export("memory").unwrap().into_memory().unwrap();
    let mut buf: Vec<u8> = Vec::with_capacity(len as usize);
    unsafe {
        buf.set_len(len as usize);
    }
    mem.read(caller, ptr as usize, &mut buf)?;
    // dealloc_with_len(ptr, len, caller).await; // TODO: free memory
    Ok(buf)
}

async fn _dealloc(
    ptr: i32,
    caller: &mut wasmtime::Caller<'_, fastn_ds::wasm::Store>,
) -> wasmtime::Result<()> {
    let mut result = vec![wasmtime::Val::I32(0)];
    let dealloc = caller
        .get_export("dealloc")
        .expect("dealloc not exported")
        .into_func()
        .expect("dealloc is not a func");

    let res = dealloc
        .call_async(caller, &[wasmtime::Val::I32(ptr)], &mut result)
        .await;

    if let Err(ref e) = res {
        println!("got error when calling dealloc: {e:?}");
    }

    res
}

async fn _dealloc_with_len(
    ptr: i32,
    len: i32,
    caller: &mut wasmtime::Caller<'_, fastn_ds::wasm::Store>,
) -> wasmtime::Result<()> {
    let mut result = vec![wasmtime::Val::I32(0)];
    let dealloc_with_len = caller
        .get_export("dealloc_with_len")
        .expect("dealloc_with_len not exported")
        .into_func()
        .expect("dealloc_with_len is not a func");

    let res = dealloc_with_len
        .call_async(
            caller,
            &[wasmtime::Val::I32(ptr), wasmtime::Val::I32(len)],
            &mut result,
        )
        .await;

    if let Err(ref e) = res {
        println!("got error when calling func: {e:?}");
    }

    res
}

async fn alloc(
    size: i32,
    caller: &mut wasmtime::Caller<'_, fastn_ds::wasm::Store>,
) -> wasmtime::Result<i32> {
    let mut result = vec![wasmtime::Val::I32(0)];
    let alloc = caller
        .get_export("alloc")
        .expect("alloc not exported")
        .into_func()
        .expect("alloc is not a func");

    let res = alloc
        .call_async(caller, &[wasmtime::Val::I32(size)], &mut result)
        .await;

    if let Err(ref e) = res {
        println!("got error when calling func: {e:?}");
    }

    Ok(result[0].i32().expect("result is not i32"))
}
