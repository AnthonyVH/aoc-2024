use std::num::NonZero;

/// Returns the number of bits required to represent the given number.
/// Returns 0 if the argument is 0.
pub fn bit_width(x: u64) -> u8 {
    (u64::BITS - x.leading_zeros()) as u8
}

pub fn digit_width_base10(x: u64) -> u32 {
    match x {
        0 => 1,
        value => (unsafe { NonZero::new_unchecked(value).ilog10() }) + 1,
    }
}
