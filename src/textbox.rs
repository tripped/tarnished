use std::path::Path;

use sdl2::rect::Rect;
use scene::{Visible, Sprite, sprite};
use renderer::{HPos, VPos};

pub struct Textbox {
    base: String,
    bounds: Rect,
}

impl Textbox {
    pub fn new(base: &str, bounds: Rect) -> Textbox {
        Textbox {
            base: base.to_string(),
            bounds: bounds,
        }
    }

    fn part(&self, name: &str) -> String {
        Path::new(&self.base).join(name).to_string_lossy().into_owned()
    }

    // XXX: Note that here we've given Textbox a method which just dumps a
    // whole bunch of sprites into a vector. Later, something needs to take
    // those sprites and put them into a Scene. Unfortunately, Scene::add()
    // is where information about z-index comes in, and Textbox probably has
    // an opinion about that. At the very least, conventional wisdom dictates
    // that all the bits of the textbox should have the same z-level.
    //
    // Given that our design philosophy emphasizes decoupling and value-based
    // programming, the lazy approach of giving render() a &Scene parameter
    // is obviously a no-go. Other options include promoting the Instruction
    // type to public and making it the output of typical render functions,
    // and the input to Scene::add().
    pub fn render(&self) -> Vec<Sprite> {
        let w = self.bounds.width() as usize;
        let h = self.bounds.height() as usize;
        let x = self.bounds.x();
        let y = self.bounds.y();
        let r = x + w as i32;
        let b = y + h as i32;

        vec![
            sprite(&self.part("tl"), HPos::Right(x),      VPos::Bottom(y)),
            sprite(&self.part("t"),  HPos::Stretch(x, w), VPos::Bottom(y)),
            sprite(&self.part("tr"), HPos::Left(r),       VPos::Bottom(y)),
            sprite(&self.part("l"),  HPos::Right(x),      VPos::Stretch(y, h)),
            sprite(&self.part("r"),  HPos::Left(r),       VPos::Stretch(y, h)),
            sprite(&self.part("bl"), HPos::Right(x),      VPos::Top(b)),
            sprite(&self.part("b"),  HPos::Stretch(x, w), VPos::Top(b)),
            sprite(&self.part("br"), HPos::Left(r),       VPos::Top(b)),
        ]
    }
}
