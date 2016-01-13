extern crate sdl2;
extern crate sdl2_image;

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::path::Path;

use self::sdl2::rect::Rect;
use self::sdl2::render::Texture;
use self::sdl2_image::LoadTexture;

pub struct Renderer {
    renderer: sdl2::render::Renderer<'static>,
    textures: HashMap<String, Texture>,
}

fn load_texture(asset: &str, renderer: &sdl2::render::Renderer) -> Texture {
    // TODO: handle assets more intelligently than just appending ".png"
    // TODO: define a wrapper that properly uses AsRef<Path>, because this
    // conversion is rather inconvenient
    let path = asset.to_string() + ".png";
    let path = Path::new(&path);
    renderer.load_texture(path).unwrap()
}

impl Renderer {
    pub fn new(renderer: sdl2::render::Renderer<'static>) -> Renderer {
        Renderer {
            renderer: renderer,
            textures: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.renderer.clear();
    }

    pub fn present(&mut self) {
        self.renderer.present();
    }

    /// Load a texture if it does not already exist in cache
    ///
    /// XXX: ideally, this function would return a reference to the texture.
    /// However, returning a reference would extend the lifetime of the borrow
    /// on `self` which implicitly borrows `self.renderer`. So, we'd be unable
    /// to draw the texture we just requested. Currently the only way to fix
    /// this would be to redesign this system so that renderer and the texture
    /// cache aren't in the same struct.
    fn ensure_texture(&mut self, asset: &str) {
        self.textures.entry(asset.to_string())
            .or_insert(load_texture(asset, &self.renderer));
    }

    pub fn draw(&mut self, asset: &str, (x, y): (i32, i32)) {
        self.ensure_texture(asset);
        let tex = self.textures.get(&asset.to_string()).unwrap();

        // Default draw is to render at full asset size
        let query = tex.query();
        let dst = Rect::new_unwrap(x, y, query.width, query.height);

        self.renderer.copy(&tex, None, Some(dst));
    }
}

/// A `Visible` object can be shown using a renderer. It is atomic with respect
/// to z-ordering, i.e., it is always entirely behind or entirely in front of
/// other visible objects. Rendering a frame or scene is a process of creating
/// many Visibles, sorting them by layer and z-index, and showing them.
pub trait Visible {
    fn show(&self, offset: (i32, i32), renderer: &mut Renderer);
}

/// A scene Instruction is a Visible and a z-index at which it is to be shown.
struct Instruction<'a> {
    z_index: i32,
    object: &'a Visible,
}

impl<'a> Ord for Instruction<'a> {
    fn cmp(&self, other: &Instruction) -> Ordering {
        self.z_index.cmp(&other.z_index)
    }
}

impl<'a> PartialOrd for Instruction<'a> {
    fn partial_cmp(&self, other: &Instruction) -> Option<Ordering> {
        self.z_index.partial_cmp(&other.z_index)
    }
}

impl<'a> PartialEq for Instruction<'a> {
    fn eq(&self, other: &Instruction) -> bool {
        self.z_index == other.z_index
    }
}

impl<'a> Eq for Instruction<'a> { }

/// A Scene is a place where Visibles may be shown.
pub struct Scene<'a> {
    elements: BinaryHeap<Instruction<'a>>,
    // TODO: should probably specify full viewport rectangle
    offset: (i32, i32),
}

impl<'a> Scene<'a> {
    pub fn new() -> Scene<'a> {
        Scene {
            elements: BinaryHeap::new(),
            offset: (0, 0),
        }
    }

    pub fn set_viewport(&mut self, offset: (i32, i32)) {
        self.offset = offset;
    }

    pub fn add(&mut self, element: &'a Visible, z_index: i32) {
        self.elements.push(Instruction {
            z_index: z_index,
            object: element
        });
    }

    pub fn present(&mut self, renderer: &mut Renderer) {
        renderer.clear();

        for element in self.elements.iter() {
            element.object.show(self.offset, renderer);
        }

        renderer.present();
    }
}

/// A Visible object that consists of a single texture.
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
        let (x, y) = self.pos;
        renderer.draw(&self.name, (x+offx, y+offy));
    }
}
