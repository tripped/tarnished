// XXX: switch to serde
use rustc_serialize::json;
use scene::Tile;
use std::fs::File;
use std::io::{Read, Write};
use std::io;
use std::path::Path;

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
        json::decode(&s).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
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

    /// The width (in tiles) of the map layer.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// The height (in tiles) of the map layer.
    pub fn height(&self) -> u32 {
        self.tiles.len() as u32 / self.width
    }

    /// Map a point in pixels to the index of the tile containing that point.
    fn point_to_index(&self, (x, y): (i32, i32)) -> Option<usize> {
        let x = x / self.tile_w as i32;
        let y = y / self.tile_h as i32;

        // XXX: For now, maps have a hard top-left edge at 0,0. Maps with a
        // non-zero origin may need to support negative coordinates, but using
        // the i32 type here is mainly for convenience.
        if x < 0 || x >= self.width() as i32 ||
           y < 0 || y >= self.height() as i32 {
            return None;
        }

        Some((y * self.width() as i32 + x) as usize)
    }

    /// Get the tile value at a specified point (in pixels)
    pub fn _get_px(&self, point: (i32, i32)) -> Option<u32> {
        match self.point_to_index(point) {
            Some(index) => Some(self.tiles[index]),
            None => None,
        }
    }

    /// Set the tile value at a specified point (in pixels)
    pub fn set_px(&mut self, point: (i32, i32), tile: u32) -> Result<(), ()> {
        match self.point_to_index(point) {
            Some(index) => {
                self.tiles[index] = tile;
                Ok(())
            },
            None => Err(()),
        }
    }
}

#[test]
fn point_to_index() {
    let map = MapLayer::new("foobar", (16, 16), 25, vec![0;25*16]);
    assert_eq!(Some(0), map.point_to_index((0, 0)));
    assert_eq!(Some(1), map.point_to_index((1*16, 0)));
    assert_eq!(Some(2), map.point_to_index((2*16, 0)));
    assert_eq!(Some(25), map.point_to_index((0, 1*16)));
    assert_eq!(Some(50), map.point_to_index((0, 2*16)));
    assert_eq!(Some(0), map.point_to_index((0, 0)));
}

#[test]
fn width_works() {
    let map = MapLayer::new("foobar", (16, 16), 25, vec![0;25*16]);
    assert_eq!(map.width(), 25);
}

#[test]
fn height_works() {
    let map = MapLayer::new("foobar", (16, 16), 25, vec![0;25*16]);
    assert_eq!(map.height(), 16);
}

#[test]
fn get_px_returns_none_on_overflow() {
    let map = MapLayer::new("foobar", (16, 16), 25, vec![0;25*16]);

    // Passing a very large value as y is likely to overflow when trying
    // to compute the index!
    assert_eq!(None, map._get_px((1, i32::max_value())));
}

#[test]
fn set_px_returns_err_on_overflow() {
    let mut map = MapLayer::new("foobar", (16, 16), 25, vec![0;25*16]);
    assert_eq!(Err(()), map.set_px((1, i32::max_value()), 0));
}

#[test]
fn serialize_works() {
    let map = MapLayer::new("foobar", (16, 16), 4,
        vec![0, 1, 2, 3, 4, 5, 6, 7]);
    assert_eq!(
        map.serialize(),
        "{\"asset\":\"foobar\",\"tile_w\":16,\"tile_h\":16,\"width\":4,\
         \"tiles\":[0,1,2,3,4,5,6,7]}");
}
