async fn create_pool(
    req_config: &fastn_core::RequestConfig,
) -> Result<deadpool_postgres::Pool, deadpool_postgres::CreatePoolError> {
    let mut cfg = deadpool_postgres::Config::new();
    cfg.libpq_style_connection_string = match req_config.config.ds.env("FASTN_DB_URL").await {
        Ok(v) => Some(v),
        Err(_) => {
            fastn_core::warning!("FASTN_DB_URL is not set");
            return Err(deadpool_postgres::CreatePoolError::Config(
                deadpool_postgres::ConfigError::ConnectionStringInvalid,
            ));
        }
    };
    cfg.manager = Some(deadpool_postgres::ManagerConfig {
        // TODO: make this configurable
        recycling_method: deadpool_postgres::RecyclingMethod::Verified,
    });
    let runtime = Some(deadpool_postgres::Runtime::Tokio1);

    if let Ok(true) = req_config
        .config
        .ds
        .env_bool("FASTN_PG_DANGER_ENABLE_SSL", false)
        .await
    {
        fastn_core::warning!(
            "FASTN_PG_DANGER_DISABLE_SSL is set to false, this is not recommended for production use",
        );
        cfg.ssl_mode = Some(deadpool_postgres::SslMode::Disable);
        return cfg.create_pool(runtime, tokio_postgres::NoTls);
    }

    let mut connector = native_tls::TlsConnector::builder();

    match req_config
        .config
        .ds
        .env("FASTN_PG_SSL_MODE")
        .await
        .as_deref()
    {
        Err(_) | Ok("require") => {
            cfg.ssl_mode = Some(deadpool_postgres::SslMode::Require);
        }
        Ok("prefer") => {
            fastn_core::warning!(
                "FASTN_PG_SSL_MODE is set to prefer, which roughly means \"I don't care about \
                encryption, but I wish to pay the overhead of encryption if the server supports it.\"\
                and is not recommended for production use",
            );
            cfg.ssl_mode = Some(deadpool_postgres::SslMode::Prefer);
        }
        Ok(v) => {
            // TODO: openssl also allows `verify-ca` and `verify-full` but native_tls does not
            fastn_core::warning!(
                "FASTN_PG_SSL_MODE is set to {}, which is invalid, only allowed values are prefer and require",
                v,
            );
            return Err(deadpool_postgres::CreatePoolError::Config(
                deadpool_postgres::ConfigError::ConnectionStringInvalid,
            ));
        }
    }

    if let Ok(true) = req_config
        .config
        .ds
        .env_bool("FASTN_PG_DANGER_ALLOW_UNVERIFIED_CERTIFICATE", false)
        .await
    {
        fastn_core::warning!(
            "FASTN_PG_DANGER_ALLOW_UNVERIFIED_CERTIFICATE is set to true, this is not \
            recommended for production use",
        );
        connector.danger_accept_invalid_certs(true);
    }

    if let Ok(cert) = req_config.config.ds.env("FASTN_PG_CERTIFICATE").await {
        // TODO: This does not work with Heroku certificate.
        let cert = req_config
            .config
            .ds
            .read_content(&fastn_ds::Path::new(cert))
            .await
            .unwrap();
        // TODO: We should allow DER formatted certificates too, maybe based on file extension?
        let cert = native_tls::Certificate::from_pem(&cert).unwrap();
        connector.add_root_certificate(cert);
    }

    let tls = postgres_native_tls::MakeTlsConnector::new(connector.build().unwrap());
    cfg.create_pool(runtime, tls)
}

// TODO: I am a little confused about the use of `tokio::sync` here, both sides are async, so why
//       do we need to use `tokio::sync`? Am I doing something wrong? How do I prove/verify that
//       this is correct?
static POOL_RESULT: tokio::sync::OnceCell<
    Result<deadpool_postgres::Pool, deadpool_postgres::CreatePoolError>,
> = tokio::sync::OnceCell::const_new();

static EXECUTE_QUERY_LOCK: once_cell::sync::Lazy<tokio::sync::Mutex<()>> =
    once_cell::sync::Lazy::new(|| tokio::sync::Mutex::new(()));

async fn pool(
    req_config: &fastn_core::RequestConfig,
) -> &'static Result<deadpool_postgres::Pool, deadpool_postgres::CreatePoolError> {
    POOL_RESULT
        .get_or_init(|| async { create_pool(req_config).await })
        .await
}

pub async fn process(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc<'_>,
    req_config: &fastn_core::RequestConfig,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    let (headers, query) = super::sqlite::get_p1_data("pg", &value, doc.name)?;

    let query_response = execute_query(
        query.as_str(),
        doc,
        value.line_number(),
        headers,
        req_config,
    )
    .await;

    match query_response {
        Ok(result) => {
            super::sqlite::result_to_value(Ok(result), kind, doc, &value, super::sql::STATUS_OK)
        }
        Err(e) => super::sqlite::result_to_value(
            Err(e.to_string()),
            kind,
            doc,
            &value,
            super::sql::STATUS_ERROR,
        ),
    }
}

type PGData = dyn postgres_types::ToSql + Sync;

struct QueryArgs {
    args: Vec<Box<PGData>>,
}

impl QueryArgs {
    fn pg_args(&self) -> Vec<&PGData> {
        self.args.iter().map(|x| x.as_ref()).collect()
    }
}

fn resolve_variable_from_doc(
    doc: &ftd::interpreter::TDoc<'_>,
    var: &str,
    e: &postgres_types::Type,
    line_number: usize,
) -> ftd::interpreter::Result<Box<PGData>> {
    let thing = match doc.get_thing(var, line_number) {
        Ok(ftd::interpreter::Thing::Variable(v)) => v.value.resolve(doc, line_number)?,
        Ok(v) => {
            return ftd::interpreter::utils::e2(
                format!("{var} is not a variable, it's a {v:?}"),
                doc.name,
                line_number,
            )
        }
        Err(e) => {
            return ftd::interpreter::utils::e2(
                format!("${var} not found in the document: {e:?}"),
                doc.name,
                line_number,
            )
        }
    };

    Ok(match (e, thing) {
        (&postgres_types::Type::TEXT, ftd::interpreter::Value::String { text, .. }) => {
            Box::new(text)
        }
        (&postgres_types::Type::VARCHAR, ftd::interpreter::Value::String { text, .. }) => {
            Box::new(text)
        }
        (&postgres_types::Type::INT4, ftd::interpreter::Value::Integer { value, .. }) => {
            Box::new(value as i32)
        }
        (&postgres_types::Type::INT8, ftd::interpreter::Value::Integer { value, .. }) => {
            Box::new(value)
        }
        (&postgres_types::Type::FLOAT4, ftd::interpreter::Value::Decimal { value, .. }) => {
            Box::new(value as f32)
        }
        (&postgres_types::Type::FLOAT8, ftd::interpreter::Value::Decimal { value, .. }) => {
            Box::new(value)
        }
        (&postgres_types::Type::BOOL, ftd::interpreter::Value::Boolean { value, .. }) => {
            Box::new(value)
        }
        (e, a) => {
            return ftd::interpreter::utils::e2(
                format!("for {} postgresql expected ${:?}, found {:?}", var, e, a),
                doc.name,
                line_number,
            )
        }
    })
}

fn resolve_variable_from_headers(
    doc: &ftd::interpreter::TDoc<'_>,
    headers: &ftd::ast::HeaderValues,
    var: &str,
    e: &postgres_types::Type,
    doc_name: &str,
    line_number: usize,
) -> ftd::interpreter::Result<Option<Box<PGData>>> {
    let header = match headers.optional_header_by_name(var, doc_name, line_number)? {
        Some(v) => v,
        None => return Ok(None),
    };

    if let ftd::ast::VariableValue::String { value, .. } = &header.value {
        if let Some(stripped) = value.strip_prefix('$') {
            return resolve_variable_from_doc(doc, stripped, e, line_number).map(Some);
        }
    }

    fn friendlier_error<T, E: ToString>(
        r: Result<T, E>,
        var: &str,
        val: &str,
        into: &str,
        doc_name: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<T> {
        match r {
            Ok(r) => Ok(r),
            Err(e) => ftd::interpreter::utils::e2(
                format!(
                    "failed to parse `{var}: {val}` into {into}: {e}",
                    e = e.to_string()
                ),
                doc_name,
                line_number,
            ),
        }
    }

    Ok(match (e, &header.value) {
        (&postgres_types::Type::TEXT, ftd::ast::VariableValue::String { value, .. }) => {
            Some(Box::new(value.to_string()))
        }
        (&postgres_types::Type::VARCHAR, ftd::ast::VariableValue::String { value, .. }) => {
            Some(Box::new(value.to_string()))
        }
        (&postgres_types::Type::INT4, ftd::ast::VariableValue::String { value, .. }) => {
            Some(Box::new(friendlier_error(
                value.parse::<i32>(),
                var,
                value,
                "i32",
                doc_name,
                line_number,
            )?))
        }
        (&postgres_types::Type::INT8, ftd::ast::VariableValue::String { value, .. }) => {
            Some(Box::new(friendlier_error(
                value.parse::<i64>(),
                var,
                value,
                "i64",
                doc_name,
                line_number,
            )?))
        }
        (&postgres_types::Type::FLOAT4, ftd::ast::VariableValue::String { value, .. }) => {
            Some(Box::new(friendlier_error(
                value.parse::<f32>(),
                var,
                value,
                "f32",
                doc_name,
                line_number,
            )?))
        }
        (&postgres_types::Type::FLOAT8, ftd::ast::VariableValue::String { value, .. }) => {
            Some(Box::new(friendlier_error(
                value.parse::<f64>(),
                var,
                value,
                "f64",
                doc_name,
                line_number,
            )?))
        }
        (&postgres_types::Type::BOOL, ftd::ast::VariableValue::String { value, .. }) => {
            Some(Box::new(friendlier_error(
                value.parse::<bool>(),
                var,
                value,
                "bool",
                doc_name,
                line_number,
            )?))
        }
        (e, a) => {
            return ftd::interpreter::utils::e2(
                format!("for {} postgresql expected ${:?}, found {:?}", var, e, a),
                doc.name,
                line_number,
            )
        }
    })
}

fn prepare_args(
    query_args: Vec<String>,
    expected_args: &[postgres_types::Type],
    doc: &ftd::interpreter::TDoc<'_>,
    line_number: usize,
    headers: ftd::ast::HeaderValues,
) -> ftd::interpreter::Result<QueryArgs> {
    if expected_args.len() != query_args.len() {
        return ftd::interpreter::utils::e2(
            format!(
                "expected {} arguments, found {}",
                expected_args.len(),
                query_args.len()
            ),
            doc.name,
            line_number,
        );
    }
    let mut args = vec![];
    for (e, a) in expected_args.iter().zip(query_args) {
        args.push(
            match resolve_variable_from_headers(doc, &headers, &a, e, doc.name, line_number)? {
                Some(v) => v,
                None => resolve_variable_from_doc(doc, &a, e, line_number)?,
            },
        );
    }
    Ok(QueryArgs { args })
}

async fn execute_query(
    query: &str,
    doc: &ftd::interpreter::TDoc<'_>,
    line_number: usize,
    headers: ftd::ast::HeaderValues,
    req_config: &fastn_core::RequestConfig,
) -> ftd::interpreter::Result<Vec<Vec<serde_json::Value>>> {
    let _lock = EXECUTE_QUERY_LOCK.lock().await;

    let (query, query_args) = super::sql::extract_arguments(query)?;
    let client = pool(req_config)
        .await
        .as_ref()
        .unwrap()
        .get()
        .await
        .unwrap();

    let stmt = client.prepare_cached(query.as_str()).await.unwrap();

    let args = prepare_args(query_args, stmt.params(), doc, line_number, headers)?;
    let rows = client.query(&stmt, &args.pg_args()).await.unwrap();
    let mut result: Vec<Vec<serde_json::Value>> = vec![];

    for r in rows {
        result.push(row_to_json(r, doc.name, line_number)?)
    }

    Ok(result)
}

fn row_to_json(
    r: tokio_postgres::Row,
    doc_name: &str,
    line_number: usize,
) -> ftd::interpreter::Result<Vec<serde_json::Value>> {
    let columns = r.columns();
    let mut row: Vec<serde_json::Value> = Vec::with_capacity(columns.len());
    for (i, column) in columns.iter().enumerate() {
        match column.type_() {
            &postgres_types::Type::BOOL => row.push(serde_json::Value::Bool(r.get(i))),
            &postgres_types::Type::INT2 => {
                row.push(serde_json::Value::Number(r.get::<usize, i16>(i).into()))
            }
            &postgres_types::Type::INT4 => {
                row.push(serde_json::Value::Number(r.get::<usize, i32>(i).into()))
            }
            &postgres_types::Type::INT8 => {
                row.push(serde_json::Value::Number(r.get::<usize, i64>(i).into()))
            }
            &postgres_types::Type::FLOAT4 => row.push(serde_json::Value::Number(
                serde_json::Number::from_f64(r.get::<usize, f32>(i) as f64).unwrap(),
            )),
            &postgres_types::Type::FLOAT8 => row.push(serde_json::Value::Number(
                serde_json::Number::from_f64(r.get::<usize, f64>(i)).unwrap(),
            )),
            &postgres_types::Type::TEXT => row.push(serde_json::Value::String(r.get(i))),
            &postgres_types::Type::CHAR => row.push(serde_json::Value::String(r.get(i))),
            &postgres_types::Type::VARCHAR => row.push(serde_json::Value::String(r.get(i))),
            &postgres_types::Type::JSON => row.push(r.get(i)),

            t => {
                return ftd::interpreter::utils::e2(
                    format!("type {} not yet supported", t),
                    doc_name,
                    line_number,
                )
            }
        }
    }

    Ok(row)
}

/*
FASTN_PG_URL=postgres://amitu@localhost/amitu fastn serve
 */

/*
CREATE TABLE users (
    id SERIAL,
    name TEXT,
    department TEXT
);

INSERT INTO "users" (name, department) VALUES ('jack', 'design');
INSERT INTO "users" (name, department) VALUES ('jill', 'engineering');

 */

/*
-- import: fastn/processors as pr

-- record person:
integer id:
string name:
string department:


-- integer id: 1

-- ftd.integer: $id

-- person list people:
$processor$: pr.pg

SELECT * FROM "users" where id >= $id ;


-- ftd.text: data from db

-- ftd.text: $p.name
$loop$: $people as $p



-- integer int_2:
$processor$: pr.pg

SELECT 20::int2;

-- ftd.integer: $int_2

-- integer int_4:
$processor$: pr.pg

SELECT 40::int4;

-- ftd.integer: $int_4

-- integer int_8:
$processor$: pr.pg

SELECT 80::int8;

-- ftd.integer: $int_8






-- decimal d_4:
$processor$: pr.pg
val: 4.01
note: `SELECT $val::FLOAT8` should work but doesn't

SELECT 1.0::FLOAT8;

-- ftd.decimal: $d_4


-- decimal d_8:
$processor$: pr.pg

SELECT 80.0::FLOAT8;

-- ftd.decimal: $d_8
*/

/*
PREPARE my_query AS
SELECT * FROM "users" where id >= $1;
SELECT parameter_types FROM pg_prepared_statements WHERE name = 'my_query';
DEALLOCATE my_query;
 */
