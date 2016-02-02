use scene::Tile;

/// A grid of cells, drawn from a tileset.
#[derive(RustcEncodable, RustcDecodable)]
pub struct MapLayer {
    asset: String,
    tile_w: u32,
    tile_h: u32,
    width: u32,
    tiles: Vec<u32>,
}

impl MapLayer {
    pub fn new(asset: &str, (tw, th): (u32, u32), width: u32, tiles: Vec<u32>)
            -> MapLayer {
        MapLayer {
            asset: asset.into(),
            tile_w: tw,
            tile_h: th,
            width: width,
            tiles: tiles,
        }
    }

    pub fn render(&self) -> Vec<Tile> {
        let mut result = Vec::new();
        for (i, tile) in self.tiles.iter().enumerate() {
            let i = i as u32;
            let x = (i % self.width) * self.tile_w;
            let y = (i / self.width) * self.tile_h;
            result.push(Tile::new(&self.asset, *tile,
                self.tile_w, self.tile_h, x as i32, y as i32));
        }
        result
    }

    /// Get the tile value at a specified point (in pixels)
    pub fn get_px(&self, (x, y): (u32, u32)) -> Option<u32> {
        let x = x / self.tile_w;
        let y = y / self.tile_h;
        let i = (y * self.width + x) as usize;
        println!("Get from map at {}, {} => idx {}", x, y, i);
        if x >= self.width || i >= self.tiles.len() {
            None
        } else {
            Some(self.tiles[i])
        }
    }

    /// Set the tile value at a specified point (in pixels)
    pub fn set_px(&mut self, (x, y): (u32, u32), tile: u32) -> Result<(), ()> {
        let x = x / self.tile_w;
        let y = y / self.tile_h;
        let i = (y * self.width + x) as usize;
        if x >= self.width || i >= self.tiles.len() {
            Err(())
        } else {
            self.tiles[i] = tile;
            Ok(())
        }
    }
}
