use carboxyl::Signal;

#[derive(Clone, Copy)]
pub struct Momentum {
    direction: f32,
    magnitude: f32,
}

/// An object with nonvarying position
pub fn static_position(x: f32, y: f32)
        -> (Signal<(f32, f32)>, Signal<Momentum>) {
    (Signal::new((x, y)), Signal::new(Momentum {
        direction: 0.0, magnitude: 0.0,
    }))
}
