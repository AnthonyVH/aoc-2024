/// Returns the number of bits required to represent the given number.
/// Returns 0 if the argument is 0.
pub fn bit_width(x: u64) -> u8 {
    (u64::BITS - x.leading_zeros()) as u8
}
