use crate::ColorValue;

#[derive(Debug, thiserror::Error)]
pub enum Error{}

pub struct Pencil {}

#[async_trait::async_trait]
impl fastn_surface::Pencil for Pencil {
    type Error = Error;

    async fn init(&self) -> Result<(), <Pencil as fastn_surface::Pencil>::Error> {
        todo!()
    }

    async fn draw_rect(&self, _w: u32, _h: u32, _fill: Option<ColorValue>) {
        todo!()
    }

    fn run(&self) {
        // fastn_surface::wgpu::render().await
    }
}

