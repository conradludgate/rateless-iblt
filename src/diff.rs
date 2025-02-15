use rand::{rngs::SmallRng, SeedableRng};
use zerocopy::{FromBytes, Immutable, IntoBytes};

use crate::{next_index, SetStreamIter, Symbol};

#[derive(PartialEq, Debug)]
pub enum Delta<T> {
    InRemote(T),
    InLocal(T),
}

pub fn set_difference<T: FromBytes + IntoBytes + Immutable + Copy>(
    remote: impl IntoIterator<Item = Symbol<T>>,
    local: impl IntoIterator<Item = Symbol<T>>,
) -> Option<Vec<Delta<T>>> {
    let mut decoder = Decoder::default();

    let mut a = remote.into_iter();
    let mut b = local.into_iter();

    loop {
        decoder.push(a.next()?, b.next()?);
        if decoder.is_complete() {
            let (remote, local) = decoder.consume();
            return Some(
                (remote.into_iter().map(Delta::InRemote))
                    .chain(local.into_iter().map(Delta::InLocal))
                    .collect(),
            );
        }
    }
}

pub struct Decoder<T> {
    remote: SetStreamIter<T>,
    local: SetStreamIter<T>,
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

                let mut rng = SmallRng::from_seed(symbol.checksum);

                // peel off this cell in all indices
                let mut i = 0u64;
                loop {
                    let Ok(k) = usize::try_from(i) else { break };
                    let Some(s) = self.symbols.get_mut(k) else {
                        break;
                    };
                    *s -= symbol;
                    i = next_index(i, &mut rng);
                }

                if symbol.count.get() == 1 {
                    self.remote.push_unchecked(symbol.sum, symbol.checksum);
                } else if symbol.count.get() == -1 {
                    self.local.push_unchecked(symbol.sum, symbol.checksum);
                } else {
                    unreachable!()
                }
            }
        }

        progress
    }
}
