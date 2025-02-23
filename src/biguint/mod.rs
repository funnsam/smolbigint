mod cmp;
mod fmt;
#[cfg(feature = "num-traits")]
mod num;
mod ops;

/// A big unsigned integer in heap, or an [`usize`] in stack if it fits.
///
/// # Internal representation
/// The order of the digits is little endian, so they are ordered from least significant to most
/// significant.
#[derive(Clone)]
pub enum BigUint {
    Small(u64),
    Big(Vec<u64>),
}

impl Default for BigUint {
    fn default() -> Self {
        Self::Small(0)
    }
}

impl From<u64> for BigUint {
    fn from(value: u64) -> Self {
        Self::Small(value)
    }
}

impl TryFrom<BigUint> for u64 {
    type Error = ();

    fn try_from(value: BigUint) -> Result<Self, Self::Error> {
        match value {
            BigUint::Small(v) => Ok(v),
            BigUint::Big(v) if value <= Self::MAX => Ok(*v.get(0).unwrap_or(&0)),
            _ => Err(()),
        }
    }
}

impl BigUint {
    pub fn left_shift_places(&mut self, n: usize) {
        if n == 0 { return };

        match self {
            Self::Small(v) => *self = Self::Big(vec![0, *v]),
            Self::Big(v) => {
                v.resize(v.len() + n, 0);
                v.rotate_right(n);
            },
        }
    }

    /// Trim away `0` digits.
    pub fn trim(&mut self) {
        match self {
            Self::Small(_) => {},
            Self::Big(v) => {
                let rpos = v.iter().rev().position(|i| *i != 0).unwrap_or(v.len());
                let len = v.len() - rpos;

                match len {
                    0 => *self = Self::Small(0),
                    1 => *self = Self::Small(v[0]),
                    2.. => v.truncate(len),
                }
            },
        }
    }

    /// Inflate `self` into a heap allocated integer with at least `len` digits.
    pub fn inflate(&mut self, len: usize) {
        let vec = core::mem::take(self).inflate_vec(len);
        *self = Self::Big(vec);
    }

    /// Inflate `self` into a heap allocated integer with at least `len` digits.
    fn inflate_vec(self, len: usize) -> Vec<u64> {
        match self {
            Self::Small(v) => {
                let mut vec = vec![0; len.max(1)];
                vec[0] = v;
                vec
            },
            Self::Big(mut v) => {
                v.resize(len.max(v.len()), 0);
                v
            }
        }
    }
}
