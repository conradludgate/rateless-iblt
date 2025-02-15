#![no_std]

#[cfg_attr(test, macro_use)]
extern crate alloc;

use sha2::Digest;

mod decoder;
mod encoder;
mod index;
mod symbol;

pub use decoder::set_difference;
pub use encoder::{Encoder, EncoderIter};
pub(crate) use index::IndexGenerator;
pub use symbol::Symbol;
use zerocopy::{FromBytes, Immutable, IntoBytes};

fn hash(x: &[u8]) -> [u8; 16] {
    sha2::Sha256::digest(x)[..16].try_into().unwrap()
}

fn xor_mut<T: FromBytes + IntoBytes + Immutable>(a: &mut T, b: &T) {
    for (a, b) in core::iter::zip(a.as_mut_bytes(), b.as_bytes()) {
        *a ^= *b;
    }
}

#[cfg(test)]
mod tests {
    use crate::{set_difference, Encoder};

    #[test]
    fn works() {
        let mut remote = Encoder::default();
        remote.extend([1, 2, 3, 4]);

        let mut local = Encoder::default();
        local.extend([1, 2, 3, 5]);

        let (remote, local) = set_difference(remote.into_iter().take(3), local).unwrap();
        assert_eq!(remote, vec![4]);
        assert_eq!(local, vec![5]);
    }

    #[test]
    fn works_bigger() {
        let mut remote = Encoder::default();
        remote.extend([1, 2, 3, 4, 7, 8, 10]);

        let mut local = Encoder::default();
        local.extend([1, 2, 3, 5, 6, 8, 9]);

        let (remote, local) = set_difference(remote.into_iter().take(12), local).unwrap();
        assert_eq!(remote, vec![4, 7, 10]);
        assert_eq!(local, vec![6, 9, 5])
    }

    #[test]
    fn works_bigger_less_diff() {
        let mut remote = Encoder::default();
        remote.extend([1, 2, 3, 4, 6, 7, 8, 9, 10]);

        let mut local = Encoder::default();
        local.extend([1, 2, 3, 4, 5, 6, 7, 8, 10]);

        let (remote, local) = set_difference(remote.into_iter().take(2), local).unwrap();
        assert_eq!(remote, vec![9]);
        assert_eq!(local, vec![5]);
    }
}
