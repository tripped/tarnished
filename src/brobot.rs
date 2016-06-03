use scene::Tile;
use sdl2::keyboard::Keycode;
use carboxyl::{Signal, Stream};
use event::IOEvent;

#[derive(Copy, Clone)]
pub enum Direction {
    Down,
    Left,
    Up,
    Right,
}

/// A radically free struct that is capable of representing the desire to move
/// in every direction at once.
#[derive(Copy, Clone, Eq, PartialEq)]
struct Impulse {
    left: bool,
    up: bool,
    right: bool,
    down: bool
}

impl Impulse {
    /// The Impulse that desires nothing.
    fn nirvana() -> Impulse {
        Impulse {
            left: false,
            up: false,
            right: false,
            down: false
        }
    }
}

/// The eternal cycle of changing desires based on SDL keyboard events
fn samsara(impulse: Impulse, event: IOEvent) -> Impulse {
    // XXX: ew.
    match event {
        IOEvent::KeyDown(Keycode::Left) =>
            Impulse {left: true, ..impulse},
        IOEvent::KeyDown(Keycode::Up) =>
            Impulse {up: true, ..impulse},
        IOEvent::KeyDown(Keycode::Right) =>
            Impulse {right: true, ..impulse},
        IOEvent::KeyDown(Keycode::Down) =>
            Impulse {down: true, ..impulse},
        IOEvent::KeyUp(Keycode::Left) =>
            Impulse {left: false, ..impulse},
        IOEvent::KeyUp(Keycode::Up) =>
            Impulse {up: false, ..impulse},
        IOEvent::KeyUp(Keycode::Right) =>
            Impulse {right: false, ..impulse},
        IOEvent::KeyUp(Keycode::Down) =>
            Impulse {down: false, ..impulse},
        _ => impulse
    }
}

/// Create a visible sprite that moves according to player input. The output
/// is a pair of signals, one containing position, and one containing the
/// visible presentation of the sprite.
pub fn controlled_sprite(asset: &str, w: u32, h: u32, x: i32, y: i32,
            keyboard: Stream<IOEvent>,
            time: Signal<f32>,
            time_delta: Stream<f32>) -> (Signal<(f32, f32)>, Signal<Tile>) {
    // First, transform keyboard events into a time-varying impulse signal
    let impulse = keyboard.fold(Impulse::nirvana(), samsara);

    // In addition to where we want to move, we must compute our facing
    // direction. This is a function not just of current impulse, but
    // also of previous impulse when impulse is zero. Thus, we will
    // generate another signal which holds the prior impulse. We call
    // this quantity "whimsy."
    let whimsy = impulse.snapshot(&keyboard, |i, _| i)
        .hold(Impulse::nirvana());

    // Finally, facing direction is computed by lifting an introspective
    // function over whimsy and impulse.
    fn decisiveness(a: Impulse, b: Impulse) -> Direction {
        let stronger = if b == Impulse::nirvana() { a } else { b };
        match stronger {
            Impulse { down: true, .. } => Direction::Down,
            Impulse { up: true, .. } => Direction::Up,
            Impulse { left: true, .. } => Direction::Left,
            Impulse { right: true, .. } => Direction::Right,
            _ => Direction::Down  // XXX: should have Direction::Mu
        }
    }
    let direction = lift!(decisiveness, &whimsy, &impulse);

    // Now, actual position can be represented as a cyclic signal
    // folding impulse over time.
    // XXX: the fold function samples the impulse signal. That's obviously
    // rather impure! Perhaps this should be a fold over a combined stream
    // containing keyboard and time signals..?
    let initial_position = (x as f32, y as f32);
    let speed = 120.0;

    let position = {
        let impulse = impulse.clone();
        time_delta.fold(initial_position, move |pos, dt| {
            let impulse = impulse.sample();
            let (mut x, mut y) = pos;
            if impulse.left {
                x -= dt * speed;
            }
            if impulse.right {
                x += dt * speed;
            }
            if impulse.up {
                y -= dt * speed;
            }
            if impulse.down {
                y += dt * speed;
            }
            (x, y)
        })
    };

    // Finally, we can describe presentation in terms of time, impulse,
    // direction, and position.
    let render = {
        // XXX: figure out the units here, this is clearly not steps per sec
        let steps_per_second = 16.0;
        let asset = asset.to_string();
        lift!(move |time, impulse, direction, position| {
            let mut frame = match direction {
                Direction::Down => 0,
                Direction::Left => 1,
                Direction::Up => 2,
                Direction::Right => 3
            };

            if impulse != Impulse::nirvana() {
                if (time * steps_per_second).sin() > 0.0 {
                    // XXX: hardcoded frame offsets = gross
                    frame += 4;
                }
            }
            let (x, y) = position;
            Tile::new(&asset, frame, w, h, x as i32, y as i32)
        },
        &time, &impulse, &direction, &position)
    };

    (position, render)
}
