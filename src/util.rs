/// Stable counterparts of `bigint_helper_methods` functions. Copied from
/// [the official `std` library](https://doc.rust-lang.org/src/core/num/int_macros.rs.html).
pub trait CarryOps: Sized {
    const ZERO: Self;

    fn _carrying_add(self, rhs: Self, carry: bool) -> (Self, bool);
    fn _borrowing_sub(self, rhs: Self, carry: bool) -> (Self, bool);
    fn _carrying_mul_add(self, rhs: Self, carry: Self, add: Self) -> (Self, Self);

    #[inline]
    fn _carrying_mul(self, rhs: Self, carry: Self) -> (Self, Self) {
        self._carrying_mul_add(rhs, carry, Self::ZERO)
    }
}

macro_rules! carry_ops {
    ($ty:ty, $carry_mul_add:expr) => {
        impl CarryOps for $ty {
            const ZERO: Self = 0;

            #[inline]
            fn _carrying_add(self, rhs: Self, carry: bool) -> (Self, bool) {
                let (a, b) = self.overflowing_add(rhs);
                let (c, d) = a.overflowing_add(carry as _);
                (c, b != d)
            }

            #[inline]
            fn _borrowing_sub(self, rhs: Self, borrow: bool) -> (Self, bool) {
                let (a, b) = self.overflowing_sub(rhs);
                let (c, d) = a.overflowing_sub(borrow as _);
                (c, b != d)
            }

            #[inline]
            fn _carrying_mul_add(self, rhs: Self, carry: Self, add: Self) -> (Self, Self) {
                $carry_mul_add(self, rhs, carry, add)
            }
        }
    };
}

carry_ops!(u64, |a, b, c, add| {
    let r = a as u128 * b as u128 + c as u128 + add as u128;
    (r as u64, (r >> 64) as u64)
});
