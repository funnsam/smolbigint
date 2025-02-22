mod u_ops;
mod util;

pub(crate) use util::*;

/// An unsigned big unsigned integer in heap, or an [`usize`] in stack if it fits.
pub enum BigUint {
    Small(usize),
    Big(Vec<usize>),
}
