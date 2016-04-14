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

/// Convert a lower-frequency sample buffer into a higher-frequency sample
/// buffer. NOTE: currently broken.
fn upsample(source: &[i16], target_samples: usize, x:i16)
        -> (Vec<i16>, i16) {
    assert!(target_samples > source.len());

    // Upsample into target buffer with zero-stuffing
    let mut stuffed = vec![0i16; target_samples];
    for i in 0..source.len() {
        stuffed[i * target_samples / source.len()] = source[i];
    }

    // Low-pass this batch of samples
    let mut filtered = vec![0i16; stuffed.len()];
    filtered[0] = stuffed[0] + x;
    for i in 1..stuffed.len() {
        filtered[i] = stuffed[i] + stuffed[i-1]
    }
    let lp = stuffed[target_samples-1];

    (filtered, lp)
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
            // 32KHz sample buffer, before upsampling
            let srclen = (out.len() * 32000) / 44100;
            let mut buffer = vec![016;srclen];
            channel.callback(buffer.as_mut_slice());

            let (result, lp) = upsample(&buffer, out.len(), self.lp[n]);
            self.lp[n] = lp;

            // Blend channel into output
            for i in 0..out.len() {
                out[i] += result[i] / num;
            }
        }
    }
}
