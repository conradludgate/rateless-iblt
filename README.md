# rateless-iblt

Rateless invertible bloom lookup table. This is a bloom filter like construction that also exposes lookup support.
It requires no design parameters to tune the correct size, as it naturally resizes until there's enough data to
extract all entries from the table. The intended usecase is set reconcilliation in distributed networks.
This problem involves determining the difference of two sets on different nodes. Rather than sending the entire set,
you can compress it into the bloom lookup table. Later, the receiver can "subtract" their nodes from the lookup table,
which will reveal the differences.

Based on https://arxiv.org/pdf/2402.02668
> Practical Rateless Set Reconciliation

## Usage

```rust
use rateless_iblt::{Symbol, Encoder, Decoder};

fn connect_to_alice() -> impl Iterator<Item = Symbol<u64>> {
    let (tx, rx) = std::sync::mpsc::sync_channel(1);

    std::thread::spawn(move || {
        // alice constructs her encoder
        let mut alice = Encoder::default();
        alice.extend([1, 2, 3, 4]);

        for entry in alice {
            // alice streams out her entries to bob
            if tx.send(entry).is_err() {
                break;
            }
        }
    });

    rx.into_iter()
}

// bob constructs his encoder
let mut bob = Encoder::default();
bob.extend([1, 2, 3, 5]);
let mut bob = bob.into_iter();

// bob constucts his decoder
let mut decoder = Decoder::default();

// bob connects to alice
let mut alice = connect_to_alice();

// bob drives the decoder
while !decoder.is_complete() {
    let alice_symbol = alice.next().unwrap();
    let bob_symbol = bob.next().unwrap();

    decoder.push(alice_symbol, bob_symbol);
}

// the decoding is complete, bob now knows what items he and alice are missing
let (alice_new, bob_new) = decoder.consume();

assert_eq!(alice_new, vec![4], "alice has 4 but bob does not");
assert_eq!(bob_new, vec![5], "bob has 5 but alice does not");
```
