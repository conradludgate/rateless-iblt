use alloc::vec::Vec;
use zerocopy::{FromBytes, Immutable, IntoBytes};

use crate::{IndexGenerator, EncoderIter, Symbol};

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
}

impl<T> Default for Decoder<T> {
    fn default() -> Self {
        Self {
            remote: Default::default(),
            local: Default::default(),
            symbols: Default::default(),
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
        let pure = cell.is_pure_cell();

        self.symbols.push(cell);

        if !pure {
            return;
        }

        loop {
            let progress = self.peel_one_layer();
            if progress == 0 {
                break;
            } else if progress < self.symbols.len() {
                continue;
            } else {
                break;
            }
        }
    }

    fn peel_one_layer(&mut self) -> usize {
        let mut progress = self.symbols.len();
        for j in (0..self.symbols.len()).rev() {
            let symbol = self.symbols[j];
            if symbol.is_pure_cell() {
                progress = j;

                // peel off this cell in all indices
                let mut index = IndexGenerator::new(symbol.checksum);
                loop {
                    let i = index.current();
                    let Ok(k) = usize::try_from(i) else { break };
                    let Some(s) = self.symbols.get_mut(k) else {
                        break;
                    };
                    *s -= symbol;
                    index.next();
                }

                if symbol.count.get() == 1 {
                    self.remote
                        .push_unchecked(symbol.sum, symbol.checksum, index);
                } else if symbol.count.get() == -1 {
                    self.local
                        .push_unchecked(symbol.sum, symbol.checksum, index);
                } else {
                    unreachable!()
                }
            }
        }

        progress
    }
}
