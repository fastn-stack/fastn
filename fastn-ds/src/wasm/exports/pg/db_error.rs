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

            ft_sys_shared::DbError::DatabaseError {
                code: c.code().to_string(),
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
