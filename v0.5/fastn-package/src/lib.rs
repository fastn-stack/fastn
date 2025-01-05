#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]
#![warn(clippy::used_underscore_binding)]
#![allow(dead_code)]

extern crate self as fastn_package;

mod reader;

pub type UR<U, R> = fastn_continuation::UR<U, R, fastn_section::Error>;
pub use reader::Reader;

#[derive(Debug)]
pub struct MainPackage {
    pub name: String,
    pub systems: Vec<System>,
    pub apps: Vec<App>,
    pub packages: std::collections::HashMap<String, Package>,
}

#[derive(Debug, Default)]
pub struct Package {
    pub name: String,
    pub dependencies: Vec<Dependency>,
    pub auto_imports: Vec<AutoImport>,
    pub favicon: Option<String>,
}

#[derive(Clone, Debug)]
pub struct AutoImport {}

// -- system: design-system.com
// via: amitu.com/ds
// alias: some alias ;; if alias is not provided, this is globally passed
#[derive(Debug)]
pub struct System {
    via: String,
    sensitive: bool,
    alias: Option<SystemAlias>,
}

#[derive(Debug)]
pub struct SystemAlias(String);

#[derive(Debug)]
pub struct Dependency {
    pub name: String,
    // vector of alias of the systems this dependency and everything downstream
    capabilities: Vec<SystemAlias>,
    dependencies: Vec<Dependency>,
    auto_imports: Vec<AutoImport>,
}

// -- path: /blog/
// provide: colorful-ds
#[derive(Debug)]
pub struct CapabilityOverride {
    // capabilities for any url prefix can be overridden using this
    path: String,
    // if this is set, the global capabilities will be merged into .capabilities, else only
    // .capabilities will be used.
    inherit_global: bool,
    capabilities: Vec<SystemAlias>,
}

// -- app: /todo/
// provide: amitu.com/db
//
// -- or --
//
// -- app: /todo/
// name: arpita.com/todo
// provide: amitu.com/db
//
// -- dependency: arpita.com/todo-main
// provide: amitu.com/db
//
// -- end: app
#[derive(Debug)]
pub struct App {
    // this must already be added as a Dependency (not a system) and is its name
    name: String,
    mount_point: String,
    // apps can have their own apps
    apps: Vec<App>,
    // Dependency.capabilities will be merged with this when serving these routes
    capabilities: Vec<SystemAlias>,
}
