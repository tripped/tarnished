extern crate sdl2;
extern crate sdl2_image;

use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::path::Path;

use self::sdl2::rect::Rect;
use self::sdl2::render::{Texture, TextureQuery};
use self::sdl2_image::LoadTexture;

/// Specifies a draw rect's horizontal position and alignment
#[derive(Copy, Clone)]
pub enum HPos {
    Left(i32),
    Right(i32),
    Center(i32),
    Stretch(i32, usize),
}

/// Specifies a draw rect's vertical position and alignment
#[derive(Copy, Clone)]
pub enum VPos {
    Top(i32),
    Bottom(i32),
    Center(i32),
    Stretch(i32, usize),
}

/// Renderer: 1. n. A person or thing that renders.
pub struct Renderer {
    renderer: RefCell<sdl2::render::Renderer<'static>>,
    textures: RefCell<HashMap<String, Texture>>,
    offset: (i32, i32),
    scale: (f32, f32),
}

fn load_texture(asset: &str, renderer: &sdl2::render::Renderer) -> Texture {
    // TODO: handle assets more intelligently than just appending ".png"
    // TODO: define a wrapper that properly uses AsRef<Path>, because this
    // conversion is rather inconvenient
    println!("Loading texture `{}`...", asset);
    let path = asset.to_string() + ".png";
    let path = Path::new(&path);
    renderer.load_texture(path).unwrap()
}

impl Renderer {
    pub fn new(renderer: sdl2::render::Renderer<'static>) -> Renderer {
        Renderer {
            renderer: RefCell::new(renderer),
            textures: RefCell::new(HashMap::new()),
            offset: (0, 0),
            scale: (1.0, 1.0),
        }
    }

    pub fn set_viewport(&mut self, offset: (i32, i32)) {
        self.offset = offset;
    }

    /// Set the scaling factors used by default for copying textures.
    pub fn set_copy_scale(&mut self, xs: f32, ys: f32) {
        self.scale = (xs, ys);
    }

    /// Set the scaling factors applied to everything.
    pub fn set_global_scale(&mut self, xscale: f32, yscale: f32) {
        self.renderer.borrow_mut().set_scale(xscale, yscale);
    }

    pub fn clear(&self) {
        self.renderer.borrow_mut().clear();
    }

    pub fn present(&self) {
        self.renderer.borrow_mut().present();
    }

    /// Load a texture if it does not already exist in cache
    ///
    /// XXX: ideally, this function would return a reference to the texture.
    /// However, returning a reference would extend the lifetime of the borrow
    /// on `self` which implicitly borrows `self.renderer`. So, we'd be unable
    /// to draw the texture we just requested. Currently the only way to fix
    /// this would be to redesign this system so that renderer and the texture
    /// cache aren't in the same struct.
    fn ensure_texture(&self, asset: &str) {
        if !self.textures.borrow().contains_key(asset) {
            self.textures.borrow_mut().insert(asset.into(),
                load_texture(asset, &self.renderer.borrow_mut()));
        }
    }

    /// Query information about a texture by its asset name.
    pub fn query(&self, asset: &str) -> TextureQuery {
        self.ensure_texture(asset);
        let cache = self.textures.borrow();
        let tex = cache.get(&asset.to_string()).unwrap();
        tex.query()
    }

    /// Copy texture, scaled by the default copy scale
    fn copy(&self, asset: &str, src: Option<Rect>, dst: Rect) {
        self.ensure_texture(asset);
        let cache = self.textures.borrow();
        let tex = cache.get(&asset.to_string()).unwrap();

        let (sx, sy) = self.scale;
        let x = (dst.x() as f32 * sx) as i32;
        let y = (dst.y() as f32 * sy) as i32;
        let w = (dst.width() as f32 * sx) as u32;
        let h = (dst.height() as f32 * sy) as u32;
        let dst = Rect::new_unwrap(x, y, w, h);
        self.renderer.borrow_mut().copy(tex, src, Some(dst));
    }

    pub fn draw(&self, asset: &str, hpos: HPos, vpos: VPos) {
        let query = self.query(asset);
        let width = query.width as i32;
        let height = query.height as i32;

        let (x1, x2) = match hpos {
            HPos::Left(x) => (x, x + width),
            HPos::Right(x) => (x - width, x),
            HPos::Center(x) => (x - width/2, x + width/2),
            HPos::Stretch(x, w) => (x, x+w as i32),
        };

        let (y1, y2) = match vpos {
            VPos::Top(y) => (y, y + height),
            VPos::Bottom(y) => (y - height, y),
            VPos::Center(y) => (y - height/2, y + height/2),
            VPos::Stretch(y, h) => (y, y+h as i32),
        };

        let (offx, offy) = self.offset;
        let dst = Rect::new_unwrap(x1, y1, (x2 - x1) as u32, (y2 - y1) as u32);
        let dst = dst.offset(offx, offy).unwrap();

        self.copy(asset, None, dst);
    }

    /// Draw the nth tile of specified size from a given image asset, at a
    /// specific position, assuming top-left alignment.
    /// XXX: Assumes that tiles are laid out in a single horizontal strip.
    pub fn draw_tile(&self, tileset: &str, n: u32, w: u32, h: u32,
                     x: i32, y: i32) {
        let src = Rect::new_unwrap((n * w) as i32, 0, w, h);
        let (offx, offy) = self.offset;
        let dst = Rect::new_unwrap(x, y, w, h).offset(offx, offy).unwrap();

        self.copy(tileset, Some(src), dst);
    }
}

/// A `Visible` object can be shown using a renderer. It is atomic with respect
/// to z-ordering, i.e., it is always entirely behind or entirely in front of
/// other visible objects. Rendering a frame or scene is a process of creating
/// many Visibles, sorting them by layer and z-index, and showing them.
pub trait Visible {
    fn show(&self, renderer: &mut Renderer);
}

/// A scene Instruction is a Visible and a z-index at which it is to be shown.
struct Instruction<'a> {
    z_index: i32,
    object: &'a Visible,
}

impl<'a> Ord for Instruction<'a> {
    fn cmp(&self, other: &Instruction) -> Ordering {
        other.z_index.cmp(&self.z_index)
    }
}

impl<'a> PartialOrd for Instruction<'a> {
    fn partial_cmp(&self, other: &Instruction) -> Option<Ordering> {
        other.z_index.partial_cmp(&self.z_index)
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

    /// Presents the scene onto the specified renderer.
    /// Consumes the scene's contents in the process.
    pub fn present(&mut self, renderer: &mut Renderer) {
        renderer.clear();
        renderer.set_viewport(self.offset);

        loop {
            match self.elements.pop() {
                Some(element) => element.object.show(renderer),
                None => break
            }
        }

        renderer.present();
    }
}

/// A Visible object that consists of a single texture.
pub struct Sprite {
    name: String,
    hpos: HPos,
    vpos: VPos,
}

impl Sprite {
    pub fn new(name: &str, h: HPos, v: VPos)
        -> Sprite {
        Sprite {
            name: name.into(),
            hpos: h,
            vpos: v,
        }
    }
}

pub fn sprite(name: &str, h: HPos, v: VPos) -> Sprite {
    Sprite::new(name, h, v)
}

impl Visible for Sprite {
    fn show(&self, renderer: &mut Renderer) {
        renderer.draw(&self.name, self.hpos, self.vpos);
    }
}

/// A Visible object that is a tile drawn from a tileset.
pub struct Tile {
    tileset: String,
    tile: u32,
    width: u32,
    height: u32,
    x: i32,
    y: i32,
}

impl Tile {
    pub fn new(tileset: &str, tile: u32, w: u32, h: u32,
               x: i32, y: i32) -> Tile {
        Tile {
            tileset: tileset.into(),
            tile: tile,
            width: w,
            height: h,
            x: x,
            y: y,
        }
    }
}

pub fn tile(tileset: &str, n: u32, w: u32, h: u32, x: i32, y: i32) -> Tile {
    Tile::new(tileset, n, w, h, x, y)
}

impl Visible for Tile {
    fn show(&self, renderer: &mut Renderer) {
        renderer.draw_tile(&self.tileset, self.tile, self.width, self.height,
                      self.x, self.y);
    }
}
