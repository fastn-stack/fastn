#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub enum Warning {
    // say someone did `-- import: foo as foo`, this is not an error but a warning
    AliasNotNeeded,
    // we prefer dashes in identifiers, e.g., `foo-bar` instead of `foo_bar`
    UnderscoreInIdentifier,
    // we prefer lowercase in identifiers, e.g., `foo` instead of `Foo`
    IdentifierNotLowerCased,
    // e.g., a component defined something but never used it
    UnusedProperty,
    UsedIdentifierStartsWithUnderscore,
    // unused import
    UnusedImport,
    // unused dependency: if not used in the entire package at all
    UnusedDependency,
    // Doc missing on some public symbol
    DocMissing,
}
