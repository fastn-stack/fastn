pub fn find_all_files_matching_extension_recursively(
    dir: impl AsRef<std::path::Path> + std::fmt::Debug,
    extension: &str,
) -> Vec<std::path::PathBuf> {
    let mut files = vec![];
    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            files.extend(find_all_files_matching_extension_recursively(
                &path, extension,
            ));
        } else {
            match path.extension() {
                Some(ext) if ext == extension => files.push(path),
                _ => continue,
            }
        }
    }
    files
}

/**
 * Removes the comment prefix (if any) from the given value.
 *
 * # Parameters
 *
 * - `value` - a mutable reference to an option of a String to remove the comment prefix from
 *
 * This function will check if the string value starts with a '/' or '\/'. If it starts with a '/', the value will be
 * set to None and the function will return. If it starts with '\/', the function will remove the first '\'
 * character from the value.
 */
pub(crate) fn remove_value_comment(value: &mut Option<String>) {
    if let Some(v) = value {
        if v.starts_with('/') {
            *value = None;
            return;
        }

        if v.starts_with(r"\/") {
            *v = v.trim_start_matches('\\').to_string();
        }
    }
}

pub const CAPTION: &str = "$caption$";
pub const INLINE_IF: &str = " if ";
pub const IF: &str = "if";

/**
 * Constructs a parse error Result of a specific type
 *
 * # Parameters
 *
 * - `m` - a message to add to the parse error
 * - `doc_id` - a reference to a string representing the document id
 * - `line_number` - a usize representing the line number where the error occured
 *
 * # Returns
 *
 * A Result of the specified type, with an error variant of `Error::ParseError`
 * containing the provided message, doc_id and line_number
 */
pub fn parse_error<T, S1>(m: S1, doc_id: &str, line_number: usize) -> ftd_p1::Result<T>
where
    S1: Into<String>,
{
    Err(ftd_p1::Error::ParseError {
        message: m.into(),
        doc_id: doc_id.to_string(),
        line_number,
    })
}

/**
 * Converts an i32 to a usize
 *
 * # Parameters
 *
 * - `i` - the i32 to convert
 *
 * # Returns
 *
 * A usize that is the result of the conversion. If the input i32 is negative, returns 0.
 */
pub(crate) fn i32_to_usize(i: i32) -> usize {
    if i < 0 { 0 } else { i as usize }
}
