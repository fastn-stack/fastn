/*
Requirements:

dir-name: package name or vice versa
package: name same as dir name
dn-url: optional

- need to create a directory
- create a index.ftd
with hello world content
- create FPM.ftd file with required config

FMP.ftd

-- import: fpm

-- fpm.package: <package-name>
download-base-url: <dummy>

index.ftd

-- ftd.text: Hello World
 */

const FPM_FTD: &str = r#"

-- import: fpm

-- fpm.package: PACKAGE_NAME
download-base-url: DOWNLOAD_BASE_URL

"#;

const INDEX_FTD: &str = r#"
-- import: fpm
-- ftd.text: Hello World
"#;

pub fn init(root: &std::path::Path, package_name: &str, db_url: Option<String>) -> fpm::Result<()> {
    if root.join(package_name.trim_start_matches('/')).exists() {
        println!("directory already present: {}", package_name);
        return fpm::Error::generic_err(format!("directory already present: {}", package_name));
    }

    Ok(())
}
