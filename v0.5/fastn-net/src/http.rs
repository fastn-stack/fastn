#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Request {
    pub uri: String,
    pub method: String,
    pub headers: Vec<(String, Vec<u8>)>,
}

impl From<hyper::http::request::Parts> for Request {
    fn from(r: hyper::http::request::Parts) -> Self {
        let mut headers = vec![];
        for (k, v) in r.headers {
            let k = match k {
                Some(v) => v.to_string(),
                None => continue,
            };
            headers.push((k, v.as_bytes().to_vec()));
        }

        Request {
            uri: r.uri.to_string(),
            method: r.method.to_string(),
            headers,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Response {
    pub status: u16,
    pub headers: Vec<(String, Vec<u8>)>,
}

pub type ProxyResponse<E = hyper::Error> =
    hyper::Response<http_body_util::combinators::BoxBody<hyper::body::Bytes, E>>;
pub type ProxyResult<E = hyper::Error> = eyre::Result<ProxyResponse<E>>;

#[allow(dead_code)]
pub fn server_error_<E>(s: String) -> ProxyResponse<E> {
    bytes_to_resp(s.into_bytes(), hyper::StatusCode::INTERNAL_SERVER_ERROR)
}

#[allow(dead_code)]
pub fn not_found_(m: String) -> ProxyResponse {
    bytes_to_resp(m.into_bytes(), hyper::StatusCode::NOT_FOUND)
}

pub fn bad_request_<E>(m: String) -> ProxyResponse<E> {
    bytes_to_resp(m.into_bytes(), hyper::StatusCode::BAD_REQUEST)
}

#[macro_export]
macro_rules! server_error {
    ($($t:tt)*) => {{
        $crate::http::server_error_(format!($($t)*))
    }};
}

#[macro_export]
macro_rules! not_found {
    ($($t:tt)*) => {{
        $crate::http::not_found_(format!($($t)*))
    }};
}

#[macro_export]
macro_rules! bad_request {
    ($($t:tt)*) => {{
        $crate::http::bad_request_(format!($($t)*))
    }};
}

#[allow(dead_code)]
pub fn redirect<S: AsRef<str>>(url: S) -> ProxyResponse {
    let mut r = bytes_to_resp(vec![], hyper::StatusCode::PERMANENT_REDIRECT);
    *r.headers_mut().get_mut(hyper::header::LOCATION).unwrap() = url.as_ref().parse().unwrap();
    r
}

pub fn bytes_to_resp<E>(bytes: Vec<u8>, status: hyper::StatusCode) -> ProxyResponse<E> {
    use http_body_util::BodyExt;

    let mut r = hyper::Response::new(
        http_body_util::Full::new(hyper::body::Bytes::from(bytes))
            .map_err(|e| match e {})
            .boxed(),
    );
    *r.status_mut() = status;
    r
}

pub fn vec_u8_to_bytes(req: hyper::Request<Vec<u8>>) -> hyper::Request<hyper::body::Bytes> {
    let (head, body) = req.into_parts();
    let body = hyper::body::Bytes::from(body);
    hyper::Request::from_parts(head, body)
}

pub async fn incoming_to_bytes(
    req: hyper::Request<hyper::body::Incoming>,
) -> eyre::Result<hyper::Request<hyper::body::Bytes>> {
    use http_body_util::BodyDataStream;
    use tokio_stream::StreamExt;

    let (head, body) = req.into_parts();
    let mut stream = BodyDataStream::new(body);
    let mut body = bytes::BytesMut::new();

    while let Some(chunk) = stream.next().await {
        body.extend_from_slice(&chunk?);
    }

    Ok(hyper::Request::from_parts(head, body.freeze()))
}

pub async fn response_to_static(
    resp: ProxyResult,
) -> eyre::Result<hyper::Response<std::borrow::Cow<'static, [u8]>>> {
    use http_body_util::BodyExt;
    let resp = resp?;

    let (parts, body) = resp.into_parts();
    let bytes = body.collect().await?.to_bytes();

    let new_resp = hyper::Response::from_parts(parts, std::borrow::Cow::Owned(bytes.to_vec()));

    Ok(new_resp)
}
