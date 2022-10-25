//! A simple automated market maker for cross chain transactions managed by Paloma.

#![warn(missing_docs)]

pub mod contract;
mod error;
pub mod helpers;
pub mod msg;
pub mod state;

pub use crate::error::ContractError;
