// This file prematurealy adds `Duration::saturating_sub`, while keeping us within stable rust.
// It also extends `Duration` with functionality which isn't even present in stable rust.
use std::time::Duration;

pub trait DurationExt {
    // duration_saturating_ops #76416
    fn my_saturating_sub(self, other : Self) -> Self;
    // duration_zero #73544
    fn my_is_zero(&self) -> bool;
    fn my_zero() -> Self;
    // reminder operator. No RFCs
    fn my_rem(self, other : Self) -> Self;
}

mod my_rem_impl {
    use std::time::Duration;

    pub fn my_rem_impl_soft(me : Duration, other : Duration) -> Duration {
        use super::DurationExt;

        assert!(!other.my_is_zero(), "`other` can't be zero");

        let mut buff = me;
        while buff >= other { buff -= other; }
        buff
    }

    pub fn my_rem_impl_u128(me : Duration, other : Duration) -> Duration {
        use super::DurationExt;

        const NANOS_PER_SEC : u128 = 1_000_000_000;

        assert!(!other.my_is_zero(), "`other` can't be zero");

        let me = me.as_nanos();
        let other = other.as_nanos();
        let res = me % other;
        Duration::new((res / NANOS_PER_SEC) as u64, (res % NANOS_PER_SEC) as u32)
    }
}

impl DurationExt for Duration {
    fn my_zero() -> Duration { Duration::new(0, 0) }

    fn my_saturating_sub(self, other : Duration) -> Duration {
        self.checked_sub(other).unwrap_or(<Duration as DurationExt>::my_zero())
    }

    fn my_is_zero(&self) -> bool {
        *self == <Duration as DurationExt>::my_zero()
    }

    fn my_rem(self, other : Duration) -> Duration {
        my_rem_impl::my_rem_impl_u128(self, other)
    }
}

#[cfg(test)]
mod duration_ext_tests {
    pub use super::DurationExt;
    pub use std::time::Duration;
    pub use super::my_rem_impl::*;

    #[test]
    fn my_zero_test() {
        let zero = <Duration as DurationExt>::my_zero();
        // is it actually zero
        assert_eq!(zero + zero, zero);
    }

    #[test]
    fn my_is_zero_test() {
        assert!(<Duration as DurationExt>::my_zero().my_is_zero());
        assert!(!Duration::new(1, 0).my_is_zero());
    }

    #[test]
    fn my_rem_same() {
        let left = Duration::new(40, 20);
        let right = Duration::new(2, 5);

        // Both implementations must produce the same result
        assert_eq!(my_rem_impl_soft(left, right), my_rem_impl_u128(left, right));
    }

    #[cfg(feature = "bench")]
    mod benches {
        use super::*;

        use test::Bencher;

        #[bench]
        fn my_rem_speed_soft(b : &mut Bencher) {
            use super::super::my_rem_impl::my_rem_impl_soft;
        
            let mut x : u64 = 0;
            b.iter(
                || 
                x = x.wrapping_add(
                    my_rem_impl_soft(Duration::new(4000000, 20), Duration::new(2, 1)).as_secs()
                )
            );
            println!("{}", x);
        }
    
        #[bench]
        fn my_rem_speed_u128(b : &mut Bencher) {
            use super::super::my_rem_impl::my_rem_impl_u128;
        
            let mut x : u64 = 0;
            b.iter(
                || 
                x = x.wrapping_add(
                    my_rem_impl_u128(Duration::new(4000000, 20), Duration::new(2, 1)).as_secs()
                )
            );
            println!("{}", x);
        }
    }
}
