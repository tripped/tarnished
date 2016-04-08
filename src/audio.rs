use snes_spc::SnesSpc;
use sdl2::audio::AudioCallback;

pub struct SpcPlayer {
    emulator: SnesSpc
}

impl SpcPlayer {
    pub fn new(track: &str) -> SpcPlayer {
        SpcPlayer {
            emulator: SnesSpc::from_file(track).unwrap()
        }
    }
}

impl AudioCallback for SpcPlayer {
    type Channel = i16;
    fn callback(&mut self, out: &mut [i16]) {
        self.emulator.play(out).unwrap();
    }
}

/// Manages a set of channels
pub struct Mixer<S> {
    channels: Vec<Box<S>>,
}

impl<S: AudioCallback<Channel = i16>> Mixer<S> {
    pub fn new() -> Mixer<S> {
        Mixer {
            channels: Vec::new(),
        }
    }

    pub fn play(&mut self, source: S) {
        self.channels.push(Box::new(source));
    }
}

impl<S: AudioCallback<Channel = i16>> AudioCallback for Mixer<S> {
    type Channel = i16;

    // XXX: assumes one channel
    fn callback(&mut self, out: &mut [i16]) {
        if self.channels.len() > 0 {
            self.channels[0].callback(out);
        }
    }
}
