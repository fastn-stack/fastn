extern crate self as fastn_package;

mod initialize;
pub mod initializer;
pub(crate) mod sqlite;

pub use initialize::initialize;

// a rwlock  containing a hashmap of string to string
