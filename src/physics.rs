use carboxyl::Signal;

#[derive(Clone, Copy)]
pub struct Vector2D {
    x: f32,
    y: f32,
}

impl Vector2D {
    pub fn new(x: f32, y: f32) -> Vector2D {
        Vector2D { x: x, y: y }
    }
}

#[derive(Clone, Copy)]
pub struct Momentum(Vector2D);

impl Momentum {
    pub fn new(x: f32, y: f32) -> Momentum {
        Momentum(Vector2D { x: x, y: y })
    }
}

#[derive(Clone, Copy)]
pub struct Position(Vector2D);

impl Position {
    pub fn new(x: f32, y: f32) -> Position {
        Position(Vector2D { x: x, y: y })
    }
}

/// An object with nonvarying position
pub fn static_position(x: f32, y: f32)
        -> (Signal<Position>, Signal<Momentum>) {
    (Signal::new(Position::new(x, y)),
     Signal::new(Momentum::new(0.0, 0.0)))
}
