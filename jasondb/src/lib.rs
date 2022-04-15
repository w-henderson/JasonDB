//! JasonDB is a blazingly-fast, JSON-based, log-structured database for Rust.
//! It can be backed with a file or be used fully in-memory, and it is highly resilient and flexible.

#![warn(missing_docs)]

mod database;
pub mod error;
pub mod replica;
pub mod sources;
mod util;

#[macro_use]
pub mod query;

#[cfg(test)]
mod tests;

pub use database::Database;
