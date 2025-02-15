use rand_core::{RngCore, SeedableRng};

#[derive(Debug, Clone)]
pub(crate) struct IndexGenerator {
    rng: rand_xoshiro::Xoroshiro128Plus,
    index: u64,
}

impl IndexGenerator {
    pub(crate) fn new(checksum: [u8; 16]) -> Self {
        Self {
            rng: SeedableRng::from_seed(checksum),
            index: 0,
        }
    }

    pub(crate) fn current(&self) -> u64 {
        self.index
    }

    pub(crate) fn next(&mut self) {
        self.index += libm::ceil(c_inv(self.index as f64, self.rng.next_u64())) as u64
    }
}

fn c_inv(i: f64, r: u64) -> f64 {
    const U: f64 = (1u64 << 32) as f64;
    (i + 1.5) * (U / libm::sqrt(r as f64) - 1.0)
}

#[cfg(test)]
mod tests {
    use zerocopy::IntoBytes;

    use crate::hash;

    use super::*;
    use alloc::collections::BTreeMap;

    fn p(i: f64) -> f64 {
        (1.0 + 0.5 * i).recip()
    }

    #[test]
    fn test_distribution() {
        let mut map = BTreeMap::<u64, u64>::new();
        const N: u64 = 100000;
        const L: u64 = 1000;

        for i in 0..N {
            let mut gen = IndexGenerator::new(hash(i.as_bytes()));
            while gen.current() < L {
                *map.entry(gen.current()).or_default() += 1;
                gen.next();
            }
        }

        let mut ecdf = 0;
        let mut cdf = 0.0;
        let mut max = 0.0;
        for (j, c) in map {
            ecdf += c;
            cdf += p(j as f64) * (N as f64);
            max = f64::max(max, (cdf - ecdf as f64).abs());
        }

        let p = max / (N as f64);
        assert!(p < 0.06, "{p}");
    }
}
