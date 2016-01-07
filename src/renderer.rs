// Some draw operations for a renderer
// TODO rename this file

extern crate sdl2;
extern crate sdl2_image;

use std::path::Path;
use self::sdl2::render::Renderer;
use self::sdl2::rect::Rect;
use self::sdl2_image::LoadTexture;

/// A `Visible` object can be shown using a renderer. It is atomic with respect
/// to z-ordering, i.e., it is always entirely behind or entirely in front of
/// other visible objects. Rendering a frame or scene is a process of creating
/// many Visibles, sorting them by layer and z-index, and showing them.
pub trait Visible {
    fn show(&self, renderer: &mut Renderer);
}

/// A Visible object that consists of a single texture. This implementation is
/// especially horrible, since it loads the texture from disk every time it
/// is shown. (TODO: add texture cache to the Visible trait?)
pub struct Sprite {
    name: String,
    pos: (i32, i32),
}

impl Sprite {
    pub fn new(name: &str, pos: (i32, i32)) -> Sprite {
        Sprite {
            name: name.into(),
            pos: pos
        }
    }
}

impl Visible for Sprite {
    fn show(&self, renderer: &mut Renderer) {
        // XXX: loading the texture here is stupid, of course
        let path = self.name.clone() + ".png";
        let path = Path::new(&path);
        let tex = renderer.load_texture(path).unwrap();

        let query = tex.query();
        let (x, y) = self.pos;
        let dst = Rect::new_unwrap(x, y, query.width, query.height);
        renderer.copy(&tex, None, Some(dst));
    }
}
