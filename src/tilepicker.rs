use scene::{Scene, Rectangle};
use renderer::RenderContext;
use sdl2;
use sdl2::rect::Rect;
use sdl2::pixels::Color;

pub struct TilePicker {
    // widget has total ownership of its position for now
    rect: Rect,
}

impl TilePicker {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> TilePicker {
        TilePicker {
            rect: Rect::new_unwrap(x, y, width, height),
        }
    }

    /// Render the tileset picker.
    /// XXX: this is kind of an experiment in different render structures
    pub fn render(&self) -> Vec<Rectangle> {
        // First, fill the whole space with a sexy dark rectangle
        vec![
            Rectangle::filled(self.rect, Color::RGBA(32, 32, 32, 255))]
    }
}
