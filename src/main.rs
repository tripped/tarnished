extern crate sdl2;
extern crate sdl2_image;
extern crate sdl2_ttf;
extern crate snes_spc;
extern crate time;
extern crate rustc_serialize;
#[macro_use(lift)]
extern crate carboxyl;
extern crate num;

use std::cmp::{min, max};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::audio::AudioSpecDesired;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use num::rational::Ratio;
use carboxyl::Sink;

mod renderer;
mod scene;
mod textbox;
mod tilepicker;
mod map;
mod brobot;
mod audio;
mod physics;

use scene::{Scene, sprite, text};
use renderer::{RenderContext, HPos, VPos};
use textbox::Textbox;
use tilepicker::TilePicker;
use map::MapLayer;
use brobot::controlled_sprite;
use audio::{SpcPlayer, Mixer};

use num::integer::Integer;

// XXX: should be able to scale signed int by either signed or unsigned ratio
trait RatioScalable: Clone + Integer {
    fn scale(&self, scale: Ratio<Self>) -> Self {
        (Ratio::from_integer(self.clone()) * scale).to_integer()
    }
}
impl RatioScalable for u32 {}


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
        freq: Some(44100),
        channels: Some(2),
        samples: None
    };

    let audio = audio_subsystem.open_playback(None, desired_spec, |spec| {
        println!("Audio initialized: {:?}", spec);
        let mut mixer = Mixer::new();
        mixer.play(SpcPlayer::new("assets/FireSpring.spc"));
        mixer
    }).unwrap();

    audio.resume();

    // Draw some stuff
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

    // The one sink for all SDL events.
    let sdl_sink = Sink::new();

    // A Stream consisting of all key-up and key-down events
    let keyboard_stream = sdl_sink.stream().filter(|event| {
        match *event {
            Event::KeyDown {..} | Event::KeyUp {..} => true,
            _ => false
        }
    });

    // Shove time deltas in here...
    let delta_sink = Sink::new();

    // ...the current time comes out here.
    let time = delta_sink.stream().fold(0.0, |a, b| a + b);

    let (hero_pos, hero_display) = controlled_sprite(
        "assets/porky", 16, 24, 85, 100,
        keyboard_stream.clone(), time.clone(), delta_sink.stream());
    let mut stupid_ticker = 0;

    // A Stream consisting of just key-down events
    // XXX: temporary, just used by scale and show_gui signals
    let keydown_stream = keyboard_stream.filter_map(|event| {
        match event {
            Event::KeyDown {keycode, ..} => keycode,
            _ => None
        }
    });

    // Render scale is a signal changed by accumulated keyboard events
    let scale_signal = keydown_stream.fold(default_scale, |s, keycode| {
        let min_scale = Ratio::new(1, 2);
        let joe_factor = Ratio::from_integer(8);
        match keycode {
            Keycode::RightBracket => min(joe_factor, s + Ratio::new(1, 2)),
            Keycode::LeftBracket => max(min_scale, s - Ratio::new(1, 2)),
            _ => s
        }
    });

    // Screen position is determined by hero position and scale
    // XXX: (Also by screen size, but we'll move this to a signal as well later
    // XXX: This should be broken up more; scaling and screen size both
    // should probably be introduced in a separate lift.
    let (screen_w, screen_h) = renderer.window().unwrap().size();
    let screen_pos = lift!(move |scale, (hero_x, hero_y)| {
        let screen_w = (Ratio::from_integer(screen_w) / scale).to_integer();
        let screen_h = (Ratio::from_integer(screen_h) / scale).to_integer();
        (hero_x as i32 - (screen_w/2) as i32 + 8,
         hero_y as i32 - (screen_h/2) as i32 + 12)
    }, &scale_signal, &hero_pos);

    // show_gui is a simple boolean signal that switches on pressing 'F'
    let show_gui = keydown_stream
        .filter(|k| *k == Keycode::F)
        .fold(false, |t, _| !t );

    'mainloop: loop {
        // XXX: this is only up here because a few of the event cases
        // below need it, which is a temporary state of affairs.
        let scale = scale_signal.sample();

        for event in sdl_context.event_pump().unwrap().poll_iter() {
            match event {
                Event::Quit{..} |
                Event::KeyDown {keycode: Some(Keycode::Escape), ..} => {
                    break 'mainloop
                },
                Event::MouseMotion {x, y, ..} => {
                    if painting {
                        let (screen_x, screen_y) = screen_pos.sample();
                        let x = ((x as u32).scale(scale.recip())
                                 as i32 + screen_x) as u32;
                        let y = ((y as u32).scale(scale.recip())
                                 as i32 + screen_y) as u32;
                        map.set_px((x, y), tilepicker.selected()).ok();
                    }
                },
                Event::MouseButtonDown {x, y, ..} => {
                    if !show_gui.sample() || !tilepicker.click((x, y)) {
                        // XXX: We have to explicitly transform by viewport,
                        // eventually UI should be part of the scene (?)
                        // XXX: oh my god really
                        let (screen_x, screen_y) = screen_pos.sample();
                        let x = ((x as u32).scale(scale.recip())
                                 as i32 + screen_x) as u32;
                        let y = ((y as u32).scale(scale.recip())
                                 as i32 + screen_y) as u32;

                        // XXX: if set_px takes screen coords, it should be i32
                        // XXX: don't swallow errors with ok!
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

            sdl_sink.send(event);
        }

        // XXX: this is the jankiest possible way to control timestep.
        // Should probably write a proper game loop next.
        let elapsed = time::precise_time_ns() - start;
        let dt = elapsed - stupid_ticker;
        if stupid_ticker > 16666666 {
            stupid_ticker = 0;
            delta_sink.send(1.0/60.0);
        } else {
            stupid_ticker += dt;
        }

        // XXX: note that box must be rendered before creating scene, since
        // scene borrows references to all the instructions added to it. This
        // is perhaps an API weakness; might end up just boxing visibles.
        let rendered_box = textbox.render();
        let rendered_map = map.render();
        let rendered_hero = hero_display.sample();

        renderer.set_draw_color(Color::RGBA(176, 208, 184, 255));
        renderer.clear();

        {
            let mut world = Scene::new();
            world.add_all(&rendered_map, -1);
            world.add(&rendered_hero, 0);
            world.add(&starman, 0);
            world.present(&mut renderer, &mut render_context,
                          screen_pos.sample(), scale);
        }

        {
            let mut hud = Scene::new();
            hud.add_all(&rendered_box, 1);
            hud.add(&hello, 2);
            hud.present_scaled(&mut renderer, &mut render_context, scale);
        }

        if show_gui.sample() {
            // This rendering bit is kind of "all wires exposed"; once we
            // figure out a more managed structure for getting Visibles from
            // widget to Scene, this will all look much nicer.
            let (rects, tiles) = tilepicker.render();
            let mut gui = Scene::new();
            gui.add_all(&rects, 0);
            gui.add_all(&tiles, 1);
            gui.present(&mut renderer, &mut render_context, (0, 0),
                Ratio::from_integer(1));
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
