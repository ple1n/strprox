use std::collections::BinaryHeap;

use crate::MeasuredPrefix;

pub mod fst;
pub mod meta;

pub trait Autocompleter {
    /// Returns the `requested` number of strings with the best PEDs that are at most `max_threshold`,
    /// or all strings available with PEDs within `max_threshold`
    ///
    /// Strings are sorted by prefix edit distance and then lexicographical order
    ///
    /// Assumes `query`'s length in Unicode characters is bounded by u8; will truncate to u8::MAX characters otherwise
    fn threshold_topk(
        &self,
        query: &str,
        requested: usize,
        max_threshold: usize,
    ) -> Vec<MeasuredPrefix>;

    /// Returns the `requested` number of strings with the best PEDs, or all strings available if less than `requested`
    ///
    /// Strings are sorted by prefix edit distance and then lexicographical order
    ///
    /// Assumes `query`'s length in Unicode characters is bounded by u8; will truncate to u8::MAX characters otherwise
    fn autocomplete(&self, query: &str, requested: usize) -> Vec<MeasuredPrefix> {
        self.threshold_topk(query, requested, usize::MAX)
    }
}

pub trait FromStrings {
    /// Returns an autocompleter which has indexed `strings`
    fn from_strings(strings: &[&str]) -> Self;
}

/// Structure convertible to MeasuredPrefix that compared only using the PED
#[derive(PartialEq, Eq, Clone, Debug)]
struct PrefixRanking {
    string: String,
    prefix_distance: usize,
}

impl Ord for PrefixRanking {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.prefix_distance.cmp(&other.prefix_distance)
    }
}
impl PartialOrd for PrefixRanking {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl From<PrefixRanking> for MeasuredPrefix {
    fn from(value: PrefixRanking) -> Self {
        let PrefixRanking {
            string,
            prefix_distance,
        } = value;
        MeasuredPrefix {
            string,
            prefix_distance,
        }
    }
}

/// Structure that holds at most `limit` of the strings closest to the query
/// with prefix edit distances of at most `max_ped`
#[derive(Clone, Default)]
pub struct PrefixRankings {
    best: BinaryHeap<PrefixRanking>,
    limit: usize,
    max_ped: usize,
}

impl PrefixRankings {
    /// Returns an Option with an inclusive upper bound for the prefix edit distance
    /// required for ranking, or None if the worst PED is already 0
    fn threshold(&self) -> Option<usize> {
        // we don't have enough results yet to set the threshold
        if self.best.len() < self.limit as usize {
            Some(self.max_ped)
        }
        // the PED of ranked strings are bounded by the current worst PED in the rankings
        else {
            // this will fail if best.len() is 0 and limit is also 0, which shouldn't be possible from the public API
            let worst = self.best.peek().unwrap().prefix_distance;
            if worst == 0 {
                None
            } else {
                // has to be better than the current rankings
                // autocomplete should terminate when worst is 0
                Some(worst - 1)
            }
        }
    }
    /// Returns Rankings that can rank up to `limit` strings with PEDs of at most `max_ped`
    fn new(limit: usize, max_ped: usize) -> PrefixRankings {
        PrefixRankings {
            best: Default::default(),
            limit,
            max_ped,
        }
    }
    /// Ranks `measure` if the number of rankings hasn't reached `self.limit`
    /// or if its PED is smaller than the current worst ranking
    fn consider(&mut self, measure: PrefixRanking) {
        // ideally the invariant wouldn't need to be preserved here,
        // and the autocomplete algorithm simply would prune away strings
        // with higher PEDs
        if measure.prefix_distance <= self.max_ped {
            self.best.push(measure);
            if self.best.len() > self.limit {
                self.best.pop();
            }
        }
    }
    /// Returns all rankings as strings with their PEDs sorted in ascending order by PED
    pub fn into_measures(self) -> Vec<MeasuredPrefix> {
        let mut measures: Vec<MeasuredPrefix> = self
            .best
            .into_sorted_vec()
            .into_iter()
            .map(Into::into)
            .collect();
        // sort by strings in lexicographical order as tie-breaker
        measures.sort();
        measures
    }
    /// Returns all ranked strings sorted by ascending prefix edit distance
    pub fn into_strings(self) -> Vec<String> {
        self.into_measures()
            .into_iter()
            .map(|measure| measure.string)
            .collect()
    }
}
