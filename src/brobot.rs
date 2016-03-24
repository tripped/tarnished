use scene::Tile;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use carboxyl;

#[derive(Copy, Clone)]
pub enum Direction {
    Down,
    Left,
    Up,
    Right,
}

pub enum State {
    Walking,
    Resting,
}

pub struct Brobot {
    asset: String,
    w: u32,
    h: u32,
    x: i32,
    y: i32,
    state: State,
    direction: carboxyl::Signal<Direction>,
    time: u32,
    step: u32,
    // XXX: really need a better way to represent movement speed at these
    // small discrete pixel scales. It's probably okay to just use floats
    // internally and alias.
    freq: u32,
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
fn samsara(impulse: Impulse, event: Event) -> Impulse {
    // XXX: ew.
    match event {
        Event::KeyDown {keycode: Some(Keycode::Left), ..} =>
            Impulse {left: true, ..impulse},
        Event::KeyDown {keycode: Some(Keycode::Up), ..} =>
            Impulse {up: true, ..impulse},
        Event::KeyDown {keycode: Some(Keycode::Right), ..} =>
            Impulse {right: true, ..impulse},
        Event::KeyDown {keycode: Some(Keycode::Down), ..} =>
            Impulse {down: true, ..impulse},
        Event::KeyUp {keycode: Some(Keycode::Left), ..} =>
            Impulse {left: false, ..impulse},
        Event::KeyUp {keycode: Some(Keycode::Up), ..} =>
            Impulse {up: false, ..impulse},
        Event::KeyUp {keycode: Some(Keycode::Right), ..} =>
            Impulse {right: false, ..impulse},
        Event::KeyUp {keycode: Some(Keycode::Down), ..} =>
            Impulse {down: false, ..impulse},
        _ => impulse
    }
}

impl Brobot {
    pub fn new(asset: &str, w: u32, h: u32, x: i32, y: i32,
               keyboard: carboxyl::Stream<Event>) -> Brobot {
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
                Impulse { left: true, .. } => Direction::Left,
                Impulse { right: true, .. } => Direction::Right,
                Impulse { up: true, .. } => Direction::Up,
                _ => Direction::Down  // XXX: should have Direction::Mu
            }
        }
        let direction = lift!(decisiveness, &whimsy, &impulse);

        Brobot {
            asset: asset.into(),
            w: w, h: h, x: x, y: y,
            state: State::Resting,
            direction: direction,
            time: 0,
            step: 30,
            freq: 2
        }
    }

    pub fn x(&self) -> i32 { self.x }
    pub fn y(&self) -> i32 { self.y }

    // XXX: just wiring through the keyboard event is kinda crap. This should
    // be using FRP! This is just a temporary measure, I promise.
    pub fn key_down(&mut self, code: Keycode) {
        match code {
            Keycode::Up => {
                self.state = State::Walking;
            },
            Keycode::Down => {
                self.state = State::Walking;
            },
            Keycode::Left => {
                self.state = State::Walking;
            },
            Keycode::Right => {
                self.state = State::Walking;
            },
            _ => {}
        }
    }

    pub fn key_up(&mut self) {
        self.state = State::Resting;
    }

    pub fn tick(&mut self) {
        self.time += 1;

        if self.time % self.freq != 0 {
            return;
        }

        match self.state {
            State::Walking => match self.direction.sample() {
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

    pub fn render(&self) -> Tile {
        let mut frame = match self.direction.sample() {
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
