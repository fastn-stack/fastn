fn extract_arguments(_query: &str) -> ftd::interpreter::Result<(String, Vec<String>)> {
    todo!()
}

#[cfg(test)]
mod test {
    #[track_caller]
    fn e(i: &str, o: &str, a: Vec<&str>) {
        let (query, arguments) = super::extract_arguments(i).unwrap();
        assert_eq!(query, o);
        assert_eq!(arguments, a);
    }

    #[test]
    fn extract_arguments() {
        e("hello", "hello", vec![]);
        e(
            "SELECT * FROM test where name = $name",
            "SELECT * FROM test where name = $1",
            vec!["name"],
        );
        e(
            "SELECT * FROM test where name = $name and full_name = $name",
            "SELECT * FROM test where name = $1 and full_name = $1",
            vec!["name"],
        );
    }
}
