use scene::Tile;
use sdl2::keyboard::Keycode;

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
    direction: Direction,
    time: u32,
    step: u32,
    // XXX: really need a better way to represent movement speed at these
    // small discrete pixel scales. It's probably okay to just use floats
    // internally and alias.
    freq: u32,
}

impl Brobot {
    pub fn new(asset: &str, w: u32, h: u32, x: i32, y: i32) -> Brobot {
        Brobot {
            asset: asset.into(),
            w: w, h: h, x: x, y: y,
            state: State::Resting,
            direction: Direction::Down,
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
                self.direction = Direction::Up;
                self.state = State::Walking;
            },
            Keycode::Down => {
                self.direction = Direction::Down;
                self.state = State::Walking;
            },
            Keycode::Left => {
                self.direction = Direction::Left;
                self.state = State::Walking;
            },
            Keycode::Right => {
                self.direction = Direction::Right;
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
            State::Walking => match self.direction {
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
        let mut frame = match self.direction {
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
