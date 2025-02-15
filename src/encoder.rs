use alloc::{collections::BinaryHeap, vec::Vec};

use zerocopy::{FromBytes, Immutable, IntoBytes};

use crate::{hash, IndexGenerator, Symbol};

#[derive(Default, Clone)]
pub struct Encoder<T> {
    entries: Vec<T>,
    heap: BinaryHeap<Entry>,
}

impl<T: FromBytes + IntoBytes + Immutable + Copy> IntoIterator for Encoder<T> {
    type Item = Symbol<T>;
    type IntoIter = EncoderIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        EncoderIter {
            entries: self.entries,
            heap: self.heap,
            index: 0,
        }
    }
}

impl<T: IntoBytes + Immutable> Extend<T> for Encoder<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let len = self.entries.len();
        self.entries.extend(iter);
        for (index, value) in self.entries[len..].iter().enumerate() {
            let checksum = hash(value.as_bytes());
            self.heap.push(Entry {
                index: index + len,
                checksum,
                index_rng: IndexGenerator::new(checksum),
            });
        }
    }
}

impl<T: IntoBytes + Immutable> EncoderIter<T> {
    pub(crate) fn push_unchecked(
        &mut self,
        value: T,
        checksum: [u8; 16],
        index_rng: IndexGenerator,
    ) {
        let index = self.entries.len();
        self.heap.push(Entry {
            index,
            checksum,
            index_rng,
        });
        self.entries.push(value);
    }
}

impl<T: FromBytes + IntoBytes + Immutable> EncoderIter<T> {
    pub(crate) fn must_next(&mut self) -> Symbol<T> {
        let mut s = Symbol::default();

        while let Some(mut p) = self.heap.peek_mut() {
            if p.index_rng.current() > self.index {
                break;
            }

            s.add_entry(&self.entries[p.index], &p.checksum);
            p.index_rng.next();
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
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.index_rng.current() == other.index_rng.current()
    }
}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.index_rng
            .current()
            .cmp(&other.index_rng.current())
            .reverse()
    }
}

impl Eq for Entry {}

pub struct EncoderIter<T> {
    pub(crate) entries: Vec<T>,
    heap: BinaryHeap<Entry>,
    index: u64,
}

impl<T> Default for EncoderIter<T> {
    fn default() -> Self {
        Self {
            entries: Default::default(),
            heap: Default::default(),
            index: Default::default(),
        }
    }
}

impl<T: FromBytes + IntoBytes + Immutable + Copy> Iterator for EncoderIter<T> {
    type Item = Symbol<T>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.must_next())
    }
}
