pub use num::rational::Ratio;
use conv::{ValueFrom, ApproxFrom, ValueInto, ApproxInto};
use num::integer::Integer;

/// A trait defining how an integer can be scaled by ratios of various types.
/// `T` is the type of the ratio, `I` is the intermediate promotion type.
/// E.g., to correctly scale i8 by Ratio<u32>, we would want both self and
/// the scale ratio to be converted to i64, then down to i8 for the final
/// result:
/// ```
///     impl Scalable<u32, i64> for u8 {}
/// ```
/// Standard warnings about integer overflow apply.
///
pub trait Scalable<T, I>: Clone + Integer + ApproxFrom<I>
        where T: Clone + Integer,
              I: Clone + Integer + ValueFrom<T> + ValueFrom<Self>{
    fn scale(&self, scale: Ratio<T>) -> Self {
        // Promote all values to intermediate type
        let n: I = scale.numer().clone().value_into().unwrap();
        let d: I = scale.denom().clone().value_into().unwrap();
        let x: I = self.clone().value_into().unwrap();

        // Apply rational multiplication and convert back to integer
        (Ratio::from_integer(x) * Ratio::new(n, d))
            .to_integer()
            .approx_into()
            .unwrap()
    }
}

impl Scalable<u32, u32> for u32 {}
impl Scalable<i32, i32> for i32 {}

// Scaling an i32 by Ratio<u32> involves first converting to i64.
impl Scalable<u32, i64> for i32 {}

impl Scalable<u32, u32> for u8 {}
impl Scalable<u32, u32> for u16 {}
impl Scalable<u32, i64> for i8 {}
impl Scalable<u32, i64> for i16 {}

// Note that there is no implementation of Scalable<i32, X> for u32; scaling
// an unsigned integer by a signed ratio makes no sense since the desired
// result type (the same as the source integer's type) can never be negative.

#[test]
fn scale_u32() {
    let r = Ratio::new(1, 2);
    assert_eq!(4u32.scale(r), 2);
}

#[test]
fn scale_i32() {
    let r = Ratio::new(1, 2);
    assert_eq!(4i32.scale(r), 2);
}

#[test]
fn scale_i32_by_u32_ratio() {
    let r = Ratio::new(1u32, 2u32);
    assert_eq!(-4.scale(r), -2);
}

#[test]
fn scale_u8_by_u32_ratio() {
    let r = Ratio::new(1523u32, 1546u32);
    assert_eq!(232u8.scale(r), 228);
}

#[test]
fn scale_u16_by_u32_ratio() {
    let r = Ratio::new(7251u32, 492124u32);
    assert_eq!(51221u16.scale(r), 754);
}

#[test]
fn scale_i8_by_u32_ratio() {
    let r = Ratio::new(5u32, 25u32);
    assert_eq!(-100.scale(r), -20);
}

#[test]
fn scale_i16_by_u32_ratio() {
    let r = Ratio::new(5u32, 20u32);
    assert_eq!(-12.scale(r), -3);
}
