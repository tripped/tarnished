use carboxyl::{Signal, Sink};
use num::rational::Ratio;
use renderer::{Renderer, RenderContext, HPos, VPos};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// A `Visible` object can be shown using a renderer. It is atomic with respect
/// to z-ordering, i.e., it is always entirely behind or entirely in front of
/// other visible objects. Rendering a frame or scene is a process of creating
/// many Visibles, sorting them by layer and z-index, and showing them.
pub trait Visible {
    fn show(&self, renderer: &mut Renderer, context: &mut RenderContext);
}

/// A scene Instruction is a Visible and a z-index at which it is to be shown.
pub struct Instruction<'a> {
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

    /// Remove the rearmost element from the Scene and return it.
    /// Panics if the scene is empty.
    pub fn pop(&mut self) -> Instruction<'a> {
        self.elements.pop().unwrap()
    }

    /// Presents the scene onto the specified renderer.
    /// Consumes the scene's contents in the process.
    pub fn present(mut self, renderer: &mut sdl2::render::Renderer<'static>,
                   context: &mut RenderContext,
                   translation: (i32, i32),
                   scale: Ratio<u32>) {
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
                          scale: Ratio<u32>) {
        self.present(renderer, context, (0, 0), scale);
    }
}

/// A Visible object that consists of a single texture copy with specified
/// source and destination. The most primitive texture-based Visible.
pub struct Tex {
    asset: String,
    src: Option<Rect>,
    dst: Rect,
}

impl Tex {
    pub fn new(asset: &str, src: Option<Rect>, dst: Rect) -> Tex {
        Tex {
            asset: asset.into(),
            src: src,
            dst: dst,
        }
    }
}

impl Visible for Tex {
    fn show(&self, renderer: &mut Renderer, context: &mut RenderContext) {
        renderer.copy(context, &self.asset, self.src, self.dst);
    }
}

/// A Visible object that consists of a single texture drawn at its native
/// scale, using dimension-aware alignment.
/// XXX: resolve generality/usefulness mismatch here between Tex, Sprite,
/// and Tile -- seems like there's too much overlap and we keep bumping into
/// it in the higher layers.
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
#[derive(Clone)]
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

    pub fn _unfilled(rect: Rect, color: Color) -> Rectangle {
        Rectangle {
            rect: rect,
            color: color,
            filled: false,
        }
    }
}

impl Visible for Rectangle {
    fn show(&self, renderer: &mut Renderer, _: &mut RenderContext) {
        if self.filled {
            renderer.fill_rect(self.rect, self.color);
        } else {
            renderer.draw_rect(self.rect, self.color);
        }
    }
}

// Tests!

#[test]
fn scene_pop_works() {
    #[derive(Eq, PartialEq)]
    struct Pixel {};
    impl Visible for Pixel {
        // XXX: Scene's role as a strict value container perhaps argues against
        // concrete Renderer and RenderContext dependencies for its items?
        fn show(&self, _: &mut Renderer, _: &mut RenderContext) { }
    }

    let fore = Pixel {};
    let back = Pixel {};

    let mut s = Scene::new();
    s.add(&fore, 10);
    s.add(&back, 0);

    s.pop();
    s.pop();

    // XXX: the way Scene deals with render instructions as trait objects
    // makes it difficult to test! The only way to actually interact with
    // a Visible right now is to show it on a Renderer.
    //assert_eq!(&fore, s.pop());
}

#[derive(Clone, Eq, PartialEq, Debug)]
enum Show {
    Sprite(String),
}

// XXX: maybe should delete Stream parameter since we don't use it yet
fn world<I>(gen: &mut I) -> Signal<Vec<Show>>
        where I: Iterator<Item=Signal<Show>> {

    // XXX: obviously just taking three signals off the iterator is wrong;
    // a better approach for producing the combined behavior is needed.
    let a = gen.next().unwrap();
    let b = gen.next().unwrap();
    let c = gen.next().unwrap();

    lift!(|a, b, c| vec![a, b, c],
        &a, &b, &c)
}

#[test]
fn world_uses_generator() {
    let generator: Sink<Signal<Show>> = Sink::new();

    let mut gen_events = generator.stream().events();

    // The generator will yield the behaviors that defines our scene
    let sprites: Vec<Signal<Show>> = vec![
        Signal::new(Show::Sprite("foo".into())),
        Signal::new(Show::Sprite("bar".into())),
        Signal::new(Show::Sprite("baz".into()))];

    generator.feed(sprites);

    let my_world: Signal<Vec<Show>> = world(&mut gen_events);

    assert_eq!(my_world.sample(), vec![
        Show::Sprite("foo".into()),
        Show::Sprite("bar".into()),
        Show::Sprite("baz".into())]);
}
