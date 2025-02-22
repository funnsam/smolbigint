use core::cmp::*;
use super::*;

impl PartialEq<u64> for BigUint {
    fn eq(&self, other: &u64) -> bool {
        self == &BigUint::Small(*other)
    }
}

impl PartialOrd<u64> for BigUint {
    fn partial_cmp(&self, other: &u64) -> Option<Ordering> {
        self.partial_cmp(&BigUint::Small(*other))
    }
}

impl PartialEq<Self> for BigUint {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Small(a), Self::Small(b)) => a == b,
            (Self::Big(a), Self::Small(b)) | (Self::Small(b), Self::Big(a)) => {
                (a.len() == 0 && *b == 0) || (a[0] == *b && !a[1..].iter().any(|i| *i != 0))
            },
            (Self::Big(a), Self::Big(b)) => {
                !a.iter().zip(b.iter()).any(|(a, b)| a != b) && (match a.len().cmp(&b.len()) {
                    Ordering::Equal => true,
                    Ordering::Less => !b[a.len()..].iter().any(|i| *i != 0),
                    Ordering::Greater => !a[b.len()..].iter().any(|i| *i != 0),
                })
            },
        }
    }
}

impl Eq for BigUint {}

impl Ord for BigUint {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (BigUint::Small(a), BigUint::Small(b)) => a.cmp(b),
            (BigUint::Small(a), BigUint::Big(b)) => {
                if b.len() == 0 {
                    a.cmp(&0)
                } else if b[1..].iter().any(|i| *i != 0) {
                    Ordering::Less
                } else {
                    a.cmp(&b[0])
                }
            },
            (BigUint::Big(_), BigUint::Small(_)) => other.cmp(self).reverse(),
            (BigUint::Big(a), BigUint::Big(b)) => {
                let max_len = a.len().max(b.len());

                // NOTE: can't use `rev()` here because it doesn't impl the required traits.
                let mut ret = Ordering::Equal;
                for (a, b) in core::iter::zip(
                    a.iter().chain(core::iter::repeat(&0)),
                    b.iter().chain(core::iter::repeat(&0)),
                )
                    .take(max_len)
                {
                    let ord = a.cmp(b);
                    if ord != Ordering::Equal { ret = ord };
                }

                ret
            },
        }
    }
}

impl PartialOrd for BigUint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[test]
fn test_eq() {
    let a = BigUint::Big(vec![12, 34]);
    let b = BigUint::Big(vec![12, 34, 0, 0]);
    assert_eq!(a, b);
    assert_eq!(b, a);
    assert_eq!(a, a);
    assert_eq!(b, b);

    let a = BigUint::Big(vec![12, 0, 0]);
    let b = BigUint::Small(12);
    assert_eq!(a, b);
    assert_eq!(b, a);
    assert_eq!(a, a);
    assert_eq!(b, b);
}
