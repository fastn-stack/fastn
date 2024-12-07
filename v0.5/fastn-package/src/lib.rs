#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]
#![warn(clippy::used_underscore_binding)]
#![allow(dead_code)]

mod read;

extern crate self as fastn_package;

pub struct Package {
    name: String,
    systems: Vec<System>,
    dependencies: Vec<Dependency>,
    pub auto_imports: Vec<AutoImport>,
    apps: Vec<App>,
}

#[derive(Clone)]
pub struct AutoImport {}

// -- system: design-system.com
// via: amitu.com/ds
// alias: some alias ;; if alias is not provided, this is globally passed
pub struct System {
    via: String,
    sensitive: bool,
    alias: Option<SystemAlias>,
}

pub struct SystemAlias(String);

pub struct Dependency {
    name: String,
    // vector of alias of the systems this dependency and everything downstream
    capabilities: Vec<SystemAlias>,
    dependencies: Vec<Dependency>,
    auto_imports: Vec<AutoImport>,
}

// -- path: /blog/
// allow: colorful-ds
pub struct CapabilityOverride {
    // capabilities for any url prefix can be overridden using this
    path: String,
    // if this is set, the global capabilities will be merged into .capabilities, else only
    // .capabilities will be used.
    inherit_global: bool,
    capabilities: Vec<SystemAlias>,
}

pub struct App {
    // this must already be added as a Dependency (not a system) and is its name
    name: String,
    mount_point: String,
    // apps can have their own apps
    apps: Vec<App>,
    // Dependency.capabilities will be merged with this when serving these routes
    capabilities: Vec<SystemAlias>,
}
