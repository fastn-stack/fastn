const DB: &str = "/Users/amitu/Projects/dotcom/fifthtry.sqlite";

fn main() {
    let conn = rusqlite::Connection::open(DB).unwrap();
    let q = r#"
        INSERT INTO ft_site
            (
                name,
                slug,
                is_static,
                is_public,
                is_editable,
                domain,
                created_at,
                updated_at,
                created_by,
                is_package
            )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING id
    "#;

    let mut stmt = conn.prepare(q).unwrap();
    let rows = stmt
        .query_and_then(
            rusqlite::params!["asdy", "asdy", 1, 1, 1, "asdy.fifthtry.site", 1, 1, 1, 1],
            |row| {
                let i = row.get::<usize, i32>(0)?;
                Ok::<i32, rusqlite::Error>(i)
            },
        )
        .unwrap();
    for r in rows {
        println!("r: {:?}", r);
    }
}
