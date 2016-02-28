use scene::{Scene, Rectangle};
use renderer::RenderContext;
use sdl2;
use sdl2::rect::Rect;
use sdl2::pixels::Color;

pub struct TilePicker {
    // widget has total ownership of its position for now
    rect: Rect,
}

impl TilePicker {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> TilePicker {
        TilePicker {
            rect: Rect::new_unwrap(x, y, width, height),
        }
    }

    /// Render the tileset picker.
    /// XXX: this is dirty and wrong. A single widget should render itself as
    /// a set of drawing instructions, not accept a Renderer directly and do
    /// naughty things to it. This is just a temporary hack to get this widget
    /// drawn, since we will need heterogeneous Visibles, and the current API
    /// doesn't make that very pretty.
    pub fn render(&self,
                  renderer: &mut sdl2::render::Renderer<'static>,
                  context: &mut RenderContext) {
        // First, fill the whole space with a sexy dark rectangle
        let bg = Rectangle::filled(self.rect, Color::RGBA(32, 32, 32, 255));
        let mut scene = Scene::new();
        scene.add(&bg, 0);
        scene.present(renderer, context, (0, 0), (1.0, 1.0));
    }
}
