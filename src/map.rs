use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::path::Path;

// XXX: switch to serde
use bincode::rustc_serialize::{encode, decode};
use rustc_serialize::{Encodable};
use rustc_serialize::json;

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

    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<MapLayer> {
        let mut f = try!(File::open(path));
        let mut s = String::new();
        try!(f.read_to_string(&mut s));
        // XXX: really should compose error instead of unwrapping
        Ok(json::decode(&s).unwrap())
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let mut f = try!(File::create(path));
        try!(f.write_all(self.serialize().as_bytes()));
        Ok(())
    }

    pub fn serialize(&self) -> String {
        json::encode(self).unwrap()
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
        let height = self.tiles.len() as u32 / self.width;
        if x >= self.width || y >= height {
            return Err(());
        }
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

#[test]
fn map_set_px_returns_err_on_overflow() {
    let mut map = MapLayer::new("assets/cotp", (16, 16), 25, vec![0;25*16]);

    // Passing a very large value as y is likely to overflow when trying
    // to compute the index!
    assert_eq!(
        map.set_px((1, u32::max_value()), 0),
        Err(()));
}
