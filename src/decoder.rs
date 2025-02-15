use alloc::vec::Vec;
use bitvec::vec::BitVec;
use zerocopy::{FromBytes, Immutable, IntoBytes};

use crate::{EncoderIter, IndexGenerator, Symbol};

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
    candidates: BitVec,
}

impl<T> Default for Decoder<T> {
    fn default() -> Self {
        Self {
            remote: Default::default(),
            local: Default::default(),
            symbols: Default::default(),
            candidates: BitVec::new(),
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

        self.candidates.push(cell.is_pure_cell());
        self.symbols.push(cell);

        loop {
            let Some(i) = self.candidates.last_one() else {
                break;
            };

            let symbol = self.symbols[i];
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
                self.candidates.set(j, s.is_pure_cell());
                index.next();
            }

            if symbol.count.get().is_positive() {
                self.remote
                    .push_unchecked(symbol.sum, symbol.checksum, index);
            } else {
                self.local
                    .push_unchecked(symbol.sum, symbol.checksum, index);
            }
        }
    }
}
