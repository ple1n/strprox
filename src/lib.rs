#![feature(closure_lifetime_binder)]
#![feature(let_chains)]
pub mod strprox;

#[doc(inline)]
pub use strprox::*;

pub mod levenshtein;
#[cfg(test)]
mod tests;
#[cfg(feature = "wasm")]
pub mod wasm;