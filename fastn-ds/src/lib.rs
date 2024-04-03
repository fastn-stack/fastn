extern crate self as fastn_ds;

use deadpool_postgres::tokio_postgres;

pub mod http;
pub mod mail;
pub mod reqwest_util;
mod utils;

#[derive(Debug, Clone)]
pub struct DocumentStore {
    pub wasm_modules: dashmap::DashMap<String, wasmtime::Module>,
    pub pg_pools: actix_web::web::Data<dashmap::DashMap<String, deadpool_postgres::Pool>>,
    root: Path,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    path: camino::Utf8PathBuf,
}

impl fastn_ds::Path {
    pub fn new<T: AsRef<str>>(path: T) -> Self {
        Self {
            path: camino::Utf8PathBuf::from(path.as_ref()),
        }
    }

    pub fn join<T: AsRef<str>>(&self, path: T) -> Self {
        Self {
            path: self.path.join(path.as_ref()),
        }
    }

    pub fn parent(&self) -> Option<Self> {
        self.path.parent().map(|path| Path {
            path: path.to_path_buf(),
        })
    }

    pub fn strip_prefix(&self, base: &Self) -> Option<Self> {
        self.path
            .strip_prefix(base.path.as_str())
            .ok()
            .map(|v| Path {
                path: v.to_path_buf(),
            })
    }

    pub fn file_name(&self) -> Option<String> {
        self.path.file_name().map(|v| v.to_string())
    }

    pub fn extension(&self) -> Option<String> {
        self.path.extension().map(|v| v.to_string())
    }

    pub fn with_extension(&self, extension: impl AsRef<str>) -> Self {
        Self {
            path: self.path.with_extension(extension),
        }
    }
}

impl std::fmt::Display for fastn_ds::Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.as_str())
    }
}

fn package_ignores(
    ignore_paths: &[String],
    root_path: &camino::Utf8PathBuf,
) -> Result<ignore::overrides::Override, ignore::Error> {
    let mut overrides = ignore::overrides::OverrideBuilder::new(root_path);
    for ignored_path in ignore_paths {
        overrides.add(format!("!{}", ignored_path).as_str())?;
    }
    overrides.build()
}

#[derive(thiserror::Error, Debug)]
pub enum RemoveError {
    #[error("io error {0}")]
    IOError(#[from] std::io::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum RenameError {
    #[error("io error {0}")]
    IOError(#[from] std::io::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum ReadError {
    #[error("io error {0}")]
    IOError(std::io::Error),
    #[error("not found")]
    NotFound,
}

impl From<std::io::Error> for ReadError {
    fn from(err: std::io::Error) -> Self {
        if err.kind() == std::io::ErrorKind::NotFound {
            ReadError::NotFound
        } else {
            ReadError::IOError(err)
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ReadStringError {
    #[error("read error {0}")]
    ReadError(#[from] ReadError),
    #[error("utf-8 error {0}")]
    UTF8Error(#[from] std::string::FromUtf8Error),
}

#[derive(thiserror::Error, Debug)]
pub enum WriteError {
    #[error("pool error {0}")]
    IOError(#[from] std::io::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum WasmReadError {
    #[error("read error {0}")]
    ReadError(#[from] ReadError),
    #[error("wasm error {0}")]
    WasmError(#[from] wasmtime::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum HttpError {
    #[error("http error {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("url parse error {0}")]
    URLParseError(#[from] url::ParseError),
    #[error("generic error {message}")]
    GenericError { message: String },
    #[error("wasm read error {0}")]
    WasmReadError(#[from] WasmReadError),
}

pub type HttpResponse = ::http::Response<bytes::Bytes>;

#[async_trait::async_trait]
pub trait RequestType {
    async fn ud(&self, ds: &fastn_ds::DocumentStore) -> Option<ft_sys_shared::UserData>;
    fn headers(&self) -> &reqwest::header::HeaderMap;
    fn method(&self) -> &str;
    fn query_string(&self) -> &str;
    fn get_ip(&self) -> Option<String>;
    fn cookies_string(&self) -> Option<String>;
    fn body(&self) -> &[u8];
}

pub static WASM_ENGINE: once_cell::sync::Lazy<wasmtime::Engine> =
    once_cell::sync::Lazy::new(|| {
        wasmtime::Engine::new(wasmtime::Config::new().async_support(true)).unwrap()
    });

// async fn create_pool(
//     req_config: &fastn_core::RequestConfig,
// ) -> Result<deadpool_postgres::Pool, deadpool_postgres::CreatePoolError> {
//     let mut cfg = deadpool_postgres::Config::new();
//     cfg.libpq_style_connection_string = match req_config.config.ds.env("FASTN_DB_URL").await {
//         Ok(v) => Some(v),
//         Err(_) => {
//             fastn_core::warning!("FASTN_DB_URL is not set");
//             return Err(deadpool_postgres::CreatePoolError::Config(
//                 deadpool_postgres::ConfigError::ConnectionStringInvalid,
//             ));
//         }
//     };
//     cfg.manager = Some(deadpool_postgres::ManagerConfig {
//         // TODO: make this configurable
//         recycling_method: deadpool_postgres::RecyclingMethod::Verified,
//     });
//     let runtime = Some(deadpool_postgres::Runtime::Tokio1);
//
//     if let Ok(true) = req_config
//         .config
//         .ds
//         .env_bool("FASTN_PG_DANGER_ENABLE_SSL", false)
//         .await
//     {
//         fastn_core::warning!(
//             "FASTN_PG_DANGER_DISABLE_SSL is set to false, this is not recommended for production use",
//         );
//         cfg.ssl_mode = Some(deadpool_postgres::SslMode::Disable);
//         return cfg.create_pool(runtime, tokio_postgres::NoTls);
//     }
//
//     let mut connector = native_tls::TlsConnector::builder();
//
//     match req_config
//         .config
//         .ds
//         .env("FASTN_PG_SSL_MODE")
//         .await
//         .as_deref()
//     {
//         Err(_) | Ok("require") => {
//             cfg.ssl_mode = Some(deadpool_postgres::SslMode::Require);
//         }
//         Ok("prefer") => {
//             fastn_core::warning!(
//                 "FASTN_PG_SSL_MODE is set to prefer, which roughly means \"I don't care about \
//                 encryption, but I wish to pay the overhead of encryption if the server supports it.\"\
//                 and is not recommended for production use",
//             );
//             cfg.ssl_mode = Some(deadpool_postgres::SslMode::Prefer);
//         }
//         Ok(v) => {
//             // TODO: openssl also allows `verify-ca` and `verify-full` but native_tls does not
//             fastn_core::warning!(
//                 "FASTN_PG_SSL_MODE is set to {}, which is invalid, only allowed values are prefer and require",
//                 v,
//             );
//             return Err(deadpool_postgres::CreatePoolError::Config(
//                 deadpool_postgres::ConfigError::ConnectionStringInvalid,
//             ));
//         }
//     }
//
//     if let Ok(true) = req_config
//         .config
//         .ds
//         .env_bool("FASTN_PG_DANGER_ALLOW_UNVERIFIED_CERTIFICATE", false)
//         .await
//     {
//         fastn_core::warning!(
//             "FASTN_PG_DANGER_ALLOW_UNVERIFIED_CERTIFICATE is set to true, this is not \
//             recommended for production use",
//         );
//         connector.danger_accept_invalid_certs(true);
//     }
//
//     if let Ok(cert) = req_config.config.ds.env("FASTN_PG_CERTIFICATE").await {
//         // TODO: This does not work with Heroku certificate.
//         let cert = req_config
//             .config
//             .ds
//             .read_content(&fastn_ds::Path::new(cert))
//             .await
//             .unwrap();
//         // TODO: We should allow DER formatted certificates too, maybe based on file extension?
//         let cert = native_tls::Certificate::from_pem(&cert).unwrap();
//         connector.add_root_certificate(cert);
//     }
//
//     let tls = postgres_native_tls::MakeTlsConnector::new(connector.build().unwrap());
//     cfg.create_pool(runtime, tls)
// }

impl DocumentStore {
    pub async fn default_pool(
        &self,
    ) -> Result<deadpool_postgres::Pool, deadpool_postgres::CreatePoolError> {
        let db_url = self.env("DATABASE_URL").await.unwrap();

        if let Some(p) = self.pg_pools.get(db_url.as_str()) {
            return Ok(p.clone());
        }

        let pool = deadpool_postgres::Config {
            url: Some(db_url.to_string()),
            ..Default::default()
        }
        .create_pool(
            Some(deadpool_postgres::Runtime::Tokio1),
            tokio_postgres::NoTls,
        )?; // TODO: use the older create_pool function
        self.pg_pools.insert(db_url.to_string(), pool.clone());
        Ok(pool)
    }

    pub fn new<T: AsRef<camino::Utf8Path>>(
        root: T,
        pg_pools: actix_web::web::Data<dashmap::DashMap<String, deadpool_postgres::Pool>>,
    ) -> Self {
        Self {
            wasm_modules: dashmap::DashMap::new(),
            pg_pools,
            root: Path::new(root.as_ref().as_str()),
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_wasm(&self, path: &str) -> Result<wasmtime::Module, WasmReadError> {
        // TODO: implement wasm module on disc caching, so modules load faster across
        //       cache purge
        match self.wasm_modules.get(path) {
            Some(module) => Ok(module.clone()),
            None => {
                let source = self.read_content(&fastn_ds::Path::new(path)).await?;
                let module = wasmtime::Module::from_binary(&WASM_ENGINE, &source)?;
                if self
                    .env_bool("DEBUG_SERVE_SITE_FROM_FOLDER", false)
                    .await
                    .unwrap_or(false)
                {
                    // we are only storing compiled module if we are not in debug mode
                    self.wasm_modules.insert(path.to_string(), module.clone());
                }
                Ok(module)
            }
        }
    }

    pub fn root(&self) -> fastn_ds::Path {
        self.root.clone()
    }

    pub fn home(&self) -> fastn_ds::Path {
        fastn_ds::Path { path: home() }
    }

    pub async fn read_content(&self, path: &fastn_ds::Path) -> Result<Vec<u8>, ReadError> {
        use tokio::io::AsyncReadExt;

        tracing::debug!("read_content {}", &path);

        let mut file = tokio::fs::File::open(self.root.join(&path.path).path).await?;
        let mut contents = vec![];
        file.read_to_end(&mut contents).await?;
        Ok(contents)
    }

    pub async fn read_to_string(&self, path: &fastn_ds::Path) -> Result<String, ReadStringError> {
        self.read_content(path)
            .await
            .map_err(ReadStringError::ReadError)
            .and_then(|v| String::from_utf8(v).map_err(ReadStringError::UTF8Error))
    }

    pub async fn copy(&self, from: &fastn_ds::Path, to: &fastn_ds::Path) -> Result<(), WriteError> {
        tracing::debug!("copy from {} to {}", from, to);

        tokio::fs::copy(&from.path, &to.path).await?;
        Ok(())
    }

    pub async fn write_content(
        &self,
        path: &fastn_ds::Path,
        data: &[u8],
    ) -> Result<(), WriteError> {
        use tokio::io::AsyncWriteExt;

        tracing::debug!("write_content {}", &path);

        let full_path = self.root.join(&path.path);

        // Create the directory if it doesn't exist
        if let Some(parent) = full_path.parent() {
            if !parent.path.exists() {
                tokio::fs::create_dir_all(parent.path).await?;
            }
        }

        let mut file = tokio::fs::File::create(full_path.path).await?;
        file.write_all(data).await?;
        Ok(())
    }

    pub async fn read_dir(&self, path: &fastn_ds::Path) -> std::io::Result<tokio::fs::ReadDir> {
        // Todo: Return type should be ftd::interpreter::Result<Vec<fastn_ds::Dir>> not ftd::interpreter::Result<tokio::fs::ReadDir>
        tracing::debug!("read_dir {}", &path);

        tokio::fs::read_dir(&path.path).await
    }

    pub async fn rename(
        &self,
        from: &fastn_ds::Path,
        to: &fastn_ds::Path,
    ) -> Result<(), RenameError> {
        Ok(tokio::fs::rename(&from.path, &to.path).await?)
    }

    pub async fn remove(&self, path: &fastn_ds::Path) -> Result<(), RemoveError> {
        if !path.path.exists() {
            return Ok(());
        }
        if path.path.is_file() {
            tokio::fs::remove_file(&path.path).await?;
        } else if path.path.is_dir() {
            tokio::fs::remove_dir_all(&path.path).await?
        } else if path.path.is_symlink() {
            // TODO:
            // It can be a directory or a file
        }
        Ok(())
    }

    pub async fn get_all_file_path(
        &self,
        path: &fastn_ds::Path,
        ignore_paths: &[String],
    ) -> Vec<fastn_ds::Path> {
        let path = &path.path;
        let mut ignore_path = ignore::WalkBuilder::new(path);
        // ignore_paths.hidden(false); // Allow the linux hidden files to be evaluated
        ignore_path.overrides(package_ignores(ignore_paths, path).unwrap());
        ignore_path
            .build()
            .flatten()
            .filter_map(|x| {
                let path = camino::Utf8PathBuf::from_path_buf(x.into_path()).unwrap();
                if path.is_dir() {
                    None
                } else {
                    Some(fastn_ds::Path { path })
                }
            }) //todo: improve error message
            .collect::<Vec<fastn_ds::Path>>()
    }

    pub async fn exists(&self, path: &fastn_ds::Path) -> bool {
        path.path.exists()
    }

    pub async fn env_bool(&self, key: &str, default: bool) -> Result<bool, BoolEnvironmentError> {
        match self.env(key).await {
            Ok(t) if t.eq("true") => Ok(true),
            Ok(t) if t.eq("false") => Ok(false),
            Ok(value) => Err(BoolEnvironmentError::InvalidValue(value.to_string())),
            Err(EnvironmentError::NotSet(_)) => Ok(default),
        }
    }

    pub async fn env(&self, key: &str) -> Result<String, EnvironmentError> {
        std::env::var(key).map_err(|_| EnvironmentError::NotSet(key.to_string()))
    }
    /// Send an email
    /// to: (name, email)
    pub async fn send_email(
        &self,
        to: (&str, &str),
        subject: &str,
        body_html: String,
        mkind: fastn_ds::mail::EmailKind,
    ) -> Result<(), fastn_ds::mail::MailError> {
        let enable_email = self
            .env_bool("FASTN_ENABLE_EMAIL", true)
            .await
            .unwrap_or(true);

        let (name, email) = to;

        tracing::info!("sending mail of {mkind}");

        fastn_ds::mail::Mailer::send_raw(
            enable_email,
            self,
            format!("{} <{}>", name, email).parse()?,
            subject,
            body_html,
        )
        .await
    }

    // This method will connect client request to the out of the world
    #[tracing::instrument(skip(req, extra_headers))]
    pub async fn http<T>(
        &self,
        url: url::Url,
        req: &T,
        extra_headers: &std::collections::HashMap<String, String>,
    ) -> Result<fastn_ds::HttpResponse, HttpError>
    where
        T: RequestType,
    {
        if url.scheme() == "wasm+proxy" {
            return self.handle_wasm(url, req, extra_headers).await;
        }
        self.http_(url, req, extra_headers).await
    }

    pub async fn handle_wasm<T>(
        &self,
        url: url::Url,
        req: &T,
        extra_headers: &std::collections::HashMap<String, String>,
    ) -> Result<fastn_ds::HttpResponse, HttpError>
    where
        T: RequestType,
    {
        let mut headers: Vec<(String, Vec<u8>)> = req
            .headers()
            .iter()
            .map(|(k, v)| (k.as_str().to_string(), v.as_bytes().to_vec()))
            .collect();
        for (k, v) in extra_headers {
            headers.push((k.to_string(), v.as_bytes().to_vec()));
        }

        let wasm_file = url.to_string();
        let wasm_file = wasm_file.strip_prefix("wasm+proxy://").unwrap();
        let wasm_file = wasm_file.split_once(".wasm").unwrap().0;
        let _module = self.get_wasm(format!("{wasm_file}.wasm").as_str()).await?;

        // let resp = hostn_wasm::wasm_stuff_shared(
        //     ft_sys_shared::Request {
        //         uri: url.as_str().to_string(),
        //         method: req.method().to_string(),
        //         headers,
        //         body: req.body().to_vec(),
        //     },
        //     None,
        //     req.ud(self).await.map(fastn_ds::ud2ud),
        //     module,
        //     self.wasm_pg_pools.clone(),
        //     self.env("DATABASE_URL").await?,
        // )
        // .await?;
        //
        // Ok(resp.into())
        //
        todo!()
    }

    // This method will connect client request to the out of the world
    #[tracing::instrument(skip(req, extra_headers))]
    pub async fn http_<T>(
        &self,
        url: url::Url,
        req: &T,
        extra_headers: &std::collections::HashMap<String, String>,
    ) -> Result<fastn_ds::HttpResponse, HttpError>
    where
        T: RequestType,
    {
        let headers = req.headers();

        // Github doesn't allow trailing slash GET requests
        let url = if req.query_string().is_empty() {
            url.as_str().trim_end_matches('/').to_string()
        } else {
            format!(
                "{}/?{}",
                url.as_str().trim_end_matches('/'),
                req.query_string()
            )
        };

        let mut proxy_request = reqwest::Request::new(
            match req.method() {
                "GET" => reqwest::Method::GET,
                "POST" => reqwest::Method::POST,
                "PUT" => reqwest::Method::PUT,
                "DELETE" => reqwest::Method::DELETE,
                "PATCH" => reqwest::Method::PATCH,
                "HEAD" => reqwest::Method::HEAD,
                "OPTIONS" => reqwest::Method::OPTIONS,
                "TRACE" => reqwest::Method::TRACE,
                "CONNECT" => reqwest::Method::CONNECT,
                _ => reqwest::Method::GET,
            },
            reqwest::Url::parse(url.as_str())?,
        );

        *proxy_request.headers_mut() = headers.to_owned();

        for (header_key, header_value) in extra_headers {
            proxy_request.headers_mut().insert(
                reqwest::header::HeaderName::from_bytes(header_key.as_bytes()).unwrap(),
                reqwest::header::HeaderValue::from_str(header_value.as_str()).unwrap(),
            );
        }

        proxy_request.headers_mut().insert(
            reqwest::header::USER_AGENT,
            reqwest::header::HeaderValue::from_static("fastn"),
        );

        if let Some(cookies) = req.cookies_string() {
            proxy_request.headers_mut().insert(
                reqwest::header::COOKIE,
                reqwest::header::HeaderValue::from_str(cookies.as_str()).unwrap(),
            );
        }

        if let Some(ip) = req.get_ip() {
            proxy_request.headers_mut().insert(
                reqwest::header::FORWARDED,
                reqwest::header::HeaderValue::from_str(ip.as_str()).unwrap(),
            );
        }

        for header in fastn_ds::utils::ignore_headers() {
            proxy_request.headers_mut().remove(header);
        }

        tracing::info!("Request details");
        tracing::info!(
            url = ?proxy_request.url(),
            method = ?proxy_request.method(),
            headers = ?proxy_request.headers(),
            body = ?proxy_request.body(),
        );

        *proxy_request.body_mut() = Some(req.body().to_vec().into());
        let response = fastn_ds::http::DEFAULT_CLIENT
            .execute(proxy_request)
            .await?;

        tracing::info!("Response details");
        tracing::info!(status = ?response.status(),headers = ?response.headers());

        Ok(fastn_ds::reqwest_util::to_http_response(response).await?)
    }
}

#[derive(thiserror::Error, PartialEq, Debug)]
pub enum BoolEnvironmentError {
    #[error("Invalid value found for boolean: {0}")]
    InvalidValue(String),
}

#[derive(thiserror::Error, PartialEq, Debug)]
pub enum EnvironmentError {
    /// The environment variable is not set.
    /// Contains the name of the environment variable.
    #[error("environment variable not set: {0}")]
    NotSet(String),
}

fn home() -> camino::Utf8PathBuf {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => {
            eprintln!("Impossible to get your home directory");
            std::process::exit(1);
        }
    };
    camino::Utf8PathBuf::from_path_buf(home).expect("Issue while reading your home directory")
}
