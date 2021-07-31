#![cfg_attr(feature = "cargo-clippy", allow(clippy::suspicious_else_formatting))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::trivially_copy_pass_by_ref))]
pub mod rellcore;
use rellcore::*;

pub mod parser;

pub mod tree;
use tree::*;

pub mod tree_traits;

pub mod logic;
pub mod symbols;
pub mod binding;

#[cfg(test)]
mod tests
{

}
