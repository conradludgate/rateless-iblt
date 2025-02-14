use rand::Rng;
use sha2::Digest;

mod diff;
mod stream;
mod symbol;

pub use diff::set_difference;
pub use stream::{SetStream, SetStreamIter};
pub use symbol::Symbol;
use zerocopy::{FromBytes, Immutable, IntoBytes};

fn hash(x: &[u8]) -> [u8; 32] {
    sha2::Sha256::digest(x).into()
}

fn next_index(i: u64, rng: &mut impl Rng) -> u64 {
    i + c_inv(i as f64, rng.random()).ceil() as u64
}

fn c_inv(i: f64, r: u64) -> f64 {
    (i + 1.5) * (f64::powi(2.0, 32) / f64::sqrt(r as f64) - 1.0)
}

fn p(i: f64) -> f64 {
    (1.0 + 0.5 * i).recip()
}

fn xor_mut<T: FromBytes + IntoBytes + Immutable>(a: &mut T, b: &T) {
    for (a, b) in std::iter::zip(a.as_mut_bytes(), b.as_bytes()) {
        *a ^= *b;
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use rand::{rngs::SmallRng, Rng, SeedableRng};

    use crate::{c_inv, diff::Delta, p, set_difference, SetStream};

    #[test]
    fn test_distribution() {
        let mut map = BTreeMap::<u64, u64>::new();
        const N: u64 = 100000;
        const L: u64 = 1000;

        for i in 0..N {
            let mut j = 0;
            let mut rng = SmallRng::seed_from_u64(i);
            while j < L {
                *map.entry(j).or_default() += 1;
                // j += c_inv(j as f64, uniform().sample(&mut rng)).ceil() as u64;
                j += c_inv(j as f64, rng.random()).ceil() as u64;
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

    #[test]
    fn works() {
        let mut alice = SetStream::default();
        alice.extend([1, 2, 3, 4]);

        let mut bob = SetStream::default();
        bob.extend([1, 2, 3, 5]);

        let diff = set_difference(alice.into_iter().take(4), bob).unwrap();
        assert_eq!(diff, vec![Delta::InRemote(4), Delta::InLocal(5)])
    }

    #[test]
    fn works_bigger() {
        let mut alice = SetStream::default();
        alice.extend([1, 2, 3, 4, 7, 8, 10]);

        let mut bob = SetStream::default();
        bob.extend([1, 2, 3, 5, 6, 8, 9]);

        let diff = set_difference(alice.into_iter().take(12), bob).unwrap();
        assert_eq!(
            diff,
            vec![
                Delta::InRemote(4),
                Delta::InRemote(7),
                Delta::InRemote(10),
                Delta::InLocal(6),
                Delta::InLocal(5),
                Delta::InLocal(9),
            ]
        )
    }

    #[test]
    fn works_bigger_less_diff() {
        let mut alice = SetStream::default();
        alice.extend([1, 2, 3, 4, 6, 7, 8, 9, 10]);

        let mut bob = SetStream::default();
        bob.extend([1, 2, 3, 4, 5, 6, 7, 8, 10]);

        let diff = set_difference(alice.into_iter().take(2), bob).unwrap();
        assert_eq!(diff, vec![Delta::InRemote(9), Delta::InLocal(5)])
    }
}
