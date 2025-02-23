use crate::BigUint;
use num_traits::*;

impl Zero for BigUint {
    fn zero() -> Self {
        Self::Small(0)
    }

    fn is_zero(&self) -> bool {
        match self {
            Self::Small(v) => *v == 0,
            Self::Big(v) => !v.iter().any(|i| *i != 0),
        }
    }

    fn set_zero(&mut self) {
        *self = Self::Small(0);
    }
}

impl One for BigUint {
    fn one() -> Self {
        Self::Small(1)
    }

    fn is_one(&self) -> bool {
        match self {
            Self::Small(v) => *v == 1,
            Self::Big(v) => v.len() >= 1 && v[0] == 1 && !v.iter().skip(1).any(|i| *i != 0),
        }
    }
}
