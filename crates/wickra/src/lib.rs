//! Wickra: streaming-first technical analysis.
//!
//! This crate is a thin re-export of [`wickra_core`] so downstream users can depend on
//! a single `wickra` package without thinking about the internal split. Every public
//! item lives in `wickra_core`; only the names re-exported here are part of the stable
//! public API.
//!
//! # Example
//!
//! ```
//! use wickra::{Indicator, Sma};
//!
//! let mut sma = Sma::new(3).unwrap();
//! let prices = [1.0, 2.0, 3.0, 4.0, 5.0];
//! let out: Vec<Option<f64>> = prices.iter().map(|p| sma.update(*p)).collect();
//! assert_eq!(out, vec![None, None, Some(2.0), Some(3.0), Some(4.0)]);
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]

pub use wickra_core::*;
