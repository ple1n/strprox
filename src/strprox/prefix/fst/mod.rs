use std::{
    cmp::{max, min},
    collections::{HashMap, HashSet},
    ops::Range,
};

use fst::{
    raw::{Fst, Node, Transition},
    Set, Streamer,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{levenshtein, MeasuredPrefix};

use super::{Autocompleter, PrefixRanking, PrefixRankings, FromStrings};

/// Supports error-tolerant autocompletion against a finite-state transducer index
pub struct FstAutocompleter<D: AsRef<[u8]>> {
    pub index: Fst<D>,
}

impl<D: AsRef<[u8]>> Autocompleter for FstAutocompleter<D> {
    fn threshold_topk(&self, query: &str, requested: usize, max_threshold: usize) -> Vec<MeasuredPrefix> {
        if requested == 0 {
            return vec![];
        }
        // get the first strings in the index if the query is empty
        if query.is_empty() {
            let mut stream = self.index.stream();
            let mut result = vec![];
            while let Some((bytes, _)) = stream.next() {
                let string = std::str::from_utf8(bytes).unwrap().to_string();
                let prefix_distance = levenshtein::prefix_edit_distance(query, string.as_str());
                result.push(MeasuredPrefix {
                    string,
                    prefix_distance,
                });
            }
            return result;
        }

        let mut rankings = PrefixRankings::new(requested, max_threshold);

        let mut query: Vec<char> = query.chars().collect();
        if query.len() > u8::MAX as usize {
            query.truncate(u8::MAX as usize);
        }

        let mut query_charset = HashSet::<char>::new();
        for &character in &query {
            query_charset.insert(character);
        }

        let mut ped_matrix = PedMatrix::new(&query);

        self.search(
            requested,
            &mut rankings,
            &query,
            &query_charset,
            self.index.root(),
            &mut Default::default(),
            &mut ped_matrix,
            PartialRowVariantInfo::Mismatch,
        );

        rankings.into_measures()
    }
}

impl FromStrings for FstAutocompleter<Vec<u8>> {
    fn from_strings(strings: &[&str]) -> Self {
        let mut source = strings.to_owned();
        source.sort();
        Self {
            index: Set::from_iter(source).unwrap().into_fst(),
        }
    }
}

/// Iterator over the next characters and associated states from a node in a FST
struct NodeCharIterator<'f, 'n, D: AsRef<[u8]>> {
    /// FST used to retrieve the nodes associated with the next characters
    fst: &'f Fst<D>,
    /// Nodes before the transitions
    nodes: [Option<Node<'n>>; 4],
    /// Indices for possible transitions needed to fill a character
    transition_indices: [u8; 4],
    /// Bytes used to decode UTF-8
    bytes: [u8; 4],
    /// Last index of byte needed to decode a character in UTF-8
    last: u8,
}

impl<'f: 'n, 'n: 'f, D: AsRef<[u8]>> NodeCharIterator<'f, 'n, D> {
    /// Returns the char at the iterator
    fn get_char(&self) -> char {
        std::str::from_utf8(&self.bytes[0..=self.last as usize])
            .unwrap()
            .chars()
            .next()
            .unwrap()
    }
    /// Determines the number of bytes needed to decode a UTF-8 character
    fn classify_first_byte(&mut self, byte: u8) {
        // https://en.wikipedia.org/wiki/UTF-8#Encoding
        self.last = if byte < 0b11000000 {
            0
        } else if byte < 0b11100000 {
            1
        } else if byte < 0b11110000 {
            2
        } else {
            3
        }
    }
    /// Initializes subsequent nodes for decoding UTF-8 based on the first transition
    fn initialize_subsequent_nodes(&mut self, mut transition: Transition) {
        for i in 1..=self.last as usize {
            let node = self.fst.node(transition.addr);
            self.nodes[i] = Some(node);
            //self.transition_indices[i] = 0;
            debug_assert_eq!(self.transition_indices[i], 0);
            // get the first transition of the next node
            transition = self.fst.node(transition.addr).transition(0);
        }
    }
    /// Prepare to decode UTF-8 based on the first transition
    fn set_first_transition(&mut self, transition: Transition) {
        self.classify_first_byte(transition.inp);
        self.initialize_subsequent_nodes(transition);
    }
    /// Returns an iterator over all characters and associated nodes after the `node` in `fst`
    fn new(fst: &'f Fst<D>, node: &'f Node<'n>) -> Self {
        Self {
            fst,
            nodes: [Some(node.clone()), None, None, None],
            transition_indices: [0; 4],
            bytes: [0; 4],
            last: 0,
        }
    }
}

impl<'f: 'n, 'n: 'f, D: AsRef<[u8]>> Iterator for NodeCharIterator<'f, 'n, D> {
    type Item = (Node<'n>, char);
    /// Returns Option with the next Node and character if possible
    fn next(&mut self) -> Option<Self::Item> {
        let mut index = self.last as usize;
        loop {
            let transition_index = self.transition_indices[index] as usize;
            let node = self.nodes[index].unwrap();
            // if it's not the last transition from the node
            if transition_index < node.len() {
                self.transition_indices[index] += 1;
                let transition = node.transition(transition_index);
                self.bytes[index] = transition.inp;
                if index == 0 {
                    self.set_first_transition(transition);
                }
                if index == self.last as usize {
                    return Some((self.fst.node(transition.addr), self.get_char()));
                } else {
                    index += 1;
                }
            } else {
                if index == 0 {
                    return None;
                } else {
                    self.transition_indices[index] = 0;
                    index -= 1;
                }
            }
        }
    }
}

impl<D: AsRef<[u8]>> FstAutocompleter<D> {
    /// Returns FstAutocompleter using the FST `index`
    pub fn new(index: Fst<D>) -> Self {
        Self { index }
    }
    /// Searches for strings formed by the `prefix` and suffixes from `node` to rank
    fn search(
        &self,
        requested: usize,
        rankings: &mut PrefixRankings,
        query: &Vec<char>,
        query_charset: &HashSet<char>,
        node: Node,
        prefix: &mut String,
        ped_matrix: &mut PedMatrix,
        variant: PartialRowVariantInfo,
    ) {
        //dbg!(&prefix);
        if node.is_final() {
            let prefix_distance = ped_matrix.min_ed() as usize;
            //println!("fin {} {}", prefix, prefix_distance);
            rankings.consider(PrefixRanking {
                string: prefix.clone(),
                // this is only the PED if there are no edit distances after or they're all higher
                prefix_distance,
            });
        }
        // a final node may still have transitions
        let iter = NodeCharIterator::new(&self.index, &node);

        let mut matching_destinations = Vec::with_capacity(u8::MAX as usize);
        let mut mismatch_destinations = Vec::with_capacity(u8::MAX as usize);

        // check the transitions that lead to matching characters first to avoid recomputing
        // the row after the next for mismatch
        for (node, character) in iter {
            if query_charset.contains(&character) {
                matching_destinations.push((node, character));
            } else {
                mismatch_destinations.push((node, character));
            }
        }

        let dest_sets = [matching_destinations, mismatch_destinations];
        // index into the current set of destinations
        let mut dest_set_index = 0;

        while dest_set_index < 2 {
            let mut dest_index = 0;
            let dest_set = &dest_sets[dest_set_index];
            while dest_index < dest_set.len() {
                let (dest_node, character) = dest_set[dest_index];
                if let Some(threshold) = rankings.threshold() {
                    if let Some(range) = ped_matrix.next_range(variant, threshold as u8, query) {
                        prefix.push(character);
                        let next_variant = ped_matrix.next_row(
                            variant,
                            range.clone(),
                            query,
                            query_charset,
                            character,
                        );
                        self.search(
                            requested,
                            rankings,
                            query,
                            query_charset,
                            dest_node,
                            prefix,
                            ped_matrix,
                            next_variant,
                        );
                        ped_matrix.current -= 1;
                        prefix.pop();
                        dest_index += 1;
                    } else {
                        // all edit distances afterwards are higher than the current one, so prefix_distance is the PED
                        let prefix_distance = ped_matrix.min_ed() as usize;
                        let mut added = 0;

                        // check all remaining destination nodes and characters or until the requested number have been checked
                        while dest_set_index < 2 {
                            let dest_set = &dest_sets[dest_set_index];
                            while dest_index < dest_set.len() {
                                let (dest_node, character) = dest_set[dest_index];
                                prefix.push(character);
                                self.consider_strings(
                                    requested,
                                    prefix_distance,
                                    rankings,
                                    dest_node,
                                    prefix,
                                    &mut added,
                                );
                                prefix.pop();
                                if added == requested { return; }
                                dest_index += 1;
                            }
                            dest_set_index += 1;
                        }
                        return;
                    }
                } else {
                    // the worst PED is 0 so the top-k has been found
                    return;
                }
            }
            dest_set_index += 1;
        }
    }
    /// Adds up to `requested` strings with the `prefix_distance` to the rankings from the `node`
    fn consider_strings(
        &self,
        requested: usize,
        prefix_distance: usize,
        rankings: &mut PrefixRankings,
        node: Node,
        prefix: &mut String,
        added: &mut usize,
    ) {
        if node.is_final() {
            rankings.consider(PrefixRanking {
                string: prefix.clone(),
                prefix_distance,
            });
            *added += 1;
            if *added == requested {
                return;
            }
        }
        // not sure how using NodeCharIterator, which has the overhead of converting from UTF-8 to char and back,
        // compares to using fst's Streamer, which checks for an automaton and needs to traverse the previous nodes to start
        let iter = NodeCharIterator::new(&self.index, &node);
        for (node, character) in iter {
            prefix.push(character);
            self.consider_strings(requested, prefix_distance, rankings, node, prefix, added);
            prefix.pop();
            if *added == requested {
                break;
            }
        }
    }
}

/// A partial row of edit distances between a candidate prefix and all prefixes of a query
#[derive(Debug)]
struct PartialRow {
    /// The starting index into the full row where `distances` begins
    start: u8,
    /// The edit distances between the candidate prefix and query[start - 1..start - 1 + distances.len()]
    distances: Vec<u8>,
}

impl PartialRow {
    /// Returns an inclusive lower bound on edit distances between all longer prefixes and the query
    fn min_ed_after(&self) -> u8 {
        // early termination criterion from doi:10.14778/2078331.2078340
        *self.distances.iter().min().unwrap()
    }
    /// Returns the edit distance between the current candidate prefix and the query
    fn ed(&self, query_len: u8) -> u8 {
        let ed_index = query_len - self.start;
        if ed_index >= self.distances.len() as u8 {
            // the ed is outside of the computation range determined by the current PED threshold
            // so it doesn't matter what it is as long as it's worse than everything else
            u8::MAX
        } else {
            // should be the last one in the row
            debug_assert_eq!(ed_index, self.distances.len() as u8 - 1);
            self.distances[ed_index as usize]
        }
    }
    /// Returns new partial row for the `character`
    ///
    /// `replace_dist` returns the edit distance for replacing a character in the candidate to match the query
    /// and is used to specialize `new` for characters that do not match any in the query
    #[inline]
    fn new(
        previous: &PartialRow,
        range: Range<usize>,
        replace_dist: impl Fn(&PartialCell, &PartialRow) -> u8,
    ) -> Self {
        debug_assert_ne!(range.len(), 0);

        let mut distances = Vec::<u8>::with_capacity(range.len());

        let mut cell = PartialCell {
            col: range.start,
            start: range.start,
            previous_start: previous.start as usize,
        };

        let previous_end = previous.distances.len() + previous.start as usize;
        // end of range where a column has filled cells in both the previous and current row
        let common_end = min(range.end, previous_end);

        // range.start .. common_end
        if range.start < common_end {
            // get the distances from the cell above-left if available and above
            // the cell to the left is unavailable
            let mut first_dist = cell.erase_dist(previous);
            if cell.previous_index() > 0 {
                first_dist = min(first_dist, replace_dist(&cell, previous));
            }

            distances.push(first_dist);
            for col in range.start + 1..common_end {
                cell.col = col;

                let dist = min(
                    // cells above-left, above, and left are available
                    replace_dist(&cell, previous),
                    min(cell.erase_dist(previous), cell.insert_dist(&distances)),
                );

                distances.push(dist);
            }
        }

        // common_end .. range.end
        if common_end < range.end {
            // there should only be at most one cell to the right of the previous row's last cell
            debug_assert!(common_end == range.end - 1);
            cell.col = range.end - 1;
            // cell above-left is available
            let mut last_dist = replace_dist(&cell, previous);
            // it is also possible for a partial row to be entirely to the right of the previous,
            // which is why this check is necessary to get the left cell's distance
            if range.start != common_end {
                last_dist = min(last_dist, cell.insert_dist(&distances));
            }
            distances.push(last_dist);
        }

        debug_assert_eq!(
            distances.len(),
            range.len(),
            "Did not compute a distance for every index in the partial row range"
        );

        //dbg!(&previous, &range, &distances);

        Self {
            start: range.start as u8,
            distances,
        }
    }
    /// Returns new row for edit distances between an empty string and the query
    fn initial(query: &Vec<char>) -> Self {
        Self {
            start: 0,
            distances: (0..=query.len() as u8).collect(),
        }
    }
    /// Returns new partial row for a character that doesn't match any in the query
    fn new_mismatch(previous: &PartialRow, range: Range<usize>) -> Self {
        // don't bother checking to see if there was a match in the query
        Self::new(previous, range, PartialCell::replace_mismatch_dist)
    }
    /// Returns new partial row for a character that matches any in the query
    fn new_match(
        previous: &PartialRow,
        range: Range<usize>,
        query: &Vec<char>,
        character: char,
    ) -> Self {
        Self::new(previous, range, move |cell, previous| {
            cell.replace_dist(query, character, previous)
        })
    }
}

/// Contains information to compute the edit distance in one cell of a partial Wagner-Fischer row
struct PartialCell {
    col: usize,
    start: usize,
    previous_start: usize,
}

impl PartialCell {
    #[inline(always)]
    /// Returns index into the full previous row for the same column
    fn previous_index(&self) -> usize {
        self.col - self.previous_start
    }
    #[inline(always)]
    /// Returns index into the current partial row
    fn current_index(&self) -> usize {
        self.col - self.start
    }
    #[inline(always)]
    /// Returns cost of replacing current character with the character in the query
    fn replace_dist(&self, query: &Vec<char>, character: char, previous: &PartialRow) -> u8 {
        // the column at 1 corresponds to the first character of the query
        let diff = character != query[self.col - 1];
        previous.distances[self.previous_index() - 1] + diff as u8
    }
    #[inline(always)]
    /// Returns cost of erasing the current character to match the query prefix
    fn erase_dist(&self, previous: &PartialRow) -> u8 {
        previous.distances[self.previous_index()] + 1
    }
    #[inline(always)]
    /// Returns cost of inserting a character in the query to match the query prefix
    fn insert_dist(&self, current_distances: &Vec<u8>) -> u8 {
        current_distances[self.current_index() - 1] + 1
    }
    #[inline(always)]
    /// Returns cost of replacing a mismatch with the character in the query
    fn replace_mismatch_dist(&self, previous: &PartialRow) -> u8 {
        previous.distances[self.previous_index() - 1] + 1
    }
}

#[derive(Default)]
struct PartialRowVariants {
    /// Partial rows computed for characters that match any character in the query
    matching: HashMap<char, PartialRow>,
    /// Partial row for characters that do not match any character in the query
    ///
    /// When the algorithm backtracks and moves on to another mismatch, it does not
    /// need to recompute the row
    mismatch: Option<PartialRow>,
}

impl PartialRowVariants {
    /// Returns wrapper around first partial row of length `threshold` between an empty string and `query`
    fn new(query: &Vec<char>) -> Self {
        Self {
            matching: Default::default(),
            mismatch: Some(PartialRow::initial(query)),
        }
    }
    /// Returns `Option` that may contain a reference to the `PartialRow`` for the given variant
    fn get(&self, variant: PartialRowVariantInfo) -> Option<&PartialRow> {
        use PartialRowVariantInfo::*;
        match variant {
            Matching(character) => self.matching.get(&character),
            Mismatch => self.mismatch.as_ref(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum PartialRowVariantInfo {
    Matching(char),
    Mismatch,
}

/// Contains edit distances between candidate prefixes and the query
struct PedMatrix {
    rows: Vec<PartialRowVariants>,
    /// Index of the current row
    current: usize,
    /// The minimum edit distances up to the current prefix, used for the PED
    min_eds: Vec<u8>,
}

impl PedMatrix {
    /// Truncates to `len` rows and `len - 1` eds
    fn truncate(&mut self, len: usize) {
        // this assumes that a new row may or may not be pushed
        self.rows.truncate(len);
        // this assumes that an edit distance will be pushed
        self.min_eds.truncate(len - 1);
    }
    /// Returns the minimum edit distance between the query and prefixes up to the current
    fn min_ed(&self) -> u8 {
        self.min_eds[self.current]
    }
    /// Returns the minimum edit distance between extended string prefixes and the query
    fn get_min_ed_after(&self, variant: PartialRowVariantInfo) -> u8 {
        self.rows[self.current].get(variant).unwrap().min_ed_after()
    }
    /// Returns a PedMatrix with an edit distance row between an empty candidate prefix and the query
    fn new(query: &Vec<char>) -> Self {
        let first = PartialRowVariants::new(query);
        Self {
            rows: vec![first],
            current: 0,
            min_eds: vec![query.len() as u8],
        }
    }
    /// Returns an option with the range for the next partial row, or None if it is not worth searching
    fn next_range(
        &self,
        variant: PartialRowVariantInfo,
        threshold: u8,
        query: &Vec<char>,
    ) -> Option<Range<usize>> {
        let min_ed_after = self.get_min_ed_after(variant);
        if min_ed_after > threshold {
            return None;
        }
        let next = self.current + 1;
        let current_row = self.rows[self.current].get(variant).unwrap();

        let left = threshold as usize;
        let right = threshold as usize;

        // the bounds are meant to exclude cells that will have EDs higher than the threshold, which are not useful
        let start = max(current_row.start as usize, next.saturating_sub(left));
        let end = min(
            next + right,
            min(
                // length of full row up to end of partial row
                // this will allow the next partial row to have at most one cell to the right of the previous
                current_row.start as usize + current_row.distances.len(),
                query.len(),
            ),
        ) + 1;

        let range = start..end;
        if range.is_empty() {
            None
        } else {
            Some(range)
        }
    }
    /// Updates the minimum edit distance between candidate prefixes using the next row
    fn next_min_ed(&mut self, next_row: &PartialRow, query: &Vec<char>) {
        let last = *self.min_eds.last().unwrap();
        let next_ed = next_row.ed(query.len() as u8);
        self.min_eds.push(min(last, next_ed));
    }
    /// Returns variant information to get the next row, generating it if necessary,
    ///
    /// `character`: The character of the candidate string for the row
    ///
    /// `threshold`: The maximum PED of a string that can be ranked
    fn next_row(
        &mut self,
        variant: PartialRowVariantInfo,
        range: Range<usize>,
        query: &Vec<char>,
        query_charset: &HashSet<char>,
        character: char,
    ) -> PartialRowVariantInfo {
        use PartialRowVariantInfo::*;
        let next = self.current + 1;

        if self.rows.len() > next {
            // if the row has already been computed for the character,
            // or if it has been computed for a character that also doesn't match anything,
            // don't recompute
            if self.rows[next].matching.contains_key(&character) {
                self.current = next;
                return Matching(character);
            }
            if self.rows[next].mismatch.is_some() && !query_charset.contains(&character) {
                self.current = next;
                return Mismatch;
            }
            // all the rows after the next need to be recomputed because we're recomputing the next one
            self.truncate(next + 1);
        } else {
            self.rows.push(Default::default());
        }

        // this should not be recomputed each loop, may need to move out
        let current_row = self.rows[self.current].get(variant).unwrap();
        self.current = next;

        let has_match = query_charset.contains(&character);
        let next_row = if has_match {
            PartialRow::new_match(current_row, range, query, character)
        } else {
            PartialRow::new_mismatch(current_row, range)
        };

        self.next_min_ed(&next_row, query);

        if has_match {
            let mut matching = HashMap::new();
            matching.insert(character, next_row);
            self.rows[next] = PartialRowVariants {
                matching,
                mismatch: None,
            };
            Matching(character)
        } else {
            self.rows[next] = PartialRowVariants {
                matching: Default::default(),
                mismatch: Some(next_row),
            };
            Mismatch
        }
    }
}
