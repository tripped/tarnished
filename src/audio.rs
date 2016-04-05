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
