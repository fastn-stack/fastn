#![deny(unused_crate_dependencies)]
#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![warn(clippy::used_underscore_binding)]

extern crate self as fastn_continuation;

mod provider;
mod result;
mod ur;

#[cfg(feature = "async_provider")]
pub use provider::{AsyncMutProvider, AsyncMutProviderWith, AsyncProvider, AsyncProviderWith};
pub use provider::{MutProvider, MutProviderWith, Provider, ProviderWith};
pub use result::Result;
pub use ur::{FromWith, UR};

use tracing as _;

pub trait Continuation {
    type Output;
    type Needed;
    type Found;
    fn continue_after(self, found: Self::Found) -> Result<Self>;
}
