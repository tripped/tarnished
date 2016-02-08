extern crate sdl2;
extern crate sdl2_image;
extern crate sdl2_ttf;

use std::collections::HashMap;
use std::path::Path;

use self::sdl2::pixels::Color;
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

/// Load a texture from an image asset, bound to a given Renderer
fn load_texture(asset: &str, renderer: &sdl2::render::Renderer) -> Texture {
    // TODO: handle assets more intelligently than just appending ".png"
    // TODO: define a wrapper that properly uses AsRef<Path>, because this
    // conversion is rather inconvenient
    println!("Loading texture `{}`...", asset);
    let path = asset.to_string() + ".png";
    let path = Path::new(&path);
    renderer.load_texture(path).unwrap()
}

/// A context for rendering, including asset caches.
/// XXX: rename?
pub struct RenderContext {
    textures: HashMap<String, Texture>,
}

impl RenderContext {
    pub fn new() -> RenderContext {
        RenderContext {
            textures: HashMap::new(),
        }
    }

    /// Load a texture if it does not already exist in cache
    ///
    /// XXX: ideally, this function would return a reference to the texture.
    /// However, returning a reference would extend the lifetime of the borrow
    /// on `self` which implicitly borrows `self.renderer`. So, we'd be unable
    /// to draw the texture we just requested. Currently the only way to fix
    /// this would be to redesign this system so that renderer and the texture
    /// cache aren't in the same struct.
    fn ensure_texture(&mut self, asset: &str,
                      renderer: &sdl2::render::Renderer) {
        if !self.textures.contains_key(asset) {
            self.textures.insert(asset.into(),
                load_texture(asset, renderer));
        }
    }

    pub fn has_texture(&self, name: &str) -> bool {
        self.textures.contains_key(name)
    }

    pub fn add_texture(&mut self, name: &str, tex: Texture) {
        self.textures.insert(name.into(), tex);
    }

    /// Return a texture if it exists in cache, otherwise None.
    pub fn get_texture(&self, asset: &str) -> Option<&Texture> {
        self.textures.get(asset)
    }

    /// Query information about a texture by its asset name.
    pub fn query(&mut self, asset: &str,
                 renderer: &sdl2::render::Renderer) -> TextureQuery {
        self.ensure_texture(asset, renderer);
        // XXX: we expect this unwrap to be safe, but really we shouldn't
        let tex = self.get_texture(asset).unwrap();
        tex.query()
    }
}

/// Renderer: 1. n. A person or thing that renders.
pub struct Renderer {
    renderer: sdl2::render::Renderer<'static>,
    ttf: sdl2_ttf::Sdl2TtfContext,
    offset: (i32, i32),
    scale: (f32, f32),
}

impl Renderer {
    pub fn new(renderer: sdl2::render::Renderer<'static>,
               ttf: sdl2_ttf::Sdl2TtfContext) -> Renderer {
        Renderer {
            renderer: renderer,
            ttf: ttf,
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
        self.renderer.set_scale(xscale, yscale);
    }

    pub fn clear(&mut self) {
        self.renderer.clear();
    }

    pub fn present(&mut self) {
        self.renderer.present();
    }

    /// Copy texture, scaled by the default copy scale
    fn copy(&mut self, context: &mut RenderContext,
            asset: &str, src: Option<Rect>, dst: Rect) {
        context.ensure_texture(asset, &self.renderer);
        let tex = context.get_texture(asset).unwrap();

        let (sx, sy) = self.scale;
        let x = (dst.x() as f32 * sx) as i32;
        let y = (dst.y() as f32 * sy) as i32;
        let w = (dst.width() as f32 * sx) as u32;
        let h = (dst.height() as f32 * sy) as u32;
        let dst = Rect::new_unwrap(x, y, w, h);

        self.renderer.copy(tex, src, Some(dst));
    }

    pub fn draw(&mut self, context: &mut RenderContext,
                asset: &str, hpos: HPos, vpos: VPos) {
        let query = context.query(asset, &self.renderer);
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

        self.copy(context, asset, None, dst);
    }

    /// Draw the nth tile of specified size from a given image asset, at a
    /// specific position, assuming top-left alignment.
    /// XXX: Assumes that tiles are laid out in a single horizontal strip.
    pub fn draw_tile(&mut self, context: &mut RenderContext,
                     tileset: &str, n: u32, w: u32, h: u32,
                     x: i32, y: i32) {
        let src = Rect::new_unwrap((n * w) as i32, 0, w, h);
        let (offx, offy) = self.offset;
        let dst = Rect::new_unwrap(x, y, w, h).offset(offx, offy).unwrap();

        self.copy(context, tileset, Some(src), dst);
    }

    /// Draw a string using a specified font.
    /// XXX: not affected by copy scaling -- yet!
    pub fn draw_text(&mut self, context: &mut RenderContext,
                     text: &str, font: &str, x: i32, y: i32) {
        // Check the texture cache for this string
        let id = format!(":STR:{}:{}", font, text);

        if !context.has_texture(&id) {
            let path = font.to_string() + ".ttf";
            let path = Path::new(&path);
            let font = self.ttf.load_font(path, 14).unwrap();

            // render a surface, and convert it to a texture
            let surface = font.render(text)
                .solid(Color::RGBA(100, 0, 0, 255)).unwrap();
            let texture = self.renderer
                .create_texture_from_surface(&surface).unwrap();

            context.add_texture(&id, texture);
        }

        // XXX: this borrowing is fairly ugly -- should redesign
        // renderer into multiple parts to separate texture cache
        let texture = context.get_texture(&id).unwrap();
        let TextureQuery { width, height, .. } = texture.query();
        let dst = Rect::new_unwrap(x, y, width, height);

        self.renderer.copy(&texture, None, Some(dst));
    }
}
