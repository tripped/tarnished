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
    fn show(&self, offset: (i32, i32), renderer: &mut Renderer);
}

/// A Scene is a places where Visibles may be shown.
pub struct Scene<'a> {
    renderer: &'a mut Renderer<'static>,
    elements: Vec<&'a Visible>,
    // TODO: should probably specify full viewport rectangle
    offset: (i32, i32),
}

impl<'a> Scene<'a> {
    pub fn new(renderer: &'a mut Renderer<'static>) -> Scene<'a> {
        Scene {
            renderer: renderer,
            elements: Vec::new(),
            offset: (0, 0),
        }
    }

    pub fn set_viewport(&mut self, offset: (i32, i32)) {
        self.offset = offset;
    }

    pub fn add(&mut self, element: &'a Visible) {
        self.elements.push(element);
    }

    pub fn present(&mut self) {
        self.renderer.clear();

        for element in self.elements.iter() {
            element.show(self.offset, self.renderer);
        }

        self.renderer.present();
    }
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
    fn show(&self, (offx, offy): (i32, i32), renderer: &mut Renderer) {
        // XXX: loading the texture here is stupid, of course
        let path = self.name.clone() + ".png";
        let path = Path::new(&path);
        let tex = renderer.load_texture(path).unwrap();

        let query = tex.query();
        let (x, y) = self.pos;
        let dst = Rect::new_unwrap(x+offx, y+offy, query.width, query.height);
        renderer.copy(&tex, None, Some(dst));
    }
}
