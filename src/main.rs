extern crate sdl2;
extern crate snes_spc;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::audio::{AudioCallback, AudioSpecDesired};
use snes_spc::SnesSpc;

mod scene;
use scene::{Visible, Sprite};

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

    let window = video.window("Tarnished", 800, 500)
        .position_centered()
        .build()
        .unwrap();

    let mut renderer = window.renderer()
        .accelerated()
        .build()
        .unwrap();

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
    let starman = Sprite::new("assets/starmanjr_lg", (300, 100));

    'mainloop: loop {
        for event in sdl_context.event_pump().unwrap().poll_iter() {
            match event {
                Event::Quit{..} |
                Event::KeyDown {keycode: Some(Keycode::Escape), ..} => {
                    break 'mainloop
                },
                _ => { }
            }
        }
        starman.show(&mut renderer);
        renderer.present();
    }
}
