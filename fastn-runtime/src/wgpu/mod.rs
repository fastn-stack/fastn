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

fn color_u8_to_f32(c: u8) -> f32 {
    c as f32 / 255.0
}
