use core::fmt::*;
use crate::BigUint;

impl Debug for BigUint {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Small(v) => write!(f, "Small({v})"),
            Self::Big(v) => {
                write!(f, "Big(")?;
                for (i, v) in v.iter().enumerate().rev() {
                    if i == 0 {
                        write!(f, "{v}")?;
                    } else {
                        write!(f, "{v}_")?;
                    }
                }
                write!(f, ")")
            },
        }
    }
}

impl Display for BigUint {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Small(v) => Display::fmt(v, f),
            Self::Big(v) => {
                let mut bcd = vec![0; v.len() * 64 / 4 + (v.len() * 64 + 2) / 3];

                for i in v.iter().rev() {
                    for bit in 0..64 {
                        for d in bcd.iter_mut() {
                            if *d >= 5 { *d += 3 }
                        }

                        let mut carry = (i >> (63 - bit)) & 1 != 0;
                        for d in bcd.iter_mut() {
                            let nc = (*d >> 3) != 0;
                            *d = ((*d << 1) | carry as u8) & 0xf;
                            carry = nc;
                        }
                    }
                }

                for d in bcd.into_iter().rev().skip_while(|i| *i == 0) {
                    write!(f, "{}", (b'0' + d) as char)?;
                }

                Ok(())
            },
        }
    }
}

#[test]
fn test_display() {
    assert_eq!(&BigUint::Small(123).to_string(), "123");
    assert_eq!(&BigUint::Big(vec![243]).to_string(), "243");
    assert_eq!(&BigUint::Big(vec![0, 1]).to_string(), "18446744073709551616");
}
