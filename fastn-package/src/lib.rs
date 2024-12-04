#![deny(unused_crate_dependencies)]

extern crate self as fastn_package;

pub mod old_fastn;

const FASTN_PACKAGE_VARIABLE: &str = "fastn#package";

pub fn fastn_ftd_2023() -> &'static str {
    include_str!("../fastn_2023.ftd")
}
