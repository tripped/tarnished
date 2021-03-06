#[macro_use(lift)]
extern crate carboxyl;
extern crate conv;
extern crate num;
extern crate rustc_serialize;
extern crate sdl2;
extern crate sdl2_image;
extern crate sdl2_ttf;
extern crate snes_spc;
extern crate time;

use carboxyl::Sink;
use sdl2::audio::AudioSpecDesired;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::cmp::{min, max};

mod audio;
mod brobot;
mod event;
mod map;
mod physics;
mod ratio;
mod renderer;
mod scene;
mod textbox;
mod tilepicker;

use audio::{SpcPlayer, Mixer};
use brobot::controlled_sprite;
use event::{IOEvent, translate_event};
use map::MapLayer;
use ratio::{Ratio, Scalable};
use renderer::{RenderContext, HPos, VPos};
use scene::{Scene, sprite, text};
use textbox::Textbox;
use tilepicker::TilePicker;

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
        //.present_vsync()
        .build()
        .unwrap();

    let mut render_context = RenderContext::new(ttf);

    // The default scaling factor we'll apply when rendering
    let default_scale = Ratio::from_integer(4u32);

    // Start making noise
    let audio_subsystem = sdl_context.audio().unwrap();
    let desired_spec = AudioSpecDesired {
        freq: Some(64000),
        channels: Some(2),
        samples: None
    };

    let audio = audio_subsystem.open_playback(None, &desired_spec, |spec| {
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
        Rect::new(12, 12, 32, 16));
    let hello = text("$0.00", "assets/orangekid", 30, 18);

    let mut map = MapLayer::from_file("assets/map.json")
        .unwrap_or(MapLayer::new("assets/cotp", (16, 16), 25, vec![0;25*16]));

    // XXX: note that this widget is rendered in unscaled space, so its width
    // of 960 is actually the full window width. Soon these different spaces
    // should be managed more cleanly.
    let mut tilepicker = TilePicker::new("assets/cotp", 16, 16, 0, 0, 960, 66);
    let mut painting = false;

    // The one sink for all SDL events.
    let sdl_sink = Sink::new();

    // A Stream consisting of all key-up and key-down events
    let keyboard_stream = sdl_sink.stream().filter(|event| {
        match *event {
            IOEvent::KeyDown(_) | IOEvent::KeyUp(_) => true,
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

    // A Stream consisting of just key-down events
    // XXX: temporary, just used by scale and show_gui signals
    let keydown_stream = keyboard_stream.filter_map(|event| {
        match event {
            IOEvent::KeyDown(keycode) => Some(keycode),
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

    // Game loop control
    let mut curtime = time::precise_time_ns();
    let mut accumulator = 0u64;
    let dt = 16666667;

    // Metrics
    let mut logic_time = 0u64;
    let mut logic_time_max = 0u64;
    let mut render_time = 0u64;
    let mut render_time_max = 0u64;
    let mut present_time = 0u64;
    let mut present_time_max = 0u64;
    let mut frames = 0u64;
    let start = time::precise_time_ns();

    'mainloop: loop {
        let logic_start = time::precise_time_ns();

        // XXX: We have to explicitly transform by viewport,
        // eventually UI should be part of the scene (?)
        let scale = scale_signal.sample();
        let (screen_x, screen_y) = screen_pos.sample();

        // Add rendering delta to accumulator
        // XXX: need to clean this up and factor out rendering/integration
        // code; would make this loop much prettier.
        let newtime = time::precise_time_ns();
        let frametime = newtime - curtime;
        curtime = newtime;
        accumulator += frametime;

        while accumulator >= dt {

            let transform_to_world = |x: i32, y: i32| {
                let x = x.scale(scale.recip()) + screen_x;
                let y = y.scale(scale.recip()) + screen_y;
                (x, y)
            };

            for event in sdl_context.event_pump().unwrap().poll_iter() {
                match event {
                    Event::Quit{..} |
                    Event::KeyDown {keycode: Some(Keycode::Escape), ..} => {
                        break 'mainloop
                    },
                    Event::MouseMotion {x, y, ..} => {
                        if painting {
                            let (x, y) = transform_to_world(x, y);
                            map.set_px((x, y), tilepicker.selected()).ok();
                        }
                    },
                    Event::MouseButtonDown {x, y, ..} => {
                        if !show_gui.sample() || !tilepicker.click((x, y)) {
                            let (x, y) = transform_to_world(x, y);
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

                translate_event(event).map(|e| sdl_sink.send(e));
            }

            accumulator -= dt;
            delta_sink.send((dt as f32) / 1e9);
        }

        // Count time spent updating the reactive network
        {
            let this_frame = time::precise_time_ns() - logic_start;
            logic_time_max = max(this_frame, logic_time_max);
            logic_time += this_frame;
        }

        let render_start = time::precise_time_ns();

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

        // Count time spent rendering the frame
        {
            let this_frame = time::precise_time_ns() - render_start;
            render_time_max = max(this_frame, render_time_max);
            render_time += this_frame;
        }

        let present_start = time::precise_time_ns();

        // XXX: this takes <1ms even with present_vsync enabled; instead the
        // rendering step absorbs the synchronization latency? Figure out why.
        renderer.present();

        // Count time spent presenting to the SDL renderer
        {
            let this_frame = time::precise_time_ns() - present_start;
            present_time_max = max(this_frame, present_time_max);
            present_time += this_frame;
        }

        frames += 1;
    }

    let end = time::precise_time_ns();
    let fps = (frames as f64 / ((end - start) as f64 / 1e9)) as u32;
    println!("Performance summary:");
    println!("Rendered {} frames in {} ns; effective: {} fps",
             frames, end - start, fps);
    println!("Logic update 𝚫t:\t\tmean: {:.*} ms\tmax: {:.*} ms",
             2, logic_time as f64 / frames as f64 / 1e6,
             2, logic_time_max as f64 / 1e6);
    println!("Render 𝚫t:\t\t\tmean: {:.*} ms\tmax: {:.*} ms",
             2, render_time as f64 / frames as f64 / 1e6,
             2, render_time_max as f64 / 1e6);
    println!("Present 𝚫t:\t\t\tmean: {:.*} ms\tmax: {:.*} ms",
             2, present_time as f64 / frames as f64 / 1e6,
             2, present_time_max as f64 / 1e6);

    map.save("assets/map.json").unwrap();
}
