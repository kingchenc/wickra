#![no_main]
//! Fuzz order-book `Indicator<Input = OrderBook>` implementations with
//! arbitrary depth snapshots.
//!
//! Each iteration consumes a byte stream, interprets it as a sequence of
//! `f64` values (8 bytes each), packs consecutive values into `(price, size)`
//! levels, and groups levels into order-book snapshots. Books are built with
//! `OrderBook::new_unchecked` so the fuzzer can explore degenerate shapes
//! (empty sides, crossed books, non-finite prices, negative sizes) that the
//! validating constructor would reject — the indicators must never panic on
//! any of them, streaming or batched.

use libfuzzer_sys::fuzz_target;
use wickra_core::{
    BatchExt, DepthSlope, Indicator, Level, Microprice, OrderBook, OrderBookImbalanceFull,
    OrderBookImbalanceTop1, OrderBookImbalanceTopN, QuotedSpread,
};

#[inline(never)]
fn drive<I>(make: impl Fn() -> I, books: &[OrderBook])
where
    I: Indicator<Input = OrderBook, Output = f64> + BatchExt,
{
    let mut streaming = make();
    for book in books {
        let _ = streaming.update(book.clone());
    }
    let _ = make().batch(books);
}

fuzz_target!(|data: &[u8]| {
    let floats: Vec<f64> = data
        .chunks_exact(8)
        .map(|c| f64::from_le_bytes(c.try_into().expect("8 bytes")))
        .collect();
    let levels: Vec<Level> = floats
        .chunks_exact(2)
        .map(|c| Level::new_unchecked(c[0], c[1]))
        .collect();
    // Group levels into snapshots of up to four levels (split into bids / asks).
    let books: Vec<OrderBook> = levels
        .chunks(4)
        .map(|chunk| {
            let half = chunk.len() / 2;
            OrderBook::new_unchecked(chunk[..half].to_vec(), chunk[half..].to_vec())
        })
        .collect();

    drive(OrderBookImbalanceTop1::new, &books);
    drive(|| OrderBookImbalanceTopN::new(3).unwrap(), &books);
    drive(OrderBookImbalanceFull::new, &books);
    drive(Microprice::new, &books);
    drive(QuotedSpread::new, &books);
    drive(DepthSlope::new, &books);
});
