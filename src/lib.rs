use rand::{
    distr::{Distribution, Uniform},
    Rng,
};
use sha2::Digest;

mod diff;
mod stream;
mod symbol;

pub use diff::set_difference;
pub use stream::{SetStream, SetStreamIter};
pub use symbol::Symbol;

fn hash(x: [u8; 32]) -> [u8; 32] {
    sha2::Sha256::digest(x).into()
}

fn uniform() -> Uniform<f64> {
    Uniform::new(0.0, 1.0).unwrap()
}

fn next_index(i: u64, rng: &mut impl Rng) -> u64 {
    let r = uniform().sample(rng);
    let g = c_inv(i as f64, r);
    i + g.ceil() as u64
}

fn c_inv(i: f64, r: f64) -> f64 {
    (i + 1.5) * ((1.0 - r).sqrt().recip() - 1.0)
}

fn p(i: f64) -> f64 {
    (1.0 + 0.5 * i).recip()
}

fn xor_mut<const N: usize>(a: &mut [u8; N], b: &[u8; N]) {
    for (a, b) in std::iter::zip(a, b) {
        *a ^= *b;
    }
}

#[cfg(test)]
mod tests {
    use crate::{set_difference, SetStream};

    #[test]
    fn works() {
        let a = [b'a'; 32];
        let b = [b'b'; 32];
        let c = [b'c'; 32];
        let d = [b'd'; 32];
        let e = [b'e'; 32];

        let mut alice = SetStream::default();
        alice.extend([a, b, c, d]);

        let mut bob = SetStream::default();
        bob.extend([a, b, c, e]);

        let diff = set_difference(alice.into_iter().take(2), bob).unwrap();
        assert_eq!(diff, vec![d, e])
    }

    #[test]
    fn works_bigger() {
        let a = [b'a'; 32];
        let b = [b'b'; 32];
        let c = [b'c'; 32];
        let d = [b'd'; 32];
        let e = [b'e'; 32];
        let f = [b'f'; 32];
        let g = [b'g'; 32];
        let h = [b'h'; 32];
        let i = [b'i'; 32];
        let j = [b'j'; 32];

        let mut alice = SetStream::default();
        alice.extend([a, b, c, d, g, h, j]);

        let mut bob = SetStream::default();
        bob.extend([a, b, c, e, f, h, i]);

        let diff = set_difference(alice.into_iter().take(8), bob).unwrap();
        assert_eq!(diff, vec![f, g, i, d, j, e])
    }
}
