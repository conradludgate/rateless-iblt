use std::{cmp::Reverse, collections::BinaryHeap};

use rand::{rngs::SmallRng, SeedableRng};
use zerocopy::{FromBytes, Immutable, IntoBytes};

use crate::{next_index, stream::Entry, SetStreamIter, Symbol};

#[derive(PartialEq, Debug)]
pub enum Delta<T> {
    InRemote(T),
    InLocal(T),
}

pub fn set_difference<T: FromBytes + IntoBytes + Immutable + Copy>(
    remote: impl IntoIterator<Item = Symbol<T>>,
    local: impl IntoIterator<Item = Symbol<T>>,
) -> Option<Vec<Delta<T>>> {
    let mut a = remote.into_iter();
    let mut b = local.into_iter();

    let mut remote = SetStreamIter {
        index: 0,
        heap: BinaryHeap::new(),
        size: 0,
    };

    let mut local = SetStreamIter {
        index: 0,
        heap: BinaryHeap::new(),
        size: 0,
    };

    let mut symbols = vec![];
    loop {
        let cell = a.next()? - b.next()? - remote.next()? + local.next()?;
        let pure = cell.is_pure_cell();

        symbols.push(cell);

        if !pure {
            continue;
        }

        loop {
            let progress = peel_one_layer(&mut symbols, &mut remote, &mut local);
            if progress == 0 {
                return Some(
                    (remote
                        .heap
                        .into_vec()
                        .into_iter()
                        .map(|e| Delta::InRemote(e.0.value)))
                    .chain(
                        local
                            .heap
                            .into_vec()
                            .into_iter()
                            .map(|e| Delta::InLocal(e.0.value)),
                    )
                    .collect(),
                );
            } else if progress < symbols.len() {
                continue;
            } else {
                break;
            }
        }
    }
}

fn peel_one_layer<T: FromBytes + IntoBytes + Immutable + Copy>(
    symbols: &mut [Symbol<T>],
    remote: &mut SetStreamIter<T>,
    local: &mut SetStreamIter<T>,
) -> usize {
    let mut progress = symbols.len();
    for j in (0..symbols.len()).rev() {
        let symbol = symbols[j];

        if j == 0 && symbol.checksum == [0; 32] {
            return 0;
        }

        if symbol.is_pure_cell() {
            progress = j;

            let mut rng = SmallRng::from_seed(symbol.checksum);

            // peel off this cell in all indices
            let mut i = 0u64;
            loop {
                let Ok(k) = usize::try_from(i) else { break };
                let Some(s) = symbols.get_mut(k) else { break };
                *s -= symbol;
                i = next_index(i, &mut rng);
            }

            if symbol.count.get() == 1 {
                remote.heap.push(Reverse(Entry {
                    next: i,
                    value: symbol.sum,
                    checksum: symbol.checksum,
                    rng,
                }));
            } else if symbol.count.get() == -1 {
                local.heap.push(Reverse(Entry {
                    next: i,
                    value: symbol.sum,
                    checksum: symbol.checksum,
                    rng,
                }));
            } else {
                unreachable!()
            }

            // if we decode cell 0, we are finished
            if j == 0 {
                break;
            }
        }
    }

    progress
}
