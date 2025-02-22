// https://doc.rust-lang.org/src/core/num/int_macros.rs.html
pub trait CarryOps: Sized {
    fn _carrying_add(self, rhs: Self, carry: bool) -> (Self, bool);
}

macro_rules! carry_ops {
    ($ty:ty) => {
        impl CarryOps for $ty {
            #[inline]
            fn _carrying_add(self, rhs: Self, carry: bool) -> (Self, bool) {
                let (a, b) = self.overflowing_add(rhs);
                let (c, d) = a.overflowing_add(carry as _);
                (c, b != d)
            }
        }
    };
}

carry_ops!(usize);
