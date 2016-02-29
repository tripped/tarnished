extern crate sdl2;
extern crate sdl2_ttf;
extern crate snes_spc;
extern crate time;
extern crate rustc_serialize;
extern crate bincode;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use snes_spc::SnesSpc;

mod renderer;
mod scene;
mod textbox;
mod tilepicker;
mod map;
use scene::{Scene, Tile, sprite, text};
use renderer::{RenderContext, HPos, VPos};
use textbox::Textbox;
use tilepicker::TilePicker;
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

enum Direction {
    Down,
    Left,
    Up,
    Right,
}

enum State {
    Walking,
    Resting,
}

struct Brobot {
    asset: String,
    w: u32,
    h: u32,
    x: i32,
    y: i32,
    state: State,
    direction: Direction,
    time: u32,
    step: u32,
    // XXX: really need a better way to represent movement speed at these
    // small discrete pixel scales. It's probably okay to just use floats
    // internally and alias.
    freq: u32,
}

impl Brobot {
    fn new(asset: &str, w: u32, h: u32) -> Brobot {
        Brobot {
            asset: asset.into(),
            w: w, h: h,
            x: 0, y: 0,
            state: State::Resting,
            direction: Direction::Down,
            time: 0,
            step: 30,
            freq: 2
        }
    }

    fn tick(&mut self) {
        self.time += 1;

        if self.time % self.freq != 0 {
            return;
        }

        match self.state {
            State::Walking => match self.direction {
                Direction::Left => {
                    self.x -= 1;
                },
                Direction::Right => {
                    self.x += 1;
                },
                Direction::Up => {
                    self.y -= 1;
                },
                Direction::Down => {
                    self.y += 1;
                }
            },
            _ => {}
        }
    }

    fn render(&self) -> Tile {
        let mut frame = match self.direction {
            Direction::Down => 0,
            Direction::Left => 1,
            Direction::Up => 2,
            Direction::Right => 3
        };

        // If walking, use the step-up frame every so often
        // XXX: should probably be part of tick(), or take freq into account
        match self.state {
            State::Walking => if (self.time / self.step) % 2 == 0 {
                // XXX: hardcoded frame offsets = gross
                frame += 4;
            },
            _ => {}
        }

        Tile::new(&self.asset, frame, self.w, self.h, self.x, self.y)
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();
    let ttf = sdl2_ttf::init().unwrap();

    let window = video.window("Tarnished", 960, 600)
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let mut renderer = window.renderer()
        .accelerated()
        .build()
        .unwrap();

    let mut render_context = RenderContext::new(ttf);

    // The default scaling factors we'll apply when rendering
    let mut scale_x = 4.0;
    let mut scale_y = 4.0;

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
    let tilepicker = TilePicker::new("assets/cotp", 16, 16, 0, 0, 960, 66);
    let mut show_gui = false;

    let mut frames = 0u32;
    let start = time::precise_time_ns();

    let mut hero = Brobot::new("assets/hero", 16, 24);
    hero.x = 85;
    hero.y = 100;
    let mut stupid_ticker = 0;

    'mainloop: loop {
        for event in sdl_context.event_pump().unwrap().poll_iter() {
            match event {
                Event::Quit{..} |
                Event::KeyDown {keycode: Some(Keycode::Escape), ..} => {
                    break 'mainloop
                },
                Event::KeyDown {keycode: Some(Keycode::F), ..} => {
                    show_gui = !show_gui;
                },
                Event::KeyDown {keycode: Some(Keycode::RightBracket), ..} => {
                    scale_x += 0.2;
                    scale_y += 0.2;
                },
                Event::KeyDown {keycode: Some(Keycode::LeftBracket), ..} => {
                    scale_x -= 0.2;
                    scale_y -= 0.2;
                },
                Event::KeyDown {keycode: Some(Keycode::Up), ..} => {
                    hero.direction = Direction::Up;
                    hero.state = State::Walking;
                },
                Event::KeyDown {keycode: Some(Keycode::Down), ..} => {
                    hero.direction = Direction::Down;
                    hero.state = State::Walking;
                },
                Event::KeyDown {keycode: Some(Keycode::Left), ..} => {
                    hero.direction = Direction::Left;
                    hero.state = State::Walking;
                },
                Event::KeyDown {keycode: Some(Keycode::Right), ..} => {
                    hero.direction = Direction::Right;
                    hero.state = State::Walking;
                },
                Event::KeyUp {..} => {
                    hero.state = State::Resting;
                },
                Event::MouseWheel {y: scroll_y, ..} => {
                    let (_, x, y) = sdl_context.mouse().mouse_state();

                    // XXX: We have to explicitly transform by viewport here,
                    // eventually UI should be part of the scene (?)
                    let x = (x / 4 - off_x) as u32;
                    let y = (y / 4 - off_y) as u32;

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

        // XXX: this is the jankiest possible way to control timestep.
        // Should probably write a proper game loop next.
        let elapsed = time::precise_time_ns() - start;
        let dt = elapsed - stupid_ticker;
        if stupid_ticker > 16666666 {
            stupid_ticker = 0;
            hero.tick();

            // For now, base scene offset on hero's position
            off_x = -hero.x + 110;
            off_y = -hero.y + 60;
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

        {
            let mut world = Scene::new();
            world.add_all(&rendered_map, -1);
            world.add(&rendered_hero, 0);
            world.add(&starman, 0);
            world.present(&mut renderer, &mut render_context,
                          (off_x, off_y), (scale_x, scale_y));
        }

        {
            let mut hud = Scene::new();
            hud.add_all(&rendered_box, 1);
            hud.add(&hello, 2);
            hud.present_scaled(
                &mut renderer, &mut render_context, (scale_x, scale_y));
        }

        if (show_gui) {
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
