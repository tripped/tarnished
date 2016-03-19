extern crate sdl2;
extern crate sdl2_ttf;
extern crate snes_spc;
extern crate time;
extern crate rustc_serialize;
extern crate carboxyl;
extern crate num;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use snes_spc::SnesSpc;
use num::rational::Ratio;

mod renderer;
mod scene;
mod textbox;
mod tilepicker;
mod map;
mod brobot;

use scene::{Scene, sprite, text};
use renderer::{RenderContext, HPos, VPos};
use textbox::Textbox;
use tilepicker::TilePicker;
use map::MapLayer;
use brobot::{Brobot};

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

    let screen_w = 960;
    let screen_h = 600;
    let window = video.window("Tarnished", screen_w, screen_h)
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let mut renderer = window.renderer()
        .accelerated()
        .build()
        .unwrap();

    let mut render_context = RenderContext::new(ttf);

    // The default scaling factor we'll apply when rendering
    let default_scale = Ratio::from_integer(4u32);

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
            emulator: SnesSpc::from_file("assets/cotp.spc").unwrap()
        }
    }).unwrap();

    audio.resume();

    // Draw some stuff
    let (mut off_x, mut off_y) = (0, 0);

    let starman = sprite("assets/starmanjr",
        HPos::Center(200), VPos::Center(125));
    let textbox = Textbox::new("assets/box",
        Rect::new_unwrap(12, 12, 32, 16));
    let hello = text("$0.00", "assets/orangekid", 30, 18);

    let mut map = MapLayer::from_file("assets/map.json")
        .unwrap_or(MapLayer::new("assets/cotp", (16, 16), 25, vec![0;25*16]));

    // XXX: note that this widget is rendered in unscaled space, so its width
    // of 960 is actually the full window width. Soon these different spaces
    // should be managed more cleanly.
    let mut tilepicker = TilePicker::new("assets/cotp", 16, 16, 0, 0, 960, 66);
    let mut painting = false;

    let mut frames = 0u32;
    let start = time::precise_time_ns();

    let mut hero = Brobot::new("assets/hero", 16, 24, 85, 100);
    let mut stupid_ticker = 0;

    let keyboard_sink = carboxyl::Sink::new();

    // Render scale is a signal changed by accumulated keyboard events
    let scale_signal = keyboard_sink.stream().fold(default_scale, |s, keycode| {
        match keycode {
            Keycode::RightBracket => s + Ratio::new(1, 2),
            Keycode::LeftBracket => s - Ratio::new(1, 2),
            _ => s
        }
    });

    // show_gui is a simple boolean signal that switches on pressing 'F'
    let show_gui = keyboard_sink.stream()
        .filter(|k| *k == Keycode::F)
        .fold(false, |t, _| !t );

    'mainloop: loop {
        for event in sdl_context.event_pump().unwrap().poll_iter() {

            // XXX: this is only up here because a few of the event cases
            // below need it, which is a temporary state of affairs.
            let scale = scale_signal.sample();

            match event {
                Event::Quit{..} |
                Event::KeyDown {keycode: Some(Keycode::Escape), ..} => {
                    break 'mainloop
                },
                Event::KeyDown {keycode: Some(code), ..} => {
                    keyboard_sink.send(code);
                    hero.key_down(code);
                },
                Event::KeyUp {..} => {
                    hero.key_up();
                },
                Event::MouseMotion {x, y, ..} => {
                    let x = x as u32;
                    let y = y as u32;
                    if painting {
                        let x = (Ratio::from_integer(x) * scale).to_integer();
                        let y = (Ratio::from_integer(y) * scale).to_integer();
                        map.set_px((x, y), tilepicker.selected()).ok();
                    }
                },
                Event::MouseButtonDown {x, y, ..} => {
                    if !show_gui.sample() || !tilepicker.click((x, y)) {
                        let x = x as u32;
                        let y = y as u32;
                        // XXX: We have to explicitly transform by viewport,
                        // eventually UI should be part of the scene (?)
                        let x = (Ratio::from_integer(x) * scale).to_integer();
                        let y = (Ratio::from_integer(y) * scale).to_integer();
                        map.set_px((x, y), tilepicker.selected()).ok();
                        painting = true;
                    }
                },
                Event::MouseButtonUp {..} => {
                    painting = false;
                },
                Event::MouseWheel {y: scroll_y, ..} => {
                    tilepicker.scroll(scroll_y);
                },
                _ => { }
            }
        }

        let scale = scale_signal.sample();

        // XXX: this is the jankiest possible way to control timestep.
        // Should probably write a proper game loop next.
        let elapsed = time::precise_time_ns() - start;
        let dt = elapsed - stupid_ticker;
        if stupid_ticker > 16666666 {
            stupid_ticker = 0;
            hero.tick();

            let (screen_w, screen_h) = renderer.window().unwrap().size();

            // For now, base scene offset on hero's position
            let screen_w = (Ratio::from_integer(screen_w) / scale).to_integer();
            let screen_h = (Ratio::from_integer(screen_h) / scale).to_integer();
            off_x = hero.x() - (screen_w/2) as i32 + 8;
            off_y = hero.y() - (screen_h/2) as i32 + 12;
        } else {
            stupid_ticker += dt;
        }

        // XXX: note that box must be rendered before creating scene, since
        // scene borrows references to all the instructions added to it. This
        // is perhaps an API weakness; might end up just boxing visibles.
        let rendered_box = textbox.render();
        let rendered_map = map.render();
        let rendered_hero = hero.render();

        renderer.set_draw_color(Color::RGBA(176, 208, 184, 255));
        renderer.clear();

        // XXX: try to use rationals everywhere!
        let scale = *scale.numer() as f32 / *scale.denom() as f32;

        {
            let mut world = Scene::new();
            world.add_all(&rendered_map, -1);
            world.add(&rendered_hero, 0);
            world.add(&starman, 0);
            world.present(&mut renderer, &mut render_context,
                          (off_x, off_y), (scale, scale));
        }

        {
            let mut hud = Scene::new();
            hud.add_all(&rendered_box, 1);
            hud.add(&hello, 2);
            hud.present_scaled(
                &mut renderer, &mut render_context, (scale, scale));
        }

        if show_gui.sample() {
            // This rendering bit is kind of "all wires exposed"; once we
            // figure out a more managed structure for getting Visibles from
            // widget to Scene, this will all look much nicer.
            let (rects, tiles) = tilepicker.render();
            let mut gui = Scene::new();
            gui.add_all(&rects, 0);
            gui.add_all(&tiles, 1);
            gui.present(&mut renderer, &mut render_context, (0, 0), (1.0, 1.0));
        }

        renderer.present();

        frames += 1;
    }

    let end = time::precise_time_ns();
    let fps = (frames as f64 / ((end - start) as f64 / 1e9)) as u32;
    println!("Rendered {} frames in {} ns; effective: {} fps",
             frames, end - start, fps);

    map.save("assets/map.json").unwrap();
}
