extern crate sdl2;
extern crate snes_spc;

use std::path::Path;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::rect::Rect;
use snes_spc::SnesSpc;

mod scene;
use scene::{Scene, Sprite,
    sprite, left, right, top, bottom, hstretch, vstretch};

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

    fn render(&self) -> Vec<Sprite> {
        let x = self.bounds.x();
        let y = self.bounds.y();
        let w = self.bounds.width() as i32;
        let h = self.bounds.height() as i32;

        vec![
            sprite(&self.part("tl"), right(x), bottom(y)),
            sprite(&self.part("t"), hstretch(x, x+w), bottom(y)),
            sprite(&self.part("tr"), left(x+w), bottom(y)),

            sprite(&self.part("l"), right(x), vstretch(y, y+h)),
            sprite(&self.part("r"), left(x+w), vstretch(y, y+h)),

            sprite(&self.part("bl"), right(x), top(y+h)),
            sprite(&self.part("b"), hstretch(x, x+w), top(y+h)),
            sprite(&self.part("br"), left(x+w), top(y+h)),
        ]
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();

    let window = video.window("Tarnished", 800, 500)
        .position_centered()
        .build()
        .unwrap();

    let renderer = window.renderer()
        .accelerated()
        .build()
        .unwrap();

    let mut renderer = scene::Renderer::new(renderer);

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

    // Draw a sprite
    let (mut x, mut y) = (0, 0);
    let starman = sprite("assets/starmanjr_lg", left(280), top(100));
    let starman2 = sprite("assets/starmanjr_lg", left(300), top(100));
    let textbox = Textbox::new("assets/box",
                               Rect::new_unwrap(16, 16, 64, 48));

    'mainloop: loop {
        for event in sdl_context.event_pump().unwrap().poll_iter() {
            match event {
                Event::Quit{..} |
                Event::KeyDown {keycode: Some(Keycode::Escape), ..} => {
                    break 'mainloop
                },
                Event::KeyDown {keycode: Some(Keycode::Up), ..} => {
                    y -= 8;
                },
                Event::KeyDown {keycode: Some(Keycode::Down), ..} => {
                    y += 8;
                },
                Event::KeyDown {keycode: Some(Keycode::Left), ..} => {
                    x -= 8;
                },
                Event::KeyDown {keycode: Some(Keycode::Right), ..} => {
                    x += 8;
                },
                _ => { }
            }
        }

        // XXX: note that box must be rendered before creating scene, since
        // scene borrows references to all the instructions added to it. This
        // is perhaps an API weakness; might end up just boxing visibles.
        let rendered_box = textbox.render();

        let mut scene = Scene::new();
        scene.set_viewport((x, y));
        scene.add(&starman, 1);
        scene.add(&starman2, 0);

        // XXX: doesn't make much sense to specify separate z-index for
        // every piece of this textbox when rendering piecewise to scene
        for p in &rendered_box {
            scene.add(p, 2);
        }

        scene.present(&mut renderer);
    }
}
