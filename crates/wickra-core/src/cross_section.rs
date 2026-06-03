//! Cross-section value type: a market-breadth snapshot across a whole universe.
//!
//! A [`CrossSection`] is a single tick that carries the per-symbol state of
//! *every* symbol in a universe at one point in time. It is the non-OHLCV input
//! consumed by the market-breadth indicator family (advance/decline, `McClellan`,
//! the TRIN / Arms index, the high-low index, ...), each of which aggregates the
//! whole cross-section into a single breadth reading. This is the same
//! one-rich-type-per-family pattern as [`DerivativesTick`] and [`OrderBook`].
//!
//! Each [`Member`] precomputes the per-symbol signals the breadth indicators
//! need — a signed price `change` (whose sign classifies the symbol as
//! advancing, declining or unchanged), the period `volume`, and the
//! `new_high` / `new_low` extreme flags — so the indicators stay stateless per
//! tick and never have to track per-symbol history.
//!
//! [`DerivativesTick`]: crate::DerivativesTick
//! [`OrderBook`]: crate::OrderBook

use crate::error::{Error, Result};

/// One symbol's contribution to a [`CrossSection`] tick.
///
/// Field invariants enforced by [`CrossSection::new`] when the member is placed
/// into a tick:
///
/// - `change` is finite (its sign classifies the symbol — positive is
///   advancing, negative is declining, zero is unchanged).
/// - `volume` is finite and non-negative.
///
/// `new_high` / `new_low` are caller-supplied flags marking whether the symbol
/// printed a new period extreme; they carry no numeric invariant.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Member {
    /// Price change versus the previous close. Sign classifies the symbol:
    /// positive is advancing, negative is declining, zero is unchanged.
    pub change: f64,
    /// Period volume for the symbol (finite, non-negative).
    pub volume: f64,
    /// Whether the symbol printed a new period high.
    pub new_high: bool,
    /// Whether the symbol printed a new period low.
    pub new_low: bool,
}

impl Member {
    /// Assemble a cross-section member.
    ///
    /// The field invariants documented on [`Member`] are validated centrally by
    /// [`CrossSection::new`] when the member is placed into a tick; this
    /// constructor only assembles the value so the `#[non_exhaustive]` struct can
    /// be built from outside the crate.
    #[must_use]
    pub const fn new(change: f64, volume: f64, new_high: bool, new_low: bool) -> Self {
        Self {
            change,
            volume,
            new_high,
            new_low,
        }
    }
}

/// A market-breadth cross-section: the per-symbol state of an entire universe at
/// a single point in time.
///
/// Invariants enforced by [`new`](CrossSection::new):
///
/// - `members` is non-empty (a breadth reading needs at least one symbol).
/// - every member's `change` is finite, and `volume` is finite and non-negative.
///
/// `timestamp` is a caller-defined epoch / resolution and is not validated.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub struct CrossSection {
    /// Per-symbol members of the universe for this tick.
    pub members: Vec<Member>,
    /// Tick timestamp (caller-defined epoch / resolution).
    pub timestamp: i64,
}

impl CrossSection {
    /// Construct a cross-section, validating every member invariant.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidCrossSection`] if `members` is empty, if any
    /// member has a non-finite `change`, or if any member has a `volume` that is
    /// not a finite non-negative number.
    pub fn new(members: Vec<Member>, timestamp: i64) -> Result<Self> {
        if members.is_empty() {
            return Err(Error::InvalidCrossSection {
                message: "cross-section must contain at least one member",
            });
        }
        for member in &members {
            if !member.change.is_finite() {
                return Err(Error::InvalidCrossSection {
                    message: "member change must be finite",
                });
            }
            if !member.volume.is_finite() || member.volume < 0.0 {
                return Err(Error::InvalidCrossSection {
                    message: "member volume must be finite and non-negative",
                });
            }
        }
        Ok(Self { members, timestamp })
    }

    /// Construct a cross-section without validation. The caller asserts that
    /// every invariant documented on [`CrossSection`] holds.
    #[must_use]
    pub const fn new_unchecked(members: Vec<Member>, timestamp: i64) -> Self {
        Self { members, timestamp }
    }

    /// Number of advancing symbols (those with a strictly positive `change`).
    #[must_use]
    pub fn advancers(&self) -> usize {
        self.members.iter().filter(|m| m.change > 0.0).count()
    }

    /// Number of declining symbols (those with a strictly negative `change`).
    #[must_use]
    pub fn decliners(&self) -> usize {
        self.members.iter().filter(|m| m.change < 0.0).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn members() -> Vec<Member> {
        vec![
            Member::new(1.5, 100.0, true, false),
            Member::new(-0.5, 50.0, false, true),
            Member::new(0.0, 0.0, false, false),
        ]
    }

    #[test]
    fn new_accepts_valid() {
        let cs = CrossSection::new(members(), 42).unwrap();
        assert_eq!(cs.members.len(), 3);
        assert_eq!(cs.timestamp, 42);
        assert_eq!(cs.members[0].change, 1.5);
        assert_eq!(cs.members[0].volume, 100.0);
        assert!(cs.members[0].new_high);
        assert!(cs.members[1].new_low);
    }

    #[test]
    fn member_new_assembles_fields() {
        let m = Member::new(2.0, 10.0, true, false);
        assert_eq!(m.change, 2.0);
        assert_eq!(m.volume, 10.0);
        assert!(m.new_high);
        assert!(!m.new_low);
    }

    #[test]
    fn new_rejects_empty() {
        assert!(matches!(
            CrossSection::new(Vec::new(), 0),
            Err(Error::InvalidCrossSection { .. })
        ));
    }

    #[test]
    fn new_rejects_non_finite_change() {
        assert!(matches!(
            CrossSection::new(vec![Member::new(f64::NAN, 10.0, false, false)], 0),
            Err(Error::InvalidCrossSection { .. })
        ));
        assert!(matches!(
            CrossSection::new(vec![Member::new(f64::INFINITY, 10.0, false, false)], 0),
            Err(Error::InvalidCrossSection { .. })
        ));
    }

    #[test]
    fn new_rejects_negative_volume() {
        assert!(matches!(
            CrossSection::new(vec![Member::new(1.0, -1.0, false, false)], 0),
            Err(Error::InvalidCrossSection { .. })
        ));
    }

    #[test]
    fn new_rejects_non_finite_volume() {
        assert!(matches!(
            CrossSection::new(vec![Member::new(1.0, f64::NAN, false, false)], 0),
            Err(Error::InvalidCrossSection { .. })
        ));
    }

    #[test]
    fn new_unchecked_skips_validation() {
        let cs = CrossSection::new_unchecked(vec![Member::new(f64::NAN, -1.0, false, false)], 7);
        assert_eq!(cs.members.len(), 1);
        assert_eq!(cs.timestamp, 7);
    }

    #[test]
    fn advancers_and_decliners_count_by_sign() {
        let cs = CrossSection::new(members(), 0).unwrap();
        assert_eq!(cs.advancers(), 1);
        assert_eq!(cs.decliners(), 1);
    }

    #[test]
    fn unchanged_members_count_as_neither() {
        let cs = CrossSection::new(
            vec![
                Member::new(0.0, 1.0, false, false),
                Member::new(0.0, 1.0, false, false),
            ],
            0,
        )
        .unwrap();
        assert_eq!(cs.advancers(), 0);
        assert_eq!(cs.decliners(), 0);
    }
}
