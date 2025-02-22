use core::ops::*;
use super::*;

impl AddAssign<&Self> for BigUint {
    fn add_assign(&mut self, rhs: &Self) {
        match (&mut *self, rhs) {
            (Self::Small(a), Self::Small(b)) => {
                let (v, c) = a.overflowing_add(*b);

                *self = if c {
                    Self::Big(vec![v, c as usize])
                } else {
                    Self::Small(v)
                };
            },
            (Self::Big(a), Self::Small(b)) => {
                if a.len() < 1 {
                    *self = Self::Small(*b)
                } else {
                    let (v, mut c) = a[0].overflowing_add(*b);
                    a[0] = v;
                    if !c { return };

                    for i in a[1..].iter_mut() {
                        (*i, c) = i.overflowing_add(1);
                        if !c { return };
                    }
                    a.push(1);
                }
            },
            (Self::Small(a), Self::Big(b)) => {
                todo!();
            },
            (Self::Big(a), Self::Big(b)) => {
                if a.len() < b.len() {
                    a.resize(b.len(), 0);
                }

                let mut c = false;
                for (a, b) in a.iter_mut().zip(b.iter()) {
                    (*a, c) = a._carrying_add(*b, c);
                }

                if c { a.push(1) }
            },
        }
    }
}
