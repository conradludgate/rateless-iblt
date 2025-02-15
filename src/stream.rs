use std::{cmp::Reverse, collections::BinaryHeap};

use zerocopy::{little_endian, FromBytes, Immutable, IntoBytes};

use crate::{hash, IndexGenerator, Symbol};

#[derive(Default, Clone)]
pub struct SetStream<T> {
    entries: Vec<T>,
    heap: BinaryHeap<Reverse<Entry>>,
}

impl<T: FromBytes + IntoBytes + Immutable + Copy> IntoIterator for SetStream<T> {
    type Item = Symbol<T>;
    type IntoIter = SetStreamIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        SetStreamIter {
            entries: self.entries,
            size: self.heap.len(),
            heap: self.heap,
            index: 0,
        }
    }
}

impl<T: FromBytes + IntoBytes + Immutable + Copy> Extend<T> for SetStream<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let len = self.entries.len();
        self.entries.extend(iter);
        for (index, value) in self.entries[len..].iter().enumerate() {
            let checksum = hash(value.as_bytes());
            self.heap.push(Reverse(Entry {
                index,
                checksum,
                index_rng: IndexGenerator::new(checksum),
            }));
        }
    }
}

impl<T: FromBytes + IntoBytes + Immutable + Copy> SetStreamIter<T> {
    pub(crate) fn push_unchecked(
        &mut self,
        value: T,
        checksum: [u8; 16],
        index_rng: IndexGenerator,
    ) {
        let index = self.entries.len();
        self.heap.push(Reverse(Entry {
            index,
            checksum,
            index_rng,
        }));
        self.entries.push(value);
    }

    pub(crate) fn must_next(&mut self) -> Symbol<T> {
        let mut s = Symbol::default();

        while let Some(mut peek) = self.heap.peek_mut() {
            if peek.0.index_rng.current() > self.index {
                break;
            }

            s += Symbol {
                sum: self.entries[peek.0.index],
                checksum: peek.0.checksum,
                count: little_endian::I64::new(1),
            };

            peek.0.index_rng.next();
        }

        self.index += 1;
        s
    }
}

#[derive(Debug, Clone)]
struct Entry {
    index: usize,
    checksum: [u8; 16],

    index_rng: IndexGenerator,
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.index_rng.current() == other.index_rng.current()
    }
}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.index_rng.current().cmp(&other.index_rng.current())
    }
}

impl Eq for Entry {}

pub struct SetStreamIter<T> {
    pub(crate) entries: Vec<T>,
    heap: BinaryHeap<Reverse<Entry>>,
    index: u64,
    size: usize,
}

impl<T> Default for SetStreamIter<T> {
    fn default() -> Self {
        Self {
            entries: Default::default(),
            heap: Default::default(),
            index: Default::default(),
            size: Default::default(),
        }
    }
}

impl<T: FromBytes + IntoBytes + Immutable + Copy> Iterator for SetStreamIter<T> {
    type Item = Symbol<T>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.must_next())
    }
}
