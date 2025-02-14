use std::{cmp::Reverse, collections::BinaryHeap};

use rand::{rngs::SmallRng, SeedableRng};
use zerocopy::little_endian;

use crate::{hash, next_index, Symbol};

#[derive(Default, Clone)]
pub struct SetStream {
    heap: BinaryHeap<Reverse<Entry>>,
}

impl IntoIterator for SetStream {
    type Item = Symbol;
    type IntoIter = SetStreamIter;

    fn into_iter(self) -> Self::IntoIter {
        SetStreamIter {
            index: 0,
            size: self.heap.len(),
            heap: self.heap,
        }
    }
}

impl Extend<[u8; 32]> for SetStream {
    fn extend<T: IntoIterator<Item = [u8; 32]>>(&mut self, iter: T) {
        for value in iter {
            let hash = hash(value);
            self.heap.push(Reverse(Entry {
                next: 0,
                value,
                hash,
                rng: SmallRng::from_seed(hash),
            }));
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Entry {
    pub(crate) next: u64,
    pub(crate) value: [u8; 32],
    pub(crate) hash: [u8; 32],
    pub(crate) rng: SmallRng,
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.next == other.next
    }
}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.next.cmp(&other.next)
    }
}

impl Eq for Entry {}

#[derive(Default)]
pub struct SetStreamIter {
    pub(crate) heap: BinaryHeap<Reverse<Entry>>,
    index: u64,
    size: usize,
}

impl Iterator for SetStreamIter {
    type Item = Symbol;

    fn next(&mut self) -> Option<Self::Item> {
        let mut s = Symbol::default();

        while let Some(mut peek) = self.heap.peek_mut() {
            if peek.0.next > self.index {
                break;
            }

            s += Symbol {
                value: peek.0.value,
                hash: peek.0.hash,
                count: little_endian::I64::new(1),
            };

            peek.0.next = next_index(peek.0.next, &mut peek.0.rng);
        }

        self.index += 1;
        Some(s)
    }
}
