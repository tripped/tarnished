use carboxyl::Signal;

/// An object with nonvarying position
pub fn static_position(x: f32, y: f32) -> Signal<(f32, f32)> {
    Signal::new((x, y))
}
