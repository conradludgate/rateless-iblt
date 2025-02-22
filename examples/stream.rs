use std::pin::pin;

use bytes::BytesMut;
use rateless_iblt::{Decoder, Encoder, Symbol};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use zerocopy::little_endian;
use zerocopy::IntoBytes;
use zerocopy_buf::ZeroCopyBuf;

#[tokio::main]
async fn main() {
    // bob constructs his encoder
    let mut bob = Encoder::default();
    bob.extend([1, 2, 3, 5].map(little_endian::I64::new));
    let mut bob = bob.into_iter();

    // bob constucts his decoder
    let mut decoder = Decoder::default();

    // bob connects to alice
    let mut alice = decode_stream(connect_to_alice());

    // bob drives the decoder
    while !decoder.is_complete() {
        let alice_symbol = alice.recv().await.unwrap();
        let bob_symbol = bob.next().unwrap();

        decoder.push(alice_symbol, bob_symbol);
    }

    // the decoding is complete, bob now knows what items he and alice are missing
    let (alice_new, bob_new) = decoder.consume();

    assert_eq!(alice_new, vec![4], "alice has 4 but bob does not");
    assert_eq!(bob_new, vec![5], "bob has 5 but alice does not");
}

fn decode_stream(
    s: impl AsyncRead + Send + 'static,
) -> tokio::sync::mpsc::Receiver<Symbol<little_endian::I64>> {
    let (tx, rx) = tokio::sync::mpsc::channel(1);

    tokio::spawn(async move {
        let mut s = pin!(s);
        let mut buf = BytesMut::new();

        loop {
            // try pull entries from the buf
            while let Ok(entry) = buf.try_get() {
                if tx.send(*entry).await.is_err() {
                    break;
                }
            }

            // read more data into the buf
            if s.as_mut().read_buf(&mut buf).await.is_err() {
                break;
            }
        }
    });

    rx
}

fn connect_to_alice() -> impl AsyncRead + Send + 'static {
    let (rx, mut tx) = tokio::io::duplex(64);

    tokio::spawn(async move {
        // alice constructs her encoder
        let mut alice = Encoder::<little_endian::I64>::default();
        alice.extend([1, 2, 3, 4].map(little_endian::I64::new));

        for entry in alice {
            // alice streams out her entries to bob
            if tx.write_all(entry.as_bytes()).await.is_err() {
                break;
            }
        }
    });

    rx
}
