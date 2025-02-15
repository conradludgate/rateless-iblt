use alloc::vec::Vec;
use zerocopy::{FromBytes, Immutable, IntoBytes};

use crate::{binaryheap, EncoderIter, IndexGenerator, Symbol};

pub fn set_difference<T: FromBytes + IntoBytes + Immutable + Copy>(
    remote: impl IntoIterator<Item = Symbol<T>>,
    local: impl IntoIterator<Item = Symbol<T>>,
) -> Option<(Vec<T>, Vec<T>)> {
    let mut decoder = Decoder::default();

    let mut a = remote.into_iter();
    let mut b = local.into_iter();

    loop {
        decoder.push(a.next()?, b.next()?);
        if decoder.is_complete() {
            return Some(decoder.consume());
        }
    }
}

pub struct Decoder<T> {
    remote: EncoderIter<T>,
    local: EncoderIter<T>,
    symbols: Vec<Symbol<T>>,
    pure_heap: Vec<usize>,
}

impl<T> Default for Decoder<T> {
    fn default() -> Self {
        Self {
            remote: Default::default(),
            local: Default::default(),
            symbols: Default::default(),
            pure_heap: Vec::new(),
        }
    }
}

impl<T: FromBytes + IntoBytes + Immutable + Copy> Decoder<T> {
    pub fn is_complete(&self) -> bool {
        !self.symbols.is_empty() && self.symbols[0].is_empty_cell()
    }

    pub fn consume(self) -> (Vec<T>, Vec<T>) {
        (self.remote.entries, self.local.entries)
    }

    pub fn push(&mut self, remote: Symbol<T>, local: Symbol<T>) {
        let cell = remote - local - self.remote.must_next() + self.local.must_next();

        if cell.is_pure_cell() {
            self.pure_heap.push(self.symbols.len());
        }
        self.symbols.push(cell);

        while !self.pure_heap.is_empty() {
            let i = self.pure_heap.swap_remove(0);
            binaryheap::sift_down(&mut self.pure_heap, 0);

            let symbol = self.symbols[i];
            if !symbol.is_pure_cell() {
                continue;
            }

            // peel off this cell in all indices
            let mut index = IndexGenerator::new(symbol.checksum);
            loop {
                let Ok(j) = usize::try_from(index.current()) else {
                    break;
                };
                let Some(s) = self.symbols.get_mut(j) else {
                    break;
                };

                *s -= symbol;
                if s.is_pure_cell() {
                    let old_index = self.pure_heap.len();
                    self.pure_heap.push(j);
                    binaryheap::sift_up(&mut self.pure_heap, 0, old_index);
                }

                index.next();
            }

            let count = symbol.count.get();
            if count == 1 {
                self.remote
                    .push_unchecked(symbol.sum, symbol.checksum, index);
            } else {
                self.local
                    .push_unchecked(symbol.sum, symbol.checksum, index);
            }
        }
    }
}
