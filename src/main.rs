extern crate sdl2;
extern crate sdl2_ttf;
extern crate snes_spc;
extern crate time;
extern crate rustc_serialize;
extern crate bincode;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::rect::Rect;
use snes_spc::SnesSpc;

mod renderer;
mod scene;
mod textbox;
mod map;
use scene::{Scene, sprite, text};
use renderer::{Renderer, RenderContext, HPos, VPos};
use textbox::Textbox;
use map::MapLayer;

struct SpcPlayer {
    emulator: SnesSpc
}

impl AudioCallback for SpcPlayer {
    type Channel = i16;
    fn callback(&mut self, out: &mut [i16]) {
        self.emulator.play(out).unwrap();
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();
    let ttf = sdl2_ttf::init().unwrap();

    let window = video.window("Tarnished", 800, 500)
        .position_centered()
        .build()
        .unwrap();

    let renderer = window.renderer()
        .accelerated()
        .build()
        .unwrap();

    let mut render_context = RenderContext::new();
    let mut renderer = Renderer::new(renderer, ttf);
    renderer.set_global_scale(2.0, 2.0);
    //renderer.set_copy_scale(2.0, 2.0);

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

    let starman = sprite("assets/starmanjr",
        HPos::Center(200), VPos::Center(125));
    let textbox = Textbox::new("assets/box",
        Rect::new_unwrap(16, 16, 128, 64));
    let hello = text("Hello, world!", "assets/orangekid", 100, 100);

    let mut map = MapLayer::from_file("assets/map.json")
        .unwrap_or(MapLayer::new("assets/cotp", (16, 16), 25, vec![0;25*16]));

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
                Event::MouseWheel {y: scroll_y, ..} => {
                    let (_, x, y) = sdl_context.mouse().mouse_state();

                    // XXX: We have to explicitly transform by viewport here,
                    // eventually UI should be part of the scene (?)
                    let x = (x / 2 - off_x) as u32;
                    let y = (y / 2 - off_y) as u32;

                    // XXX: figure out this signed/unsigned and error condition
                    match map.get_px((x, y)) {
                        Some(tile) => {
                            map.set_px((x, y),
                                if scroll_y < 0 && tile > 0 {
                                    tile - 1
                                } else if scroll_y > 0 {
                                    tile + 1
                                } else {
                                    0
                                }).ok();
                        },
                        _ => {}
                    }
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

        scene.add(&hello, 2);

        scene.present(&mut renderer, &mut render_context);

        frames += 1;
    }

    let end = time::precise_time_ns();
    let fps = (frames as f64 / ((end - start) as f64 / 1e9)) as u32;
    println!("Rendered {} frames in {} ns; effective: {} fps",
             frames, end - start, fps);

    map.save("assets/map.json").unwrap();
}
