use rand::RngCore;
use std::{
    iter::Cycle,
    vec::IntoIter,
};

pub struct MockRng {
    cycle: Cycle<IntoIter<u8>>,
}

impl MockRng {
    pub fn new(data: impl Into<Vec<u8>>) -> Self {
        let cycle = data.into()
            .into_iter()
            .cycle();
        Self { cycle }
    }

    pub fn next(&mut self) -> u8 {
        self.cycle
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
        let mut rng = MockRng::new("0");
        assert_eq!(rng.gen::<i8>(), 48);
        assert_eq!(rng.gen::<i64>(), 3472328296227680304);
        assert_eq!(rng.gen::<i128>(), 64053151420411946063694043751862251568);

        let mut slice: [u8; 3] = [0, 0, 0];
        rng.try_fill_bytes(slice.as_mut_slice())
            .expect("infallable");
        assert_eq!(slice, [48, 48, 48]);

        let mut zero = MockRng::new("");
        assert_eq!(zero.gen::<i64>(), 0);
    }
}
