#![no_main]
//! Fuzz market-breadth `Indicator<Input = CrossSection>` implementations with
//! arbitrary cross-section streams.
//!
//! Each iteration consumes a byte stream, interprets it as a sequence of `f64`
//! values (8 bytes each), packs consecutive pairs into [`Member`]s (a `change`
//! and a `volume`, with the high/low flags taken from the value bit parity), and
//! groups the members into bounded-size [`CrossSection`] ticks. Cross-sections
//! are built with `new_unchecked` so the fuzzer can explore degenerate values
//! (non-finite changes, negative volumes, empty-adjacent groups) that the
//! validating constructor would reject — the indicators must never panic,
//! streaming or batched.

use libfuzzer_sys::fuzz_target;
use wickra_core::{AbsoluteBreadthIndex, AdVolumeLine, AdvanceDecline, AdvanceDeclineRatio, BatchExt, BreadthThrust, BullishPercentIndex, CrossSection, CumulativeVolumeIndex, HighLowIndex, Indicator, McClellanOscillator, McClellanSummationIndex, Member, NewHighsNewLows, PercentAboveMa, TickIndex, Trin, UpDownVolumeRatio};

#[inline(never)]
fn drive<I>(make: impl Fn() -> I, sections: &[CrossSection])
where
    I: Indicator<Input = CrossSection, Output = f64> + BatchExt,
{
    let mut streaming = make();
    for section in sections {
        let _ = streaming.update(section.clone());
    }
    let _ = make().batch(sections);
}

fuzz_target!(|data: &[u8]| {
    let floats: Vec<f64> = data
        .chunks_exact(8)
        .map(|c| f64::from_le_bytes(c.try_into().expect("8 bytes")))
        .collect();
    let members: Vec<Member> = floats
        .chunks_exact(2)
        .map(|c| Member::new(c[0], c[1], c[0].to_bits() & 1 == 1, c[1].to_bits() & 1 == 1))
        .collect();
    // Group members into cross-sections of up to eight symbols each so a single
    // input yields a stream of ragged universes.
    let sections: Vec<CrossSection> = members
        .chunks(8)
        .filter(|chunk| !chunk.is_empty())
        .map(|chunk| CrossSection::new_unchecked(chunk.to_vec(), 0))
        .collect();

    drive(AdvanceDecline::new, &sections);
    drive(AdvanceDeclineRatio::new, &sections);
    drive(AdVolumeLine::new, &sections);
    drive(McClellanOscillator::new, &sections);
    drive(McClellanSummationIndex::new, &sections);
    drive(Trin::new, &sections);
    drive(|| BreadthThrust::new(10).unwrap(), &sections);
    drive(NewHighsNewLows::new, &sections);
    drive(|| HighLowIndex::new(10).unwrap(), &sections);
    drive(PercentAboveMa::new, &sections);
    drive(UpDownVolumeRatio::new, &sections);
    drive(BullishPercentIndex::new, &sections);
    drive(CumulativeVolumeIndex::new, &sections);
    drive(AbsoluteBreadthIndex::new, &sections);
    drive(TickIndex::new, &sections);
});
