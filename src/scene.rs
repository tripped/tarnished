extern crate sdl2;
extern crate sdl2_image;

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::path::Path;

use self::sdl2::rect::Rect;
use self::sdl2::render::Texture;
use self::sdl2_image::LoadTexture;

/// Specifies a draw rect's horizontal position and alignment
#[derive(Copy, Clone)]
pub enum HorizontalPlacement {
    Left(i32),
    Right(i32),
    Center(i32),
    Stretch(i32, i32),
}

/// Specifies a draw rect's vertical position and alignment
#[derive(Copy, Clone)]
pub enum VerticalPlacement {
    Top(i32),
    Bottom(i32),
    Center(i32),
    Stretch(i32, i32),
}

pub fn left(x: i32) -> HorizontalPlacement {
    HorizontalPlacement::Left(x)
}

pub fn right(x: i32) -> HorizontalPlacement {
    HorizontalPlacement::Right(x)
}

pub fn hcenter(x: i32) -> HorizontalPlacement {
    HorizontalPlacement::Center(x)
}

pub fn hstretch(x1: i32, x2: i32) -> HorizontalPlacement {
    HorizontalPlacement::Stretch(x1, x2)
}

pub fn top(y: i32) -> VerticalPlacement {
    VerticalPlacement::Top(y)
}

pub fn bottom(y: i32) -> VerticalPlacement {
    VerticalPlacement::Bottom(y)
}

pub fn vcenter(y: i32) -> VerticalPlacement {
    VerticalPlacement::Center(y)
}

pub fn vstretch(y1: i32, y2: i32) -> VerticalPlacement {
    VerticalPlacement::Stretch(y1, y2)
}

/// Renderer: 1. n. A person or thing that renders.
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

    pub fn draw(&mut self, asset: &str, 
                hpos: HorizontalPlacement,
                vpos: VerticalPlacement) {
        self.ensure_texture(asset);
        let tex = self.textures.get(&asset.to_string()).unwrap();

        let query = tex.query();
        let width = query.width as i32;
        let height = query.height as i32;

        let (x1, x2) = match hpos {
            HorizontalPlacement::Left(x) => (x, x + width),
            HorizontalPlacement::Right(x) => (x - width, x),
            HorizontalPlacement::Center(x) => (x - width/2, x + width/2),
            HorizontalPlacement::Stretch(x1, x2) => (x1, x2),
        };

        let (y1, y2) = match vpos {
            VerticalPlacement::Top(y) => (y, y + height),
            VerticalPlacement::Bottom(y) => (y - height, y),
            VerticalPlacement::Center(y) => (y - height/2, y + height/2),
            VerticalPlacement::Stretch(y1, y2) => (y1, y2),
        };

        // XXX: check these bounds; we'll just panic in the case of inversion
        let dst = Rect::new(x1, y1,
            (x2 - x1) as u32, (y2 - y1) as u32).unwrap();
        self.renderer.copy(&tex, None, dst);
    }

    pub fn draw_stretched(&mut self, asset: &str, dst: Rect) {
        self.ensure_texture(asset);
        let tex = self.textures.get(&asset.to_string()).unwrap();
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
    hpos: HorizontalPlacement,
    vpos: VerticalPlacement,
}

impl Sprite {
    pub fn new(name: &str, h: HorizontalPlacement, v: VerticalPlacement)
        -> Sprite {
        Sprite {
            name: name.into(),
            hpos: h,
            vpos: v,
        }
    }
}

pub fn sprite(name: &str, h: HorizontalPlacement, v: VerticalPlacement) -> Sprite {
    Sprite::new(name, h, v)
}

impl Visible for Sprite {
    fn show(&self, (offx, offy): (i32, i32), renderer: &mut Renderer) {
        renderer.draw(&self.name, self.hpos, self.vpos);
    }
}
