use scene::{Rectangle, Tex};
use sdl2::rect::Rect;
use sdl2::pixels::Color;

pub struct TilePicker {
    tileset: String,
    tile_width: u32,
    tile_height: u32,
    // widget has total ownership of its position for now
    rect: Rect,
    offset: u32,
    selected: u32,
}

impl TilePicker {
    pub fn new(tileset: &str, tile_width: u32, tile_height: u32,
               x: i32, y: i32, width: u32, height: u32) -> TilePicker {
        TilePicker {
            tileset: tileset.into(),
            tile_width: tile_width,
            tile_height: tile_height,
            rect: Rect::new_unwrap(x, y, width, height),
            offset: 0,
            selected: 0,
        }
    }

    pub fn selected(&self) -> u32 {
        return self.selected;
    }

    pub fn scroll(&mut self, delta: i32) {
        if delta > 0 || self.offset > delta.abs() as u32 {
            self.offset = (self.offset as i32 + delta) as u32;
        }
    }

    /// Render the tileset picker.
    /// XXX: this is kind of an experiment in different render structures
    pub fn render(&self) -> (Vec<Rectangle>, Vec<Tex>) {
        // First, fill the whole space with a sexy dark rectangle
        let rects = vec![
            Rectangle::filled(self.rect, Color::RGBA(32, 32, 32, 255))];

        // Draw some tiles!
        // XXX: this duplicates the code in Renderer::draw_tile that determines
        // tileset layout, but that didn't seem like the appropriate place to
        // have it anyway. Dedup and move when appropriate.

        let mut tiles = Vec::new();
        let (w, h) = (64, 64); // target render dimensions for each tile
        let padding = 1;

        let n = self.rect.width() / w;

        for i in (0..n) {
            let tile = i + self.offset;
            let src = Rect::new_unwrap(
                (tile * self.tile_width) as i32, 0,
                self.tile_width, self.tile_height);
            let dst = Rect::new_unwrap(
                (i * (w + padding)) as i32, padding as i32, w, h);

            tiles.push(Tex::new(&self.tileset, Some(src), dst));
        }

        (rects, tiles)
    }
}
