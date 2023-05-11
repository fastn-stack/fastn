/// Pencil lets you draw things. It either creates a window or captures the terminal.
/// Pencil comes with a bunch of drawing functions, which our layout-ing system uses
/// to render fastn documents.
#[async_trait::async_trait]
pub trait Pencil: Sized {
    type Error: std::error::Error + std::fmt::Debug;

    async fn init(&self) -> Result<(), <Self as Pencil>::Error>;
    async fn draw_rect(&self, w: u32, h: u32, fill: Option<fastn_surface::ColorValue>);
    fn run(&self);
}

