#![no_std]

#[cfg_attr(test, macro_use)]
extern crate alloc;

#[cfg(test)]
extern crate std;

mod binaryheap;
mod decoder;
mod encoder;
mod index;
mod symbol;

pub use decoder::{set_difference, Decoder};
pub use encoder::{Encoder, EncoderIter};
pub use symbol::Symbol;
use zerocopy::{FromBytes, Immutable, IntoBytes};

fn hash(x: &[u8]) -> [u8; 16] {
    blake3::hash(x).as_bytes()[..16].try_into().unwrap()
}

fn xor_mut<T: FromBytes + IntoBytes + Immutable>(a: &mut T, b: &T) {
    for (a, b) in core::iter::zip(a.as_mut_bytes(), b.as_bytes()) {
        *a ^= *b;
    }
}

#[cfg(test)]
mod tests {
    use rand_core::{RngCore, SeedableRng};
    use rand_xoshiro::Xoshiro256StarStar;

    use crate::{set_difference, Encoder};

    #[test]
    fn works() {
        let mut remote = Encoder::default();
        remote.extend([1, 2, 3, 4]);

        let mut local = Encoder::default();
        local.extend([1, 2, 3, 5]);

        let (remote, local) = set_difference(remote.into_iter().take(4), local).unwrap();
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
        assert_eq!(remote, vec![7, 4, 10]);
        assert_eq!(local, vec![5, 9, 6])
    }

    #[test]
    fn works_bigger_less_diff() {
        let mut remote = Encoder::default();
        remote.extend([1, 2, 3, 4, 6, 7, 8, 9, 10]);

        let mut local = Encoder::default();
        local.extend([1, 2, 3, 4, 5, 6, 7, 8, 10]);

        let (remote, local) = set_difference(remote.into_iter().take(4), local).unwrap();
        assert_eq!(remote, vec![9]);
        assert_eq!(local, vec![5]);
    }

    #[test]
    #[ignore = "very slow"]
    fn huge() {
        const N: u64 = 10_000_000;
        const M: u64 = 8;

        let mut rng = Xoshiro256StarStar::seed_from_u64(0);

        let mut remote = Encoder::default();
        let mut local = Encoder::default();

        for _ in 0..N {
            let i = rng.next_u64();
            if i < M * 2 {
                continue;
            }
            remote.extend([i]);
            local.extend([i]);
        }
        for i in 0..M {
            remote.extend([i]);
        }
        for j in M..2 * M {
            local.extend([j]);
        }

        let (tx, remote_rx) = std::sync::mpsc::sync_channel(1);
        std::thread::spawn(move || {
            let iter = remote.into_iter().take(5 * M as usize);
            for entry in iter {
                if tx.send(entry).is_err() {
                    break;
                }
            }
        });

        let (tx, local_rx) = std::sync::mpsc::sync_channel(1);
        std::thread::spawn(move || {
            let iter = local;
            for entry in iter {
                if tx.send(entry).is_err() {
                    break;
                }
            }
        });

        let (remote, local) = set_difference(remote_rx, local_rx).unwrap();
        assert_eq!(remote.len(), M as usize);
        assert_eq!(local.len(), M as usize);
    }
}
