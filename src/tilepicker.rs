use scene::{Rectangle, Tile};
use sdl2::rect::Rect;
use sdl2::pixels::Color;

pub struct TilePicker {
    tileset: String,
    tile_width: u32,
    tile_height: u32,
    // widget has total ownership of its position for now
    rect: Rect,
}

impl TilePicker {
    pub fn new(tileset: &str, tile_width: u32, tile_height: u32,
               x: i32, y: i32, width: u32, height: u32) -> TilePicker {
        TilePicker {
            tileset: tileset.into(),
            tile_width: tile_width,
            tile_height: tile_height,
            rect: Rect::new_unwrap(x, y, width, height),
        }
    }

    /// Render the tileset picker.
    /// XXX: this is kind of an experiment in different render structures
    pub fn render(&self) -> (Vec<Rectangle>, Vec<Tile>) {
        // First, fill the whole space with a sexy dark rectangle
        let rects = vec![
            Rectangle::filled(self.rect, Color::RGBA(32, 32, 32, 255))];

        // Draw the first tile!
        let tiles = vec![
            Tile::new(&self.tileset, 0, self.tile_width, self.tile_height,
                      0, 0)];

        (rects, tiles)
    }
}
