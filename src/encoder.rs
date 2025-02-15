use alloc::vec::Vec;

use zerocopy::{FromBytes, Immutable, IntoBytes};

use crate::{binaryheap, hash, IndexGenerator, Symbol};

#[derive(Default, Clone)]
pub struct Encoder<T> {
    entries: Vec<T>,
}

impl<T: FromBytes + IntoBytes + Immutable + Copy> IntoIterator for Encoder<T> {
    type Item = Symbol<T>;
    type IntoIter = EncoderIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        let mut heap = Vec::with_capacity(self.entries.len());
        for (entry_index, value) in self.entries.iter().enumerate() {
            let checksum = hash(value.as_bytes());
            heap.push(Entry {
                index: IndexGenerator::new(checksum),
                entry_index,
                checksum,
            });
        }

        EncoderIter {
            entries: self.entries,
            heap,
            index: 0,
        }
    }
}

impl<T: IntoBytes + Immutable> Extend<T> for Encoder<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.entries.extend(iter);
    }
}

#[derive(Debug, Clone)]
struct Entry {
    index: IndexGenerator,
    entry_index: usize,
    checksum: [u8; 16],
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(Ord::cmp(self, other))
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.index.current() == other.index.current()
    }
}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        Ord::cmp(&self.index.current(), &other.index.current()).reverse()
    }
}

impl Eq for Entry {}

pub struct EncoderIter<T> {
    pub(crate) entries: Vec<T>,
    heap: Vec<Entry>,
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

impl<T: IntoBytes + Immutable> EncoderIter<T> {
    pub(crate) fn push_unchecked(&mut self, value: T, checksum: [u8; 16], index: IndexGenerator) {
        let entry_index = self.entries.len();
        self.heap.push(Entry {
            entry_index,
            checksum,
            index,
        });
        binaryheap::sift_up(&mut self.heap, 0, entry_index);
        self.entries.push(value);
    }
}

impl<T: FromBytes + IntoBytes + Immutable> EncoderIter<T> {
    fn threshold(&self) -> u64 {
        if self.entries.len() < 2 {
            return 0;
        }

        // based on the intersection of
        // * y = n (linear search)
        // * y = p(x) * log2(n) (binary heap search)
        // solution: p(x) = n/(1+0.5x),
        //           x = 2log2(n) - 2
        u64::from(usize::ilog2(self.entries.len())) * 2
    }

    #[cold]
    fn update_many(&mut self) -> Symbol<T> {
        let mut s = Symbol::default();

        for p in self.heap.iter_mut() {
            if p.index.current() > self.index {
                continue;
            }

            s.add_entry(&self.entries[p.entry_index], &p.checksum);
            p.index.next();
        }

        // only build the binary heap when it's time to switch strategy
        if self.index == self.threshold() {
            binaryheap::rebuild(&mut self.heap);
        }

        s
    }

    fn update_few(&mut self) -> Symbol<T> {
        let mut s = Symbol::default();

        while let Some(p) = self.heap.first_mut() {
            if p.index.current() > self.index {
                break;
            }

            s.add_entry(&self.entries[p.entry_index], &p.checksum);
            p.index.next();
            binaryheap::sift_down(&mut self.heap, 0);
        }

        s
    }

    pub(crate) fn must_next(&mut self) -> Symbol<T> {
        let s = if self.index <= self.threshold() {
            self.update_many()
        } else {
            self.update_few()
        };

        self.index += 1;
        s
    }
}

impl<T: FromBytes + IntoBytes + Immutable + Copy> Iterator for EncoderIter<T> {
    type Item = Symbol<T>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.must_next())
    }
}
