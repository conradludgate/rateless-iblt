use std::cmp::Reverse;

use rand::{rngs::SmallRng, SeedableRng};

use crate::{next_index, stream::Entry, SetStream, SetStreamIter, Symbol};

pub fn set_difference(a: impl IntoIterator<Item = Symbol>, b: SetStream) -> Option<Vec<[u8; 32]>> {
    let mut decoded = vec![];

    let mut a = a.into_iter();
    let mut b = b.into_iter();

    let mut symbols = vec![];
    loop {
        let cell = a.next()? - b.next()?;
        let pure = cell.is_pure_cell();

        symbols.push(cell);

        if !pure {
            continue;
        }

        loop {
            let progress = peel_one_layer(&mut symbols, &mut b, &mut decoded);
            if progress == 0 {
                return Some(decoded);
            } else if progress < symbols.len() {
                continue;
            } else {
                break;
            }
        }
    }

    // decoded
}

fn peel_one_layer(
    symbols: &mut [Symbol],
    b: &mut SetStreamIter,
    decoded: &mut Vec<[u8; 32]>,
) -> usize {
    let mut progress = symbols.len();
    for j in (0..symbols.len()).rev() {
        let symbol = symbols[j];
        if symbol.is_pure_cell() {
            progress = j;

            let mut rng = SmallRng::from_seed(symbol.hash);

            // peel off this cell in all indices
            let mut i = 0u64;
            loop {
                let Ok(k) = usize::try_from(i) else { break };
                let Some(s) = symbols.get_mut(k) else { break };
                *s -= symbol;
                i = next_index(i, &mut rng);
            }

            decoded.push(symbol.value);
            b.heap.push(Reverse(Entry {
                next: i,
                value: symbol.value,
                hash: symbol.hash,
                rng,
            }));

            // if we decode cell 0, we are finished
            if j == 0 {
                break;
            }
        }
    }

    progress
}
