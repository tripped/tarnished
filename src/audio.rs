use sdl2::audio::AudioCallback;
use snes_spc::SnesSpc;

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

/// Convert a lower-frequency sample buffer into a higher-frequency buffer.
/// XXX: this is broken in several ways. First, it ignores the fact that our
/// samples are actually interleaved LRLR stereo. Both upsampling and the
/// low-pass filter will naturally butcher the audio when applied naively.
/// We also need to account for temporal aliasing when zero-stuffing; starting
/// at 0 for every batch of samples is incorrect, we must account for the
/// integer division error from the previous buffer.
fn upsample(source: &[i16], target_samples: usize, x:i16)
        -> (Vec<i16>, i16) {
    assert!(target_samples > source.len());

    // Upsample into target buffer with zero-stuffing
    let mut stuffed = vec![0i16; target_samples];
    for i in 0..source.len() {
        stuffed[i * target_samples / source.len()] = source[i];
    }

    // Low-pass this batch of samples
    /*
    let mut filtered = vec![0i16; stuffed.len()];
    filtered[0] = stuffed[0] + x;
    for i in 1..stuffed.len() {
        filtered[i] = stuffed[i] + stuffed[i-1]
    }
    let lp = stuffed[target_samples-1];
    */

    (stuffed, x)
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
            let srclen = (out.len() * 32000) / 64000;
            let mut buffer = vec![0i16;srclen];
            channel.callback(buffer.as_mut_slice());

            // Split buffer into left and right channels
            assert!(srclen % 2 == 0);
            let mut left = vec![0i16;srclen/2];
            let mut right = vec![0i16;srclen/2];
            for i in 0..srclen {
                if i % 2 == 0 {
                    left[i/2] = buffer[i];
                } else {
                    right[i/2] = buffer[i];
                }
            }

            // Upsample the channels independently
            // XXX: channels need independent upsample spillover
            assert!(out.len() % 2 == 0);
            let (left, lp) = upsample(&left, out.len()/2, self.lp[n]);
            let (right, lp) = upsample(&right, out.len()/2, lp);
            self.lp[n] = lp;

            // Blend channel into output
            for i in 0..out.len() {
                if i % 2 == 0 {
                    out[i] += left[i/2] / num;
                } else {
                    out[i] += right[i/2] / num;
                }
            }
        }
    }
}
