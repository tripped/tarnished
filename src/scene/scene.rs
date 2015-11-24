// Simple 2D rendering instructions
extern crate sdl2;
extern crate sdl2_image;

use std::cmp::Ordering;
use std::collections::btree_set::BTreeSet;
use self::sdl2::render::{Renderer, Texture};
use self::sdl2_image::{LoadTexture, INIT_PNG};

/// A render operation is an input to the renderer that causes it to draw
/// something on the screen. During the construction of one frame, any number
/// of render operations may be created and added to the scene. When this
/// is done, a call to scene.present() will sort all the provided instructions
/// by z-index and execute them, then clear the scene.
///
/// In the future there should be a useful mechanism for expressing z-index
/// relationships in various ways, but for now every render instruction must
/// be added with an explicit position.
pub trait RenderOp {
    fn draw(renderer: &mut Renderer);
}

/// A RenderPosition describes where in the draw order an instruction belongs.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct RenderPosition {
    z_index: i32,
}

struct Instruction<'a> {
    position: RenderPosition,
    op: &'a RenderOp,
}

impl<'a> Ord for Instruction<'a> {
    fn cmp(&self, other: &Instruction) -> Ordering {
        self.position.cmp(&other.position)
    }
}

impl<'a> PartialOrd for Instruction<'a> {
    fn partial_cmp(&self, other: &Instruction) -> Option<Ordering> {
        self.position.partial_cmp(&other.position)
    }
}

impl<'a> PartialEq for Instruction<'a> {
    fn eq(&self, other: &Instruction) -> bool {
        self.position == other.position
    }
}

impl<'a> Eq for Instruction<'a> { }

/// A trait describing objects that produce sequences of render instructions.
pub trait Renderable {
    fn render(&self) -> Vec<Instruction>;
}

///
pub struct Scene<'a> {
    scene: BTreeSet<Instruction<'a>>,
}

impl<'a> Scene<'a> {
    fn new() -> Scene<'a> {
        Scene {
            scene: BTreeSet::new()
        }
    }

    /// Add a render instruction to the scene.
    fn add(&mut self, instruction: Instruction<'a>) {
        self.scene.insert(instruction);
    }
}
