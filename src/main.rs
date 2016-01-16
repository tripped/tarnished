extern crate sdl2;
extern crate snes_spc;

use std::path::Path;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::rect::Rect;
use snes_spc::SnesSpc;

mod scene;
use scene::{Scene, Sprite, sprite, HPos, VPos};

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
            sprite(&self.part("br"), HPos::Right(r),      VPos::Top(b)),
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
    let starman = sprite("assets/starmanjr_lg",
                         HPos::Center(400), VPos::Center(250));
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

        // XXX: doesn't make much sense to specify separate z-index for
        // every piece of this textbox when rendering piecewise to scene
        for p in &rendered_box {
            scene.add(p, 2);
        }

        scene.present(&mut renderer);
    }
}
