use std::{cmp::Reverse, collections::BinaryHeap};

use rand::{rngs::SmallRng, SeedableRng};
use zerocopy::{little_endian, FromBytes, Immutable, IntoBytes};

use crate::{hash, next_index, Symbol};

#[derive(Default, Clone)]
pub struct SetStream<T> {
    heap: BinaryHeap<Reverse<Entry<T>>>,
}

impl<T: FromBytes + IntoBytes + Immutable + Copy> IntoIterator for SetStream<T> {
    type Item = Symbol<T>;
    type IntoIter = SetStreamIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        SetStreamIter {
            index: 0,
            size: self.heap.len(),
            heap: self.heap,
        }
    }
}

impl<T: FromBytes + IntoBytes + Immutable + Copy> Extend<T> for SetStream<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for value in iter {
            let hash = hash(value.as_bytes());
            self.heap.push(Reverse(Entry {
                next: 0,
                value,
                checksum: hash,
                rng: SmallRng::from_seed(hash),
            }));
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Entry<T> {
    pub(crate) next: u64,
    pub(crate) value: T,
    pub(crate) checksum: [u8; 32],
    pub(crate) rng: SmallRng,
}

impl<T> PartialOrd for Entry<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> PartialEq for Entry<T> {
    fn eq(&self, other: &Self) -> bool {
        self.next == other.next
    }
}

impl<T> Ord for Entry<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.next.cmp(&other.next)
    }
}

impl<T> Eq for Entry<T> {}

#[derive(Default)]
pub struct SetStreamIter<T> {
    pub(crate) heap: BinaryHeap<Reverse<Entry<T>>>,
    pub(crate) index: u64,
    pub(crate) size: usize,
}

impl<T: FromBytes + IntoBytes + Immutable + Copy> Iterator for SetStreamIter<T> {
    type Item = Symbol<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut s = Symbol::default();

        while let Some(mut peek) = self.heap.peek_mut() {
            if peek.0.next > self.index {
                break;
            }

            s += Symbol {
                sum: peek.0.value,
                checksum: peek.0.checksum,
                count: little_endian::I64::new(1),
            };

            peek.0.next = next_index(peek.0.next, &mut peek.0.rng);
        }

        self.index += 1;
        Some(s)
    }
}
