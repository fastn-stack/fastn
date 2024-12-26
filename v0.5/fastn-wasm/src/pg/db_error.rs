pub fn pg_to_shared(postgres_error: tokio_postgres::Error) -> ft_sys_shared::DbError {
    use std::error::Error;

    match postgres_error.code() {
        None => ft_sys_shared::DbError::UnableToSendCommand(postgres_error.to_string()),
        Some(c) => {
            let db_error = postgres_error
                .source()
                .and_then(|e| e.downcast_ref::<tokio_postgres::error::DbError>().cloned())
                .expect("It's a db error, because we've got a SQLState code above");

            let statement_position = db_error.position().map(|e| match e {
                tokio_postgres::error::ErrorPosition::Original(position)
                | tokio_postgres::error::ErrorPosition::Internal { position, .. } => {
                    *position as i32
                }
            });

            let kind = match c.code() {
                // code taken from diesel's PgResult::new()
                UNIQUE_VIOLATION => ft_sys_shared::DatabaseErrorKind::UniqueViolation,
                FOREIGN_KEY_VIOLATION => ft_sys_shared::DatabaseErrorKind::ForeignKeyViolation,
                SERIALIZATION_FAILURE => ft_sys_shared::DatabaseErrorKind::SerializationFailure,
                READ_ONLY_TRANSACTION => ft_sys_shared::DatabaseErrorKind::ReadOnlyTransaction,
                NOT_NULL_VIOLATION => ft_sys_shared::DatabaseErrorKind::NotNullViolation,
                CHECK_VIOLATION => ft_sys_shared::DatabaseErrorKind::CheckViolation,
                CONNECTION_EXCEPTION
                | CONNECTION_FAILURE
                | SQLCLIENT_UNABLE_TO_ESTABLISH_SQLCONNECTION
                | SQLSERVER_REJECTED_ESTABLISHMENT_OF_SQLCONNECTION => {
                    ft_sys_shared::DatabaseErrorKind::ClosedConnection
                }
                _ => ft_sys_shared::DatabaseErrorKind::Unknown,
            };
            ft_sys_shared::DbError::DatabaseError {
                kind,
                message: db_error.message().to_string(),
                details: db_error.detail().map(|s| s.to_string()),
                hint: db_error.hint().map(|s| s.to_string()),
                table_name: db_error.table().map(|s| s.to_string()),
                column_name: db_error.column().map(|s| s.to_string()),
                constraint_name: db_error.constraint().map(|s| s.to_string()),
                statement_position,
            }
        }
    }
}

const CONNECTION_EXCEPTION: &str = "08000";
const CONNECTION_FAILURE: &str = "08006";
const SQLCLIENT_UNABLE_TO_ESTABLISH_SQLCONNECTION: &str = "08001";
const SQLSERVER_REJECTED_ESTABLISHMENT_OF_SQLCONNECTION: &str = "08004";
const NOT_NULL_VIOLATION: &str = "23502";
const FOREIGN_KEY_VIOLATION: &str = "23503";
const UNIQUE_VIOLATION: &str = "23505";
const CHECK_VIOLATION: &str = "23514";
const READ_ONLY_TRANSACTION: &str = "25006";
const SERIALIZATION_FAILURE: &str = "40001";
