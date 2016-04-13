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
    lp: Vec<i16>,
}

impl<S: AudioCallback<Channel = i16>> Mixer<S> {
    pub fn new() -> Mixer<S> {
        Mixer {
            channels: Vec::new(),
            lp: Vec::new(),
        }
    }

    pub fn play(&mut self, source: S) {
        self.channels.push(Box::new(source));
        self.lp.push(0);
    }
}

impl<S: AudioCallback<Channel = i16>> AudioCallback for Mixer<S> {
    type Channel = i16;

    fn callback(&mut self, out: &mut [i16]) {
        // Zero the output buffer first, since this is not done for us!
        for i in 0..out.len() {
            out[i] = 0;
        }

        let num = self.channels.len() as i16;

        for (n, channel) in self.channels.iter_mut().enumerate() {
            // 32KHz sample buffer, before interpolation
            let srclen = (out.len() * 32000) / 44100;
            let mut buffer = vec![016;srclen];

            let mut interpolated = vec![0i16;out.len()];

            channel.callback(buffer.as_mut_slice());

            // Upsample into 44.1KHz buffer with zero-stuffing
            for i in 0..buffer.len() {
                interpolated[(i * 44100) / 32000] = buffer[i];
            }

            // Low-pass this batch of samples
            let mut filtered = vec![0i16;interpolated.len()];
            filtered[0] = interpolated[0] + self.lp[n];
            for i in 1..interpolated.len() {
                filtered[i] = interpolated[i] + interpolated[i-1]
            }
            self.lp[n] = interpolated[interpolated.len()-1];

            // Blend channel into output
            for i in 0..out.len() {
                out[i] += filtered[i] / num;
            }
        }
    }
}
