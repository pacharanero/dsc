// dsc places `#[cfg(test)] mod tests` next to the code it covers rather than
// at end-of-file, so allow that arrangement crate-wide (purely organizational).
#![allow(clippy::items_after_test_module)]

pub mod api;
pub mod cli;
pub mod commands;
pub mod config;
pub mod utils;
