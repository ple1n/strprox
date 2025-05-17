pub mod prefix;
use std::{cmp::Ordering, fmt::Display};

pub use prefix::meta::{TreeString, TreeStringT};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
//
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

/*
mod hs_tree;
use hs_tree::HSTree;
use hs_tree::Rankings;
*/

/// Structure that associates a string with its Levenshtein distance from the query
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "wasm", wasm_bindgen(getter_with_clone))]
pub struct MeasuredString {
    pub string: String,
    pub distance: usize,
}
impl Ord for MeasuredString {
    /// Compare the edit distances and then the strings for MeasuredString
    fn cmp(&self, other: &Self) -> Ordering {
        self.distance
            .cmp(&other.distance)
            .then_with(|| self.string.cmp(&other.string))
    }
}

/// Structure that associates a string with its its prefix edit distance from the query
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "wasm", wasm_bindgen(getter_with_clone))]
pub struct MeasuredPrefix {
    pub string: String,
    pub prefix_distance: usize,
}
impl Ord for MeasuredPrefix {
    /// Compare the prefix and then the strings for MeasuredPrefix
    fn cmp(&self, other: &Self) -> Ordering {
        self.prefix_distance
            .cmp(&other.prefix_distance)
            .then_with(|| self.string.cmp(&other.string))
    }
}
impl PartialOrd for MeasuredString {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialOrd for MeasuredPrefix {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Display for MeasuredPrefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(string: {}, PED: {})",
            self.string, self.prefix_distance
        )
    }
}

//#[doc(inline)]
#[doc(inline)]
pub use prefix::Autocompleter;

#[cfg(feature = "fst")]
#[doc(inline)]
pub use prefix::fst::FstAutocompleter;

#[doc(inline)]
pub use prefix::meta::MetaAutocompleter;
//pub type StringSearcher<'a, U> = HSTree<'a, U>;
