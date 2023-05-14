mod boilerplate;
mod control;
mod event;
mod operations;
mod rectangles;
mod runtime;

pub use boilerplate::Wgpu;
pub use operations::OperationData;
pub use rectangles::RectData;
pub use runtime::render_document;
