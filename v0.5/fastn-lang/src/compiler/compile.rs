pub async fn compile(ds: &mut Box<dyn fastn_lang::DS>, path: &str, _strict: bool) {
    let _source = ds.source(path).await.unwrap();

    todo!()
}
