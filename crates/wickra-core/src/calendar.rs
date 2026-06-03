//! Pure calendar arithmetic for the timestamp-driven seasonality indicators.
//!
//! Every indicator in the *Seasonality & Session* family keys off the wall-clock
//! fields of [`Candle::timestamp`](crate::Candle) (epoch milliseconds), shifted
//! by a caller-supplied `utc_offset_minutes` so the buckets line up with the
//! relevant exchange session rather than UTC. This module turns an epoch
//! millisecond instant into its civil fields using Howard Hinnant's
//! branch-light `civil_from_days` algorithm (the same one libc++ ships).
//!
//! All arithmetic is floor-based (`div_euclid`/`rem_euclid`) so instants before
//! the Unix epoch decompose correctly without a dedicated negative-input branch.

/// Civil (wall-clock) decomposition of an epoch-millisecond instant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CivilTime {
    /// Proleptic Gregorian year (can be negative for instants before year 1).
    pub(crate) year: i64,
    /// Month of year, `1..=12`.
    pub(crate) month: u32,
    /// Day of month, `1..=31`.
    pub(crate) day: u32,
    /// Hour of day, `0..=23`.
    pub(crate) hour: u32,
    /// Minute of hour, `0..=59`.
    pub(crate) minute: u32,
    /// Day of week with Monday as `0` through Sunday as `6`.
    pub(crate) weekday: u32,
}

impl CivilTime {
    /// Minute of day, `0..=1439`.
    pub(crate) const fn minute_of_day(&self) -> u32 {
        self.hour * 60 + self.minute
    }
}

/// Decompose an epoch-millisecond instant into local civil fields.
///
/// `utc_offset_minutes` shifts the instant before decomposition: `0` yields
/// UTC, `-300` U.S. Eastern standard time, `60` Central European time, etc.
pub(crate) fn civil_from_timestamp(millis: i64, utc_offset_minutes: i32) -> CivilTime {
    let local_secs = millis.div_euclid(1000) + i64::from(utc_offset_minutes) * 60;
    let days = local_secs.div_euclid(86_400);
    let secs_of_day = local_secs.rem_euclid(86_400);
    let hour = (secs_of_day / 3600) as u32;
    let minute = ((secs_of_day % 3600) / 60) as u32;
    let (year, month, day) = civil_from_days(days);
    // 1970-01-01 was a Thursday; Monday-based weekday is `(z + 3) mod 7`.
    let weekday = (days + 3).rem_euclid(7) as u32;
    CivilTime {
        year,
        month,
        day,
        hour,
        minute,
        weekday,
    }
}

/// Gregorian `(year, month, day)` for a day count `z` relative to 1970-01-01.
///
/// Howard Hinnant, "chrono-Compatible Low-Level Date Algorithms".
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    (if month <= 2 { year + 1 } else { year }, month, day)
}

/// Whether `year` is a Gregorian leap year.
pub(crate) const fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Number of days in `month` (`1..=12`) of `year`.
pub(crate) const fn days_in_month(year: i64, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ => {
            if is_leap(year) {
                29
            } else {
                28
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_zero_is_thursday_midnight() {
        let t = civil_from_timestamp(0, 0);
        assert_eq!(
            t,
            CivilTime {
                year: 1970,
                month: 1,
                day: 1,
                hour: 0,
                minute: 0,
                weekday: 3, // Thursday
            }
        );
        assert_eq!(t.minute_of_day(), 0);
    }

    #[test]
    fn known_utc_instant_mid_year() {
        // 2021-06-15 13:45:00 UTC = 1623764700 s.
        let t = civil_from_timestamp(1_623_764_700_000, 0);
        assert_eq!(t.year, 2021);
        assert_eq!(t.month, 6);
        assert_eq!(t.day, 15);
        assert_eq!(t.hour, 13);
        assert_eq!(t.minute, 45);
        assert_eq!(t.weekday, 1); // Tuesday
        assert_eq!(t.minute_of_day(), 13 * 60 + 45);
    }

    #[test]
    fn new_year_2021_is_friday() {
        // 2021-01-01 00:00:00 UTC = 1609459200 s — exercises the m<=2 year bump.
        let t = civil_from_timestamp(1_609_459_200_000, 0);
        assert_eq!((t.year, t.month, t.day), (2021, 1, 1));
        assert_eq!(t.weekday, 4); // Friday
    }

    #[test]
    fn positive_offset_rolls_to_next_day() {
        // 2021-01-01 23:30 UTC shifted +60 min -> 2021-01-02 00:30 local.
        let base = 1_609_459_200_000 + (23 * 3600 + 30 * 60) * 1000;
        let t = civil_from_timestamp(base, 60);
        assert_eq!((t.year, t.month, t.day), (2021, 1, 2));
        assert_eq!((t.hour, t.minute), (0, 30));
        assert_eq!(t.weekday, 5); // Saturday
    }

    #[test]
    fn negative_offset_rolls_to_previous_day() {
        // 2021-01-01 00:30 UTC shifted -60 min -> 2020-12-31 23:30 local.
        let base = 1_609_459_200_000 + 30 * 60 * 1000;
        let t = civil_from_timestamp(base, -60);
        assert_eq!((t.year, t.month, t.day), (2020, 12, 31));
        assert_eq!((t.hour, t.minute), (23, 30));
        assert_eq!(t.weekday, 3); // Thursday
    }

    #[test]
    fn sub_epoch_millis_floor_correctly() {
        // -1 ms -> 1969-12-31 23:59:59.999, a Wednesday.
        let t = civil_from_timestamp(-1, 0);
        assert_eq!((t.year, t.month, t.day), (1969, 12, 31));
        assert_eq!((t.hour, t.minute), (23, 59));
        assert_eq!(t.weekday, 2); // Wednesday
    }

    #[test]
    fn far_negative_day_count_hits_pre_era_branch() {
        // A day count below -719468 drives `z + 719468` negative, exercising the
        // `z - 146096` era branch in civil_from_days (year < 1).
        let (year, month, day) = civil_from_days(-1_000_000);
        // -1_000_000 days before 1970-01-01 is 0768-02-04 BCE (proleptic
        // Gregorian, astronomical year numbering where year 0 exists).
        assert_eq!((year, month, day), (-768, 2, 4));
    }

    #[test]
    fn leap_year_rules() {
        assert!(is_leap(2000));
        assert!(!is_leap(1900));
        assert!(is_leap(2024));
        assert!(!is_leap(2023));
    }

    #[test]
    fn days_in_month_all_cases() {
        assert_eq!(days_in_month(2023, 1), 31);
        assert_eq!(days_in_month(2023, 4), 30);
        assert_eq!(days_in_month(2023, 2), 28);
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2023, 12), 31);
        assert_eq!(days_in_month(2023, 11), 30);
    }

    #[test]
    fn leap_day_decodes() {
        // 2024-02-29 12:00 UTC.
        let secs = 1_709_208_000; // 2024-02-29T12:00:00Z
        let t = civil_from_timestamp(secs * 1000, 0);
        assert_eq!((t.year, t.month, t.day), (2024, 2, 29));
        assert_eq!(t.hour, 12);
    }
}
