pub use num::rational::Ratio;
use num::integer::Integer;

// XXX: should be able to scale signed int by either signed or unsigned ratio
pub trait Scalable: Clone + Integer {
    fn scale(&self, scale: Ratio<Self>) -> Self {
        (Ratio::from_integer(self.clone()) * scale).to_integer()
    }
}

impl Scalable for u32 {}
impl Scalable for i32 {}
