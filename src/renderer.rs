use std::collections::HashMap;
use std::path::Path;

use sdl2;
use sdl2_ttf;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Texture, TextureQuery};
use sdl2_image::LoadTexture;
use num::rational::Ratio;

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
    ttf: sdl2_ttf::Sdl2TtfContext,
}

impl RenderContext {
    pub fn new(ttf: sdl2_ttf::Sdl2TtfContext) -> RenderContext {
        RenderContext {
            textures: HashMap::new(),
            ttf: ttf,
        }
    }

    /// Load a texture if it does not already exist in cache
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
    pub fn get_texture(&mut self, asset: &str,
                       renderer: &sdl2::render::Renderer) -> Option<&Texture> {
        self.ensure_texture(asset, renderer);
        self.textures.get(asset)
    }

    /// Query information about a texture by its asset name.
    pub fn query(&mut self, asset: &str,
                 renderer: &sdl2::render::Renderer) -> Option<TextureQuery> {
        match self.get_texture(asset, renderer) {
            Some(tex) => Some(tex.query()),
            None => None
        }
    }
}

/// Renderer: 1. n. A person or thing that renders.
pub struct Renderer<'a> {
    renderer: &'a mut sdl2::render::Renderer<'static>,
    offset: (i32, i32),
    original_scale: (f32, f32),
}

impl<'a> Renderer<'a> {
    /// Create a new Renderer, wrapping an existing SDL Renderer and
    /// setting up drawing for the specified scale and translation.
    pub fn new(renderer: &'a mut sdl2::render::Renderer<'static>,
               offset: (i32, i32),
               scale: Ratio<u32>) -> Renderer<'a> {
        let scale = *scale.numer() as f32 / *scale.denom() as f32;
        let original_scale = renderer.scale();
        renderer.set_scale(scale, scale);
        Renderer {
            renderer: renderer,
            offset: offset,
            original_scale: original_scale,
        }
    }

    /// Copy a named texture onto the rendering surface at specified
    /// destination rect, with optional source rect.
    pub fn copy(&mut self, context: &mut RenderContext,
            asset: &str, src: Option<Rect>, dst: Rect) {
        let tex = context.get_texture(asset, &self.renderer).unwrap();

        let (dx, dy) = self.offset;
        let x = (dst.x() - dx) as i32;
        let y = (dst.y() - dy) as i32;
        let w = dst.width();
        let h = dst.height();
        let dst = Rect::new_unwrap(x, y, w, h);

        self.renderer.copy(tex, src, Some(dst));
    }

    pub fn draw(&mut self, context: &mut RenderContext,
                asset: &str, hpos: HPos, vpos: VPos) {
        let query = context.query(asset, &self.renderer).unwrap();
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

        let dst = Rect::new_unwrap(x1, y1, (x2 - x1) as u32, (y2 - y1) as u32);
        self.copy(context, asset, None, dst);
    }

    /// Draw the nth tile of specified size from a given image asset, at a
    /// specific position, assuming top-left alignment.
    /// XXX: Assumes that tiles are laid out in a single horizontal strip.
    pub fn draw_tile(&mut self, context: &mut RenderContext,
                     tileset: &str, n: u32, w: u32, h: u32,
                     x: i32, y: i32) {
        let src = Rect::new_unwrap((n * w) as i32, 0, w, h);
        let dst = Rect::new_unwrap(x, y, w, h);
        self.copy(context, tileset, Some(src), dst);
    }

    /// Draw a string using a specified font.
    pub fn draw_text(&mut self, context: &mut RenderContext,
                     text: &str, font: &str, x: i32, y: i32) {
        // Check the texture cache for this string
        let id = format!(":STR:{}:{}", font, text);

        if !context.has_texture(&id) {
            let path = font.to_string() + ".ttf";
            let path = Path::new(&path);
            let font = context.ttf.load_font(path, 14).unwrap();

            // render a surface, and convert it to a texture
            // XXX: configurable text color
            let surface = font.render(text)
                .solid(Color::RGBA(224, 224, 224, 255)).unwrap();
            let texture = self.renderer
                .create_texture_from_surface(&surface).unwrap();

            context.add_texture(&id, texture);
        }

        self.draw(context, &id, HPos::Center(x), VPos::Center(y));
    }

    /// Draw a non-filled rectangle onto the target surface.
    /// NB: note that multiple rectangles drawn in the same layer could be
    /// optimized into a draw_rects() call. Something to think about when
    /// implementing the widget system.
    pub fn draw_rect(&mut self, rect: Rect, color: Color) {
        self.renderer.set_draw_color(color);
        self.renderer.draw_rect(rect);
    }

    /// Draw a filled rectangle onto the target surface.
    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        self.renderer.set_draw_color(color);
        self.renderer.fill_rect(rect);
    }
}

impl<'a> Drop for Renderer<'a> {
    fn drop(&mut self) {
        // Restore the borrowed renderer to its original scale
        let (sx, sy) = self.original_scale;
        self.renderer.set_scale(sx, sy);
    }
}
