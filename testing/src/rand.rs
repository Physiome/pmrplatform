use rand::RngCore;
use std::{
    iter::Cycle,
    vec::IntoIter,
};

/// A crappy Rng that xors two different cycles of `u8`s together.  If
/// they are of different prime number lengths (or at the very least
/// lengths that are not a factor of the other) they may have a slightly
/// longer cycle than just working with one cycle.  Only suitable for
/// generating test values, hilariously unsuitable for anything that
/// requires any notion of security.
pub struct MockRng {
    cycle1: Cycle<IntoIter<u8>>,
    cycle2: Cycle<IntoIter<u8>>,
}

impl MockRng {
    pub fn default() -> Self {
        Self::new(
            "the quick brown fox jumps over the lazy fox",
            [204, 240, 170, 85, 15, 51, 219],
        )
    }

    pub fn new(data1: impl Into<Vec<u8>>, data2: impl Into<Vec<u8>>) -> Self {
        let cycle1 = data1.into()
            .into_iter()
            .cycle();
        let cycle2 = data2.into()
            .into_iter()
            .cycle();
        Self { cycle1, cycle2 }
    }

    pub fn next(&mut self) -> u8 {
        self.cycle1
            .next()
            .unwrap_or(0) ^
        self.cycle2
            .next()
            .unwrap_or(0)
    }
}

impl RngCore for MockRng {
    fn next_u32(&mut self) -> u32 {
        u32::from(self.next()) << 24 |
        u32::from(self.next()) << 16 |
        u32::from(self.next()) << 8 |
        u32::from(self.next())
    }

    fn next_u64(&mut self) -> u64 {
        u64::from(self.next_u32()) << 32 |
        u64::from(self.next_u32())
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        dest.iter_mut()
            .for_each(|x| *x = self.next())
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;
    use super::*;

    #[test]
    fn smoke() {
        let mut rng = MockRng::new("0", "");
        assert_eq!(rng.gen::<i8>(), 48);
        assert_eq!(rng.gen::<i32>(), 808464432);
        assert_eq!(rng.gen::<i64>(), 3472328296227680304);
        assert_eq!(rng.gen::<i128>(), 64053151420411946063694043751862251568);

        let mut slice: [u8; 3] = [0, 0, 0];
        rng.try_fill_bytes(slice.as_mut_slice())
            .expect("infallable");
        assert_eq!(slice, [48, 48, 48]);

        let mut zero = MockRng::new("", "");
        assert_eq!(zero.gen::<i64>(), 0);
        assert_eq!(zero.gen::<i64>(), 0);

        let mut one = MockRng::new("0", "1");
        assert_eq!(one.gen::<i8>(), 1);
        assert_eq!(one.gen::<i8>(), 1);
        assert_eq!(one.gen::<i32>(), 16843009);

        // sequence repeats in 15 (product of lengths of the two inputs).
        let mut xor = MockRng::new([45, 76, 182], [251, 189, 49, 128, 7]);
        let mut repeats = [0u8; 30];
        xor.fill_bytes(repeats.as_mut_slice());
        assert_eq!(repeats[..15], repeats[15..])
    }
}
