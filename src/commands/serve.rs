async fn serve_static(req: actix_web::HttpRequest) -> actix_web::Result<actix_files::NamedFile> {
    // TODO: It should ideally fallback to index file if not found than an error file or directory listing
    let path: std::path::PathBuf = req.match_info().query("path").parse().unwrap();
    let file_path = std::path::PathBuf::new().join(".build").join(path);
    let file_path = if file_path.is_file() {
        file_path
    } else {
        std::path::PathBuf::new().join(file_path).join("index.html")
    };
    let file = actix_files::NamedFile::open_async(file_path).await?;
    Ok(file)
}

#[actix_web::main]
pub async fn serve() -> std::io::Result<()> {
    println!("### Server Started ###");
    println!("Go to: {}", "http://127.0.0.1:8000");
    actix_web::HttpServer::new(|| {
        actix_web::App::new().route("/{path:.*}", actix_web::web::get().to(serve_static))
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}
