use scene::{Rectangle, Tex};
use sdl2::rect::Rect;
use sdl2::pixels::Color;

pub struct TilePicker {
    tileset: String,
    tile_width: u32,
    tile_height: u32,
    scale: f32,
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
            scale: 4.0,
            rect: Rect::new_unwrap(x, y, width, height),
            offset: 0,
            selected: 0,
        }
    }

    pub fn selected(&self) -> u32 {
        return self.selected;
    }

    pub fn scroll(&mut self, delta: i32) {
        if delta > 0 || self.offset >= delta.abs() as u32 {
            self.offset = (self.offset as i32 + delta) as u32;
        }
    }

    pub fn click(&mut self, abs_pos: (i32, i32)) -> bool {
        let (x, y) = abs_pos;

        // XXX: our version of sdl2 doesn't have Rect::contains??
        if x < self.rect.x() || x > self.rect.x() + self.rect.width() as i32 ||
            y < self.rect.y() || y > self.rect.y() + self.rect.height() as i32 {
            return false;
        }

        // Select a new tile
        let x = x - self.rect.x();
        // XXX: there has GOT to be a way to avoid these obnoxious casts
        let dx = (self.tile_width as f32 * self.scale) as i32 + 1;
        self.selected = (x / dx) as u32 + self.offset;
        return true;
    }

    /// Render the tileset picker.
    /// XXX: this is kind of an experiment in different render structures
    pub fn render(&self) -> (Vec<Rectangle>, Vec<Tex>) {
        // First, fill the whole space with a sexy dark rectangle
        let mut rects = vec![
            Rectangle::filled(self.rect, Color::RGBA(32, 32, 32, 255))];

        // Draw some tiles!
        // XXX: this duplicates the code in Renderer::draw_tile that determines
        // tileset layout, but that didn't seem like the appropriate place to
        // have it anyway. Dedup and move when appropriate.

        let mut tiles = Vec::new();

        // target render dimensions for each tile
        // XXX: seriously all this (x as f32 * y) as u32 crap is annoying
        let (w, h) = ((self.tile_width as f32 * self.scale) as u32,
                      (self.tile_height as f32 * self.scale) as u32);
        let padding = 1;

        let n = self.rect.width() / w;

        for i in 0..n {
            let tile = i + self.offset;
            let src = Rect::new_unwrap(
                (tile * self.tile_width) as i32, 0,
                self.tile_width, self.tile_height);
            let dst = Rect::new_unwrap(
                (i * (w + padding)) as i32, padding as i32, w, h);

            tiles.push(Tex::new(&self.tileset, Some(src), dst));

            // Add a rectangle if this tile is selected
            if tile == self.selected {
                rects.push(
                    Rectangle::filled(
                        Rect::new_unwrap(
                            -(padding as i32) + (i * (w+padding)) as i32, 0,
                            w + 2*padding, h + 2*padding),
                        Color::RGBA(255, 0, 0, 255)));
            }
        }

        (rects, tiles)
    }
}
