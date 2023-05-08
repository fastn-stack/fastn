extern crate self as fastn_surface;

/// fastn-surface is a way to describe UI in platform independent way
///
/// fastn-surface::UI is a way to describe arbitrary UI that can be displayed on various backends
/// like in browser, terminal or native. fastn-surface::UI exposes mutation methods, which can be
/// used to mutate the UI once the UI has been rendered on some surface. The mutations are applied
/// in an efficient way.
///
/// fastn-surface::UI also send UI events, like window resize, keyboard, mouse events etc. The
/// event includes data about the event.
#[cfg(feature = "native")]
pub mod native;
