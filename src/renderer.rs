// Some draw operations for a renderer
// TODO rename

extern crate sdl2;
extern crate sdl2_image;

use std::path::Path;
use self::sdl2::render::Renderer;
use self::sdl2::rect::Rect;
use self::sdl2_image::LoadTexture;

/// A `DrawOp` is the abstraction of a single, atomic operation in the process
/// of drawing a scene. "Atomic" here means that a DrawOp has a single, defined
/// position in the scene's z-order, and is thus is either completely in front
/// of or completely behind any other DrawOp. Rendering one frame is then the
/// process of creating a set of DrawOps, sorting them, and executing them.
pub trait DrawOp {
    fn draw(&self, renderer: &mut Renderer);
}

/// A simple DrawOp that just renders a single texture. This implementation is
/// especially horrible, since it loads the texture from disk every time it
/// is drawn. (TODO: add texture cache to the DrawOp trait?)
pub struct DrawSprite {
    name: String,
    pos: (i32, i32),
}

impl DrawSprite {
    pub fn new(name: &str, pos: (i32, i32)) -> DrawSprite {
        DrawSprite {
            name: name.into(),
            pos: pos
        }
    }
}

impl DrawOp for DrawSprite {
    fn draw(&self, renderer: &mut Renderer) {
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
