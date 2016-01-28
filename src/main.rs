extern crate sdl2;
extern crate snes_spc;
extern crate time;
extern crate rustc_serialize;
extern crate bincode;

use bincode::rustc_serialize::{encode, decode};
use rustc_serialize::{Encodable, Decodable};
use rustc_serialize::json;
use std::path::Path;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::rect::Rect;
use snes_spc::SnesSpc;

mod scene;
use scene::{Scene, Sprite, sprite, Tile, HPos, VPos};

struct SpcPlayer {
    emulator: SnesSpc
}

impl AudioCallback for SpcPlayer {
    type Channel = i16;
    fn callback(&mut self, out: &mut [i16]) {
        self.emulator.play(out).unwrap();
    }
}

struct Textbox {
    base: String,
    bounds: Rect,
}

impl Textbox {
    fn new(base: &str, bounds: Rect) -> Textbox {
        Textbox {
            base: base.to_string(),
            bounds: bounds,
        }
    }

    fn part(&self, name: &str) -> String {
        Path::new(&self.base).join(name).to_string_lossy().into_owned()
    }

    // XXX: Note that here we've given Textbox a method which just dumps a
    // whole bunch of sprites into a vector. Later, something needs to take
    // those sprites and put them into a Scene. Unfortunately, Scene::add()
    // is where information about z-index comes in, and Textbox probably has
    // an opinion about that. At the very least, conventional wisdom dictates
    // that all the bits of the textbox should have the same z-level.
    //
    // Given that our design philosophy emphasizes decoupling and value-based
    // programming, the lazy approach of giving render() a &Scene parameter
    // is obviously a no-go. Other options include promoting the Instruction
    // type to public and making it the output of typical render functions,
    // and the input to Scene::add().
    fn render(&self) -> Vec<Sprite> {
        let w = self.bounds.width() as usize;
        let h = self.bounds.height() as usize;
        let x = self.bounds.x();
        let y = self.bounds.y();
        let r = x + w as i32;
        let b = y + h as i32;

        vec![
            sprite(&self.part("tl"), HPos::Right(x),      VPos::Bottom(y)),
            sprite(&self.part("t"),  HPos::Stretch(x, w), VPos::Bottom(y)),
            sprite(&self.part("tr"), HPos::Left(r),       VPos::Bottom(y)),
            sprite(&self.part("l"),  HPos::Right(x),      VPos::Stretch(y, h)),
            sprite(&self.part("r"),  HPos::Left(r),       VPos::Stretch(y, h)),
            sprite(&self.part("bl"), HPos::Right(x),      VPos::Top(b)),
            sprite(&self.part("b"),  HPos::Stretch(x, w), VPos::Top(b)),
            sprite(&self.part("br"), HPos::Left(r),       VPos::Top(b)),
        ]
    }
}

/// A grid of cells, drawn from a tileset.
#[derive(RustcEncodable, RustcDecodable)]
struct MapLayer {
    asset: String,
    tile_w: u32,
    tile_h: u32,
    width: u32,
    tiles: Vec<u32>,
}

impl MapLayer {
    fn new(asset: &str, (tw, th): (u32, u32), width: u32) -> MapLayer {
        MapLayer {
            asset: asset.into(),
            tile_w: tw,
            tile_h: th,
            width: width,
            tiles: Vec::new(),
        }
    }

    fn render(&self) -> Vec<Tile> {
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
    fn get_px(&self, (x, y): (u32, u32)) -> Option<u32> {
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
    fn set_px(&mut self, (x, y): (u32, u32), tile: u32) -> Result<(), ()> {
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

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();

    let window = video.window("Tarnished", 800, 500)
        .position_centered()
        .build()
        .unwrap();

    let mut renderer = window.renderer()
        .accelerated()
        .build()
        .unwrap();

    let mut renderer = scene::Renderer::new(renderer);
    renderer.set_global_scale(2.0, 2.0);

    // Start making noise
    let audio_subsystem = sdl_context.audio().unwrap();
    let desired_spec = AudioSpecDesired {
        freq: Some(32000),
        channels: Some(2),
        samples: None
    };

    let audio = audio_subsystem.open_playback(None, desired_spec, |spec| {
        println!("Audio initialized: {:?}", spec);
        SpcPlayer {
            emulator: SnesSpc::from_file("assets/otherworldly.spc").unwrap()
        }
    }).unwrap();

    audio.resume();

    // Draw some stuff
    let (mut off_x, mut off_y) = (0, 0);

    // XXX: the upscaled sprite here is now out of place; add view scaling.
    let starman = sprite("assets/starmanjr",
        HPos::Center(200), VPos::Center(125));
    let textbox = Textbox::new("assets/box",
        Rect::new_unwrap(16, 16, 128, 64));
    let mut map = MapLayer::new("assets/cotp", (16, 16), 25);
    map.tiles = vec![0;25*16];

    let mut frames = 0u32;
    let start = time::precise_time_ns();

    'mainloop: loop {
        for event in sdl_context.event_pump().unwrap().poll_iter() {
            match event {
                Event::Quit{..} |
                Event::KeyDown {keycode: Some(Keycode::Escape), ..} => {
                    break 'mainloop
                },
                Event::KeyDown {keycode: Some(Keycode::Up), ..} => {
                    off_y -= 8;
                },
                Event::KeyDown {keycode: Some(Keycode::Down), ..} => {
                    off_y += 8;
                },
                Event::KeyDown {keycode: Some(Keycode::Left), ..} => {
                    off_x -= 8;
                },
                Event::KeyDown {keycode: Some(Keycode::Right), ..} => {
                    off_x += 8;
                },
                Event::MouseButtonDown {x: x, y: y, ..} => {
                    // XXX: We have to explicitly transform by viewport here,
                    // eventually UI should be part of the scene (?)
                    let x = (x / 2 - off_x) as u32;
                    let y = (y / 2 - off_y) as u32;

                    // XXX: figure out this signed/unsigned and error condition
                    map.get_px((x, y)).map(
                        |tile| map.set_px((x, y), tile + 1));
                },
                _ => { }
            }
        }

        // XXX: note that box must be rendered before creating scene, since
        // scene borrows references to all the instructions added to it. This
        // is perhaps an API weakness; might end up just boxing visibles.
        let rendered_box = textbox.render();
        let rendered_map = map.render();

        let mut scene = Scene::new();
        scene.set_viewport((off_x, off_y));
        scene.add(&starman, 0);

        // XXX: doesn't make much sense to specify separate z-index for
        // every piece of this textbox when rendering piecewise to scene
        for p in &rendered_box {
            scene.add(p, 1);
        }

        for t in &rendered_map {
            scene.add(t, -1);
        }

        scene.present(&mut renderer);

        frames += 1;
    }

    let end = time::precise_time_ns();
    let fps = (frames as f64 / ((end - start) as f64 / 1e9)) as u32;
    println!("Rendered {} frames in {} ns; effective: {} fps",
             frames, end - start, fps);
}
