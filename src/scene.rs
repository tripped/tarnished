use std::cmp::Ordering;
use std::collections::BinaryHeap;

use sdl2;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use renderer::{Renderer, RenderContext, HPos, VPos};

/// A `Visible` object can be shown using a renderer. It is atomic with respect
/// to z-ordering, i.e., it is always entirely behind or entirely in front of
/// other visible objects. Rendering a frame or scene is a process of creating
/// many Visibles, sorting them by layer and z-index, and showing them.
pub trait Visible {
    fn show(&self, renderer: &mut Renderer, context: &mut RenderContext);
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
}

impl<'a> Scene<'a> {
    pub fn new() -> Scene<'a> {
        Scene {
            elements: BinaryHeap::new(),
        }
    }

    pub fn add(&mut self, element: &'a Visible, z_index: i32) {
        self.elements.push(Instruction {
            z_index: z_index,
            object: element
        });
    }

    /// Add multiple objects with the same z-index
    pub fn add_all<P: Visible>(&mut self, elements: &'a Vec<P>, z_index: i32) {
        for obj in elements {
            self.add(obj, z_index);
        }
    }

    /// Presents the scene onto the specified renderer.
    /// Consumes the scene's contents in the process.
    pub fn present(mut self, renderer: &mut sdl2::render::Renderer<'static>,
                   context: &mut RenderContext,
                   translation: (i32, i32),
                   scale: (f32, f32)) {
        let mut renderer = Renderer::new(renderer, translation, scale);
        loop {
            match self.elements.pop() {
                Some(element) => element.object.show(&mut renderer, context),
                None => break
            }
        }
    }

    pub fn present_scaled(self,
                          renderer: &mut sdl2::render::Renderer<'static>,
                          context: &mut RenderContext,
                          scale: (f32, f32)) {
        self.present(renderer, context, (0, 0), scale);
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
    fn show(&self, renderer: &mut Renderer, context: &mut RenderContext) {
        renderer.draw(context, &self.name, self.hpos, self.vpos);
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
    fn show(&self, renderer: &mut Renderer, context: &mut RenderContext) {
        renderer.draw_tile(context, &self.tileset, self.tile,
                           self.width, self.height, self.x, self.y);
    }
}

/// A Visible object that displays text in a given font
pub struct Text {
    text: String,
    font: String,
    x: i32,
    y: i32
}

impl Text {
    pub fn new(text: &str, font: &str, x: i32, y: i32) -> Text {
        Text {
            text: text.into(),
            font: font.into(),
            x: x,
            y: y
        }
    }
}

pub fn text(text: &str, font: &str, x: i32, y: i32) -> Text {
    Text::new(text, font, x, y)
}

impl Visible for Text {
    fn show(&self, renderer: &mut Renderer, context: &mut RenderContext) {
        renderer.draw_text(context, &self.text, &self.font, self.x, self.y);
    }
}

/// A Visible object that is just a rectangle. Woo, rectangles.
pub struct Rectangle {
    rect: Rect,
    color: Color,
    filled: bool,
}

impl Rectangle {
    pub fn filled(rect: Rect, color: Color) -> Rectangle {
        Rectangle {
            rect: rect,
            color: color,
            filled: true,
        }
    }

    pub fn unfilled(rect: Rect, color: Color) -> Rectangle {
        Rectangle {
            rect: rect,
            color: color,
            filled: false,
        }
    }
}

impl Visible for Rectangle {
    fn show(&self, renderer: &mut Renderer, context: &mut RenderContext) {
        if self.filled {
            renderer.fill_rect(self.rect, self.color);
        } else {
            renderer.draw_rect(self.rect, self.color);
        }
    }
}
