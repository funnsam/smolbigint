use core::cmp::Ordering;
use core::ops::*;
use crate::{BigUint, CarryOps};

macro_rules! impl_helpers {
    ($assign_t:ident : $assign_fn:ident , $biop_t:ident : $biop_fn:ident) => {
        impl<T: Into<Self>> $assign_t<T> for BigUint {
            fn $assign_fn(&mut self, rhs: T) {
                $assign_t::$assign_fn(self, &rhs.into());
            }
        }

        impl $biop_t<&Self> for BigUint {
            type Output = Self;

            fn $biop_fn(mut self, rhs: &Self) -> Self::Output {
                $assign_t::$assign_fn(&mut self, rhs);
                self
            }
        }

        impl<T: Into<Self>> $biop_t<T> for BigUint {
            type Output = Self;

            fn $biop_fn(mut self, rhs: T) -> Self::Output {
                $assign_t::$assign_fn(&mut self, &rhs.into());
                self
            }
        }
    };
}

impl_helpers!(AddAssign: add_assign, Add: add);
impl_helpers!(SubAssign: sub_assign, Sub: sub);
impl_helpers!(MulAssign: mul_assign, Mul: mul);
impl_helpers!(DivAssign: div_assign, Div: div);
impl_helpers!(RemAssign: rem_assign, Rem: rem);

impl AddAssign<&Self> for BigUint {
    fn add_assign(&mut self, rhs: &Self) {
        match (&mut *self, rhs) {
            (Self::Small(a), Self::Small(b)) => {
                let (v, c) = a.overflowing_add(*b);
                *self = if c { Self::Big(vec![v, c as _]) } else { Self::Small(v) };
            },
            (Self::Big(a), Self::Small(b)) => {
                if a.len() < 1 {
                    // self == 0
                    *self = Self::Small(*b);
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
            (_, Self::Big(b)) => {
                let mut a = core::mem::take(self).inflate_vec(b.len());

                let mut c = false;
                for (a, b) in a.iter_mut().zip(b.iter().chain(core::iter::repeat(&0))) {
                    (*a, c) = a._carrying_add(*b, c);
                }

                if c { a.push(1) }
                *self = Self::Big(a);
            },
        }
    }
}

impl SubAssign<&Self> for BigUint {
    fn sub_assign(&mut self, rhs: &Self) {
        match (&*self).cmp(rhs) {
            Ordering::Equal => return *self = Self::Small(0),
            Ordering::Less => panic!("attempt to subtract with overflow"),
            _ => {},
        }

        match (&mut *self, rhs) {
            (Self::Small(a), Self::Small(b)) => *a -= *b,
            (Self::Small(a), Self::Big(b)) => {
                // NOTE: `b` is strictly less than `a` here, so `b` must only have 1 digit in the
                // least significant place.
                *a -= b[0];
            },
            (Self::Big(a), Self::Small(b)) => {
                let (v, mut b) = a[0].overflowing_sub(*b);
                a[0] = v;

                // NOTE: if `a.len() >= 1` then `b` must be false, because if it's true then it'd
                // be caught in the first match, so the `a[1..]` never panics.
                if b {
                    for i in a[1..].iter_mut() {
                        (*i, b) = i.overflowing_sub(1);
                        if !b { break };
                    }
                }
            },
            (Self::Big(a), Self::Big(b)) => {
                let mut c = false;
                for (a, b) in a.iter_mut().zip(b.iter().chain(core::iter::repeat(&0))) {
                    (*a, c) = a._borrowing_sub(*b, c);
                }
            },
        }
    }
}

impl MulAssign<&Self> for BigUint {
    fn mul_assign(&mut self, rhs: &Self) {
        fn mul_big_small(a: &mut Vec<u64>, b: u64) {
            let mut c = 0;

            for i in a.iter_mut() {
                (*i, c) = i._carrying_mul(b, c);
            }

            if c != 0 { a.push(c) }
        }

        if *rhs == 0 { return *self = BigUint::from(0) };

        match (&mut *self, rhs) {
            (Self::Small(a), Self::Small(b)) => {
                let (v, c) = a._carrying_mul(*b, 0);
                *self = if c == 0 { Self::Small(v) } else { Self::Big(vec![v, c]) };
            },
            (Self::Big(a), Self::Small(b)) => mul_big_small(a, *b),
            (Self::Small(a), Self::Big(b)) => {
                let mut b = b.clone();
                mul_big_small(&mut b, *a);
                *self = Self::Big(b);
            },
            (Self::Big(a), Self::Big(b)) => {
                let mut acc = Self::from(0);

                if a.len() <= b.len() {
                    for (p, d) in a.iter().enumerate() {
                        let mut b = b.clone();
                        mul_big_small(&mut b, *d);
                        let mut b = BigUint::Big(b);
                        b.left_shift_places(p);
                        acc += b;
                    }
                } else {
                    for (p, d) in b.iter().enumerate() {
                        let mut a = a.clone();
                        mul_big_small(&mut a, *d);
                        let mut a = BigUint::Big(a);
                        a.left_shift_places(p);
                        acc += a;
                    }
                }

                *self = acc;
            },
        }
    }
}

impl DivAssign<&Self> for BigUint {
    fn div_assign(&mut self, rhs: &Self) {
        self.div_assign_rem(rhs);
    }
}

impl RemAssign<&Self> for BigUint {
    fn rem_assign(&mut self, rhs: &Self) {
        *self = self.div_assign_rem(rhs);
    }
}

impl BigUint {
    pub fn div_assign_rem(&mut self, rhs: &Self) -> Self {
        if *rhs == 0 {
            panic!("attempt to divide by 0");
        }

        if *self < *rhs {
            return core::mem::replace(self, BigUint::from(0));
        }

        match (&mut *self, rhs) {
            (BigUint::Small(a), BigUint::Small(b)) => {
                let rem = *a % *b;
                *a /= *b;
                BigUint::from(rem)
            },
            (BigUint::Small(a), BigUint::Big(b)) => {
                let rem = *a % b[0];
                *a /= b[0];
                BigUint::from(rem)
            },
            (BigUint::Big(a), BigUint::Small(b)) => {
                let mut rem = BigUint::from(0);

                for a in a.iter_mut().rev() {
                    let this_rem = (rem.clone() + *a) % *b;
                    let this_div = (rem.clone() + *a) / *b;
                    *a = this_div.try_into().unwrap();

                    rem = this_rem;
                    rem.left_shift_places(1);
                    rem.trim();
                }

                BigUint::from(rem)
            },
            (BigUint::Big(a), BigUint::Big(_)) => {
                let mut rem = BigUint::from(0);

                for a in a.iter_mut().rev() {
                    let this_rem = (rem.clone() + *a) % rhs;
                    let this_div = (rem.clone() + *a) / rhs;
                    *a = this_div.try_into().unwrap();

                    rem = this_rem;
                    rem.left_shift_places(1);
                    rem.trim();
                }

                BigUint::from(rem)
            },
        }
    }
}

#[test]
fn test_add_assign() {
    let mut a = BigUint::from(u64::MAX);
    a += 1;
    assert_eq!(a, BigUint::Big(vec![0, 1]));

    a += 3;
    assert_eq!(a, BigUint::Big(vec![3, 1]));

    a += u64::MAX;
    assert_eq!(a, BigUint::Big(vec![2, 2]));

    a += BigUint::Big(vec![1, u64::MAX]);
    assert_eq!(a, BigUint::Big(vec![3, 1, 1]));
}

#[test]
fn test_sub_assign() {
    let mut a = BigUint::Big(vec![3, 0, 1]);
    a -= 3;
    assert_eq!(a, BigUint::Big(vec![0, 0, 1]));

    a -= 1;
    assert_eq!(a, BigUint::Big(vec![u64::MAX, u64::MAX]));

    a -= 1;
    assert_eq!(a, BigUint::Big(vec![u64::MAX - 1, u64::MAX]));

    a -= BigUint::Big(vec![u64::MAX, 1]);
    assert_eq!(a, BigUint::Big(vec![u64::MAX, u64::MAX - 2]));

    a -= BigUint::Big(vec![u64::MAX - 1, u64::MAX - 2]);
    assert_eq!(a, 1);

    a -= BigUint::Big(vec![1, 0, 0]);
    assert_eq!(a, 0);
}

#[test]
fn test_mul_assign() {
    let mut a = BigUint::from(1);
    a *= 4;
    assert_eq!(a, 4);

    a *= u64::MAX / 4 + 1;
    assert_eq!(a, BigUint::Big(vec![0, 1]));

    a *= 4;
    assert_eq!(a, BigUint::Big(vec![0, 4]));

    a *= u64::MAX / 4 + 1;
    assert_eq!(a, BigUint::Big(vec![0, 0, 1]));

    a *= BigUint::Big(vec![0, 4, 1]);
    assert_eq!(a, BigUint::Big(vec![0, 0, 0, 4, 1]));

    a *= BigUint::Big(vec![0, u64::MAX / 4 + 1]);
    assert_eq!(a, BigUint::Big(vec![0, 0, 0, 0, 0, u64::MAX / 4 + 2]));
}

#[test]
fn test_div_assign() {
    let mut a = BigUint::Big(vec![0, 0, 0, 0, 0, u64::MAX / 4 + 2]);
    a /= BigUint::Big(vec![0, u64::MAX / 4 + 1]);
    assert_eq!(a, BigUint::Big(vec![0, 0, 0, 4, 1]));

    a /= BigUint::Big(vec![0, 4, 1]);
    assert_eq!(a, BigUint::Big(vec![0, 0, 1]));

    a /= u64::MAX / 4 + 1;
    assert_eq!(a, BigUint::Big(vec![0, 4]));

    a /= 4;
    assert_eq!(a, BigUint::Big(vec![0, 1]));

    a /= u64::MAX / 4 + 1;
    assert_eq!(a, 4);

    a /= 4;
    assert_eq!(a, 1);
}
