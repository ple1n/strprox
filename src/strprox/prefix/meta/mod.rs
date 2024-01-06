use std::{
    borrow::Cow,
    cmp::{max, min},
    collections::{hash_map, HashMap, HashSet},
    marker::PhantomData,
    ops::Range,
};

use super::{MeasuredPrefix, FromStrings};
use crate::{levenshtein, Autocompleter};

use debug_print::debug_println;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use yoke::{Yoke, Yokeable};

//mod compact_tree;

/// Implements "Matching-Based Method for Error-Tolerant Autocompletion" (META) from https://doi.org/10.14778/2977797.2977808

// Arithmetic using generics/traits is cumbersome in Rust
// These are here to have inlay type hints in my IDE, which are missing when a macro is added for them
// They are three repeated letters to easily search and replace later to add macros
/// Type that bounds the length of a stored string
type UUU = u8;
/// Type that bounds the number of stored strings
type SSS = u32;

/// A trie node with a similar structure from META
#[derive(PartialEq, Eq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
struct Node<UUU, SSS> {
    /// One Unicode character
    character: char,
    /// Range of indices into descendant nodes
    descendant_range: Range<SSS>,
    /// Range of indices into strings with the prefix from this node
    string_range: Range<SSS>,
    /// Length of the prefix from this node
    depth: UUU,
}

impl Node<UUU, SSS> {
    /// Returns the index into the trie where the node is
    #[inline]
    fn id(&self) -> usize {
        self.descendant_range.start as usize - 1
    }
    #[inline]
    /// Returns the id of the first child/descendant, which is equivalent to the id for sorting
    fn first_descendant_id(&self) -> usize {
        self.descendant_range.start as usize
    }
}

pub type TreeString<'stored> = Cow<'stored, str>;
type TrieStrings<'stored> = Vec<TreeString<'stored>>;
type TrieNodes<UUU, SSS> = Vec<Node<UUU, SSS>>;

pub trait TreeStringT<'a>: 'a + Clone {
    fn from_string(sx: &'a String) -> Self;
    fn to_str<'s>(&'s self) -> &'s str;
    fn from_owned(sx: String) -> Self;
}

impl<'a> TreeStringT<'a> for Cow<'a, str> {
    fn from_string(sx: &'a String) -> Self {
        Cow::Borrowed(sx.as_str())
    }
    fn to_str<'s>(&'s self) -> &'s str {
        &self
    }
    fn from_owned(sx: String) -> Self {
        Cow::Owned(sx)
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Trie<'stored, UUU, SSS> {
    nodes: TrieNodes<UUU, SSS>,
    #[cfg_attr(feature = "serde", serde(borrow))]
    /// Stored strings
    pub strings: TrieStrings<'stored>,
}

/// Returns an Option with the next valid Unicode scalar value after `character`, unless `character` is char::MAX
#[inline]
fn char_succ(character: char) -> Option<char> {
    let mut char_range = character..=char::MAX;
    char_range.nth(1)
}

impl<'stored> Trie<'stored, UUU, SSS> {
    /// Returns the root node of the trie (panics if the trie is empty)
    fn root(&self) -> &Node<UUU, SSS> {
        // this shouldn't be able to panic from the public API
        self.nodes.first().unwrap()
    }
    /// Returns trie over `source` (expects `source` to have at most usize::MAX - 1 strings)
    pub fn new(len: usize, source: impl IntoIterator<Item = TreeString<'stored>>) -> Self {
        let mut strings = TrieStrings::<'stored>::with_capacity(len);
        for string in source.into_iter() {
            strings.push(string);
        }
        // sort and dedup to compute the `string_range` for each node using binary search
        strings.sort();
        strings.dedup();

        // rough estimate on the size of the trie
        let nodes = TrieNodes::with_capacity(3 * len);

        let mut trie = Self { strings, nodes };

        // Construct all nodes
        trie.init_nodes(
            &mut 0,
            0,
            &mut Default::default(),
            '\0',
            0,
            0,
            trie.strings.len(),
        );
        trie
    }
    /// `last_char` is the last character in the prefix
    fn init_nodes(
        &mut self,
        node_id: &mut usize,
        depth: UUU,
        prefix: &mut String,
        last_char: char,
        suffix_start: usize,
        start: usize,
        end: usize,
    ) {
        let current_id = node_id.clone();

        let current_node: Node<u8, u32> = Node::<UUU, SSS> {
            character: last_char,
            // change the descendant range later
            descendant_range: Default::default(),
            string_range: start as SSS..end as SSS,
            depth,
        };
        // the current node is added before all the descendants,
        // and its location in `nodes` is `current_id`
        debug_assert_eq!(self.nodes.len(), current_id);
        self.nodes.push(current_node);

        // the next node, if it exists, will have 1 higher id
        *node_id += 1;

        // `node_id` is required to be incremented in pre-order to have continuous `descendant_range``
        let mut child_start = start;
        while child_start != end {
            // add to the prefix
            let suffix = &self.strings[child_start][suffix_start..];
            if let Some(next_char) = suffix.chars().next() {
                // strings in strings[child_start..child_end] will have the same prefix
                let child_end;
                let next_prefix;

                // get the boundary in `strings` for strings with the prefix extended with next_char
                if let Some(succ) = char_succ(next_char) {
                    // `lexicographic_marker` is the first string that's lexicographically ordered after all strings with prefix
                    let lexicographic_marker = &mut *prefix;
                    lexicographic_marker.push(succ);

                    // offset from start where the lexicographic marker would be
                    let offset;
                    match self.strings[start..end]
                        .binary_search(&TreeStringT::from_string(&lexicographic_marker))
                    {
                        // same bound either way, but if it's Err it will be the last iteration
                        Ok(x) => offset = x,
                        Err(x) => offset = x,
                    }
                    debug_assert_eq!(
                        offset,
                        self.strings[start..end].partition_point(
                            |string| string < &TreeStringT::from_string(&lexicographic_marker)
                        )
                    );
                    child_end = start + offset;

                    debug_assert!(child_end > child_start);

                    next_prefix = lexicographic_marker;
                    next_prefix.pop();
                } else {
                    // the next character in the prefix is char::MAX,
                    // so this must be the last prefix from the current one
                    debug_assert_eq!(next_char, char::MAX);
                    child_end = end;
                    next_prefix = prefix;
                }
                next_prefix.push(next_char);

                // requires nightly
                //let next_suffix_start = strings[start].ceil_char_boundary(suffix_start + 1);

                let next_suffix_start = suffix_start + next_char.len_utf8();

                // Construct all descendant nodes with the next prefix
                self.init_nodes(
                    node_id,
                    depth + 1,
                    next_prefix,
                    next_char,
                    next_suffix_start,
                    child_start,
                    child_end,
                );

                // reset the prefix state
                let prefix = next_prefix;
                prefix.pop();

                // look at strings with a different next character in their prefix
                child_start = child_end;
            } else {
                // this string has already been accounted for by the parent node,
                // whose prefix is equal to the whole string
                child_start += 1;
            }
        }

        // node_id is now 1 greater than the index of the last in-order node that's in the subtree from the current node
        let descendant_range = current_id as SSS + 1..*node_id as SSS;
        self.nodes[current_id].descendant_range = descendant_range;
    }
}

/// Inverted index from META
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
struct InvertedIndex<UUU, SSS> {
    /// depth |-> (character |-> nodes ids in trie)
    index: Vec<HashMap<char, Vec<SSS>>>,
    /// Marker to allow macros to specialize methods for UUU
    u_marker: PhantomData<UUU>,
}

impl InvertedIndex<UUU, SSS> {
    /// Constructs an inverted index from depth to character to nodes using a trie
    fn new(trie: &Trie<UUU, SSS>) -> Self {
        let mut max_depth = 0;
        for node in &trie.nodes {
            max_depth = max(max_depth, node.depth as usize);
        }

        let mut index = Vec::<HashMap<char, Vec<SSS>>>::with_capacity(max_depth + 1);
        index.resize(max_depth + 1, Default::default());

        // put all nodes into the index at a certain depth and character
        for node in &trie.nodes {
            let depth = node.depth as usize;
            let char_map = &mut index[depth];
            if let Some(nodes) = char_map.get_mut(&node.character) {
                nodes.push(node.id() as SSS);
            } else {
                char_map.insert(node.character, vec![node.id() as SSS]);
            }
        }
        // sort the nodes by id for binary search (cache locality with Vec)
        for char_map in &mut index {
            for (_, nodes) in char_map {
                nodes.sort_unstable();
            }
        }
        Self {
            index,
            u_marker: PhantomData,
        }
    }
    /// Returns the node ids with `depth` and `character`
    fn get(&self, depth: usize, character: char) -> Option<&Vec<SSS>> {
        self.index[depth].get(&character)
    }
    /// Returns maximum depth of nodes stored in the index
    fn max_depth(&self) -> usize {
        self.index.len() - 1
    }
}

/// Structure that allows for autocompletion based on a string dataset
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Yokeable)]
pub struct MetaAutocompleter<'stored, UUU = u8, SSS = u32> {
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub trie: Trie<'stored, UUU, SSS>,
    inverted_index: InvertedIndex<UUU, SSS>,
}

impl<'stored> MetaAutocompleter<'stored, UUU, SSS> {
    /// Constructs an Autocompleter given the string dataset `source` (does not copy strings)
    pub fn new(len: usize, source: impl IntoIterator<Item = TreeString<'stored>>) -> Self {
        let trie = Trie::<'stored, UUU, SSS>::new(len, source);
        let inverted_index = InvertedIndex::<UUU, SSS>::new(&trie);
        Self {
            trie,
            inverted_index,
        }
    }
    pub fn len(&self) -> usize {
        self.trie.strings.len()
    }
}

#[derive(Clone)]
struct Matching<'stored, UUU, SSS> {
    query_prefix_len: UUU,
    node: &'stored Node<UUU, SSS>,
    edit_distance: UUU,
}

impl<'stored> Matching<'stored, UUU, SSS> {
    /// Returns an upper bound on the edit distance between the query and a prefix of length `stored_len` that intersects
    /// with the matching node's prefix
    fn deduced_edit_distance(&self, query_len: usize, stored_len: usize) -> usize {
        self.edit_distance as usize
            + max(
                query_len.saturating_sub(self.query_prefix_len as usize),
                stored_len.saturating_sub(self.node.depth as usize),
            )
    }
    /// Returns an upper bound on the edit distance between the query and the matching node's prefix
    fn deduced_prefix_edit_distance(&self, query_len: usize) -> usize {
        self.edit_distance as usize + query_len - self.query_prefix_len as usize
    }
}

#[derive(Debug, Default)]
struct MatchingSet<'stored, UUU, SSS> {
    /// Maps the first two parts of a matching to the edit distance
    pub matchings: HashMap<(UUU, &'stored Node<UUU, SSS>), UUU>,
}

impl<'stored> MatchingSet<'stored, UUU, SSS> {
    /// Inserts `matching` into the MatchingSet
    fn insert(&mut self, matching: Matching<'stored, UUU, SSS>) {
        self.matchings.insert(
            (matching.query_prefix_len, matching.node),
            matching.edit_distance,
        );
    }
    /// Returns an iterator over the matchings
    fn iter(&self) -> MatchingSetIter<'_, 'stored, UUU, SSS> {
        MatchingSetIter {
            iter: self.matchings.iter(),
        }
    }
    /// Returns whether there is a matching for `query_prefix_len` and `node`
    fn contains(&self, query_prefix_len: UUU, node: &'stored Node<UUU, SSS>) -> bool {
        self.matchings.contains_key(&(query_prefix_len, node))
    }
    /// Returns a matching set with a matching for the root of the `trie` and an empty query
    fn new(trie: &'stored Trie<'stored, UUU, SSS>) -> Self {
        let mut matchings = HashMap::<(UUU, &'stored Node<UUU, SSS>), UUU>::new();
        let query_prefix_len = 0;
        let node = trie.root();
        let edit_distance = 0;
        matchings.insert((query_prefix_len, node), edit_distance);
        Self { matchings }
    }
}

/// Iterator over the matchings in a MatchingSet
struct MatchingSetIter<'iter, 'stored, UUU, SSS> {
    iter: hash_map::Iter<'iter, (UUU, &'stored Node<UUU, SSS>), UUU>,
}

impl<'stored> Iterator for MatchingSetIter<'_, 'stored, UUU, SSS> {
    type Item = Matching<'stored, UUU, SSS>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((&(query_prefix_len, node), &edit_distance)) = self.iter.next() {
            Some(Matching {
                query_prefix_len,
                node,
                edit_distance,
            })
        } else {
            None
        }
    }
}

impl<'stored> MetaAutocompleter<'stored, UUU, SSS> {
    /// Returns the top `requested` number of strings with the best prefix distance from the query
    /// sorted by prefix edit distance and then lexicographical order,
    /// or all strings available if `requested` is larger than the number stored
    ///
    /// Assumes `query`'s length in Unicode characters is bounded by UUU; will truncate to UUU::MAX characters otherwise
    pub fn autocomplete(&'stored self, query: &str, requested: usize) -> Vec<MeasuredPrefix> {
        self.threshold_topk(query, requested, usize::MAX)
    }
    /// Returns the top `requested` number of strings with the best prefix distance from the query
    /// sorted by prefix edit distance and then lexicographical order,
    /// or all strings available if `requested` is larger than the number stored
    ///
    /// Assumes `query`'s length in Unicode characters is bounded by UUU; will truncate to UUU::MAX characters otherwise
    pub fn threshold_topk(
        &'stored self,
        query: &str,
        requested: usize,
        max_threshold: usize,
    ) -> Vec<MeasuredPrefix> {
        if requested == 0 {
            return Default::default();
        }

        // Return the first `requested` strings or as many as possible, and 0 as the PED (because the best prefix is empty)
        if query.is_empty() {
            let strings = &self.trie.strings;
            let string_range = 0..min(requested, strings.len());
            return string_range
                .map(|index| {
                    let string = strings[index].to_string();
                    MeasuredPrefix {
                        string,
                        prefix_distance: 0,
                    }
                })
                .collect();
        }

        let mut query_chars: Vec<char> = query.chars().collect();
        if query_chars.len() > UUU::MAX as usize {
            query_chars.truncate(UUU::MAX as usize);
        }

        let mut result = HashSet::<TreeString<'stored>>::new();
        let mut threshold = 0;
        let mut active_matching_set = MatchingSet::<'stored, UUU, SSS>::new(&self.trie);

        let format_result = |result: HashSet<TreeString>| {
            let mut result: Vec<MeasuredPrefix> = result
                .into_iter()
                .map(|string| MeasuredPrefix {
                    string: string.to_string(),
                    prefix_distance: levenshtein::prefix_edit_distance(
                        query,
                        TreeStringT::to_str(&string),
                    ),
                })

                // it seems that META doesn't actually bound the full PED until near the end when
                // query_prefix_len == query.len(), so a filter is necessary
                .filter(|measure| measure.prefix_distance <= max_threshold)

                .collect();

            result.sort();
            result
        };
        for query_prefix_len in 1..=query_chars.len() {
            result.clear();
            threshold = self.autocomplete_step(
                &mut active_matching_set,
                &query_chars,
                query_prefix_len,
                threshold,
                max_threshold,
                requested,
                &mut result,
            );
            if threshold > max_threshold {
                return format_result(result);
            }
        }
        format_result(result)
    }
    /// Adds the strings prefixed by `node` to `result` until all have been added or the `requested` size has been reached
    ///
    /// Returns whether the `requested` size has been reached
    fn fill_results(
        &self,
        node: &Node<UUU, SSS>,
        result: &mut HashSet<TreeString<'stored>>,
        requested: usize,
    ) -> bool {
        if requested == 0 {
            return true;
        }
        debug_assert_ne!(result.len(), requested);

        for string_index in node.string_range.clone() {
            result.insert(self.trie.strings[string_index as usize].clone());
            if result.len() >= requested {
                return true;
            }
        }
        debug_assert_ne!(result.len(), requested);
        false
    }
    /// Performs a single step of autocomplete for one character of a query
    ///
    /// Returns the new `threshold` for the maximum prefix edit distance in the result set
    fn autocomplete_step(
        &'stored self,
        active_matching_set: &mut MatchingSet<'stored, UUU, SSS>,
        query: &[char],
        query_len: usize,
        threshold: usize,
        max_threshold: usize,
        requested: usize,
        result: &mut HashSet<TreeString<'stored>>,
    ) -> usize {
        // this may need to become public along with MatchingSet to support result caching for previous query prefixes
        let character = query[query_len - 1];

        debug_println!("A {} Tau {}", active_matching_set.matchings.len(), threshold);

        *active_matching_set = self.first_deducing(
            active_matching_set,
            character,
            query_len,
            threshold.saturating_sub(1),
        );

        for matching in active_matching_set.iter() {
            let prefix_edit_distance = matching.deduced_prefix_edit_distance(query_len);
            if prefix_edit_distance < threshold {
                if self.fill_results(matching.node, result, requested) {
                    return threshold;
                }
            }
        }

        if self.second_deducing(
            active_matching_set,
            query,
            query_len,
            result,
            threshold,
            requested,
        ) {
            threshold
        } else {
            // don't bother looking for results with a higher PED
            if threshold == max_threshold {
                return threshold + 1;
            }
            let full = self.second_deducing(
                active_matching_set,
                query,
                query_len,
                result,
                threshold + 1,
                requested,
            );
            debug_assert!(full);
            threshold + 1
        }
    }
    /// Applies the `visitor` function to all descendants in the inverted index at `depth` and `character` of `matching.node`
    fn traverse_inverted_index<VisitorFn>(
        &'stored self,
        matching: &Matching<'stored, UUU, SSS>,
        depth: usize,
        character: char,
        mut visitor: VisitorFn,
    ) where
        VisitorFn: FnMut(&'stored Node<UUU, SSS>),
    {
        let node = matching.node;
        if let Some(nodes) = self.inverted_index.get(depth, character) {
            // get the index where the first descendant of the node would be
            let start = nodes.partition_point(|&id| id < node.first_descendant_id() as SSS);

            // get the index of where the first node after all descendants would be
            let end = nodes.partition_point(|&id| id < node.descendant_range.end);

            let descendant_ids = &nodes[start..end];

            for &descendant_id in descendant_ids {
                visitor(&self.trie.nodes[descendant_id as usize]);
            }
        }
    }
    /// Returns the next b-matching set for a query extended by one character
    /// (loop step of MatchingBasedFramework from META without any removal from the matching set)
    fn first_deducing(
        &'stored self,
        active_matching_set: &MatchingSet<'stored, UUU, SSS>,
        character: char,
        query_len: usize,
        threshold: usize,
    ) -> MatchingSet<'stored, UUU, SSS> {
        let mut best_edit_distances = HashMap::<SSS, UUU>::new();
        for matching in active_matching_set.iter() {
            let node = matching.node;
            let node_prefix_len = node.depth as usize;
            // lines 5-7 of MatchingBasedFramework, also used in SecondDeducing
            for depth in node_prefix_len + 1
                ..=min(
                    node_prefix_len + threshold + 1,
                    self.inverted_index.max_depth(),
                )
            {
                self.traverse_inverted_index(&matching, depth, character, |descendant| {
                    // the depth of a node is equal to the length of its associated prefix
                    let bound = matching.deduced_edit_distance(
                        query_len - 1,
                        node.depth.saturating_sub(1) as usize,
                    );
                    let bound = bound as UUU;
                    let id = descendant.id() as SSS;
                    if bound <= threshold as UUU {
                        if let Some(edit_distance) = best_edit_distances.get_mut(&id) {
                            *edit_distance = min(*edit_distance, bound);
                        } else {
                            best_edit_distances.insert(id, bound);
                        }
                    }
                });
            }
        }
        let mut new_active_matching_set = MatchingSet::<'stored, UUU, SSS>::default();
        // lines 8-9
        for (node_id, edit_distance) in best_edit_distances {
            let query_prefix_len = query_len as UUU;
            let node = &self.trie.nodes[node_id as usize];
            let matching = Matching::<'stored, UUU, SSS> {
                query_prefix_len, // i'' = i
                node,
                edit_distance,
            };
            // the second kind
            new_active_matching_set.insert(matching);
        }
        // lines 10-11
        for matching in active_matching_set.iter() {
            // this condition appears to be for threshold-based autocomplete and not for top-k in META
            //if matching.deduced_prefix_edit_distance(query_len) <= threshold {
            new_active_matching_set.insert(matching); // i'' < i
            // the first kind
            //}
        }
        new_active_matching_set
    }
    /// Implements SecondDeducing from META
    ///
    /// Returns whether `result` was filled to `requested`
    fn second_deducing(
        &'stored self,
        active_matching_set: &mut MatchingSet<'stored, UUU, SSS>,
        query: &[char],
        query_len: usize,
        result: &mut HashSet<TreeString<'stored>>,
        threshold: usize,
        requested: usize,
    ) -> bool {
        type Appendix<'stored> = Vec<Matching<'stored, UUU, SSS>>;
        // need to specify `active_matching_set` as a parameter to have compatible borrow lifetimes with mut and immutable uses
        let mut per_matching = |active_matching_set: &MatchingSet<'stored, UUU, SSS>,
                                matching: Matching<'stored, UUU, SSS>,
                                appendix: &mut Appendix<'stored>| {
            let max_ped = matching.deduced_prefix_edit_distance(query_len) as UUU;
            // lines 2-4 of SecondDeducing
            if max_ped == threshold as UUU {
                if self.fill_results(matching.node, result, requested) {
                    return true;
                }
            }

            let last_depth = min(
                matching.node.depth as usize + threshold - matching.edit_distance as usize + 1,
                self.inverted_index.max_depth(),
            );
            let last_query_prefix_len = min(
                matching.query_prefix_len as usize + threshold - matching.edit_distance as usize
                    + 1,
                query_len,
            );

            // line 5
            let mut append = |descendant: &'stored Node<UUU, SSS>, query_prefix_len: usize| {
                if !active_matching_set.contains(query_prefix_len as UUU, descendant)
                    && matching
                        .deduced_edit_distance(query_prefix_len - 1, descendant.depth as usize - 1)
                        == threshold
                {
                    let matching = Matching::<'stored, UUU, SSS> {
                        query_prefix_len: query_prefix_len as UUU,
                        node: descendant,
                        edit_distance: threshold as UUU,
                    };
                    appendix.push(matching);
                }
            };
            for query_prefix_len in matching.query_prefix_len as usize + 1..=last_query_prefix_len {
                let character = query[query_prefix_len - 1];
                self.traverse_inverted_index(&matching, last_depth, character, |descendant| {
                    append(descendant, query_prefix_len)
                });
            }

            let last_character = query[last_query_prefix_len - 1];
            for depth in matching.node.depth as usize + 1..=last_depth {
                self.traverse_inverted_index(&matching, depth, last_character, |descendant| {
                    append(descendant, last_query_prefix_len)
                });
            }

            false
        };

        // Can't simultaneously insert into the matching set while iterating through it
        let mut appendix = Vec::<Matching<'stored, UUU, SSS>>::new();
        for matching in active_matching_set.iter() {
            if per_matching(active_matching_set, matching, &mut appendix) {
                return true;
            }
        }
        while !appendix.is_empty() {
            // Is it possible that this would append anything after the first loop earlier?
            // Not sure if appending more matchings is necessary
            let mut new_appendix = Vec::<Matching<'stored, UUU, SSS>>::new();
            let mut full = false;
            let mut into_appendix = appendix.into_iter();
            // process everything in appendix
            while let Some(matching) = into_appendix.next() {
                if per_matching(active_matching_set, matching.clone(), &mut new_appendix) {
                    full = true;
                }
                active_matching_set.insert(matching);
                if full {
                    break;
                }
            }
            // add everything from appendix (the algorithm as written seems to insert a matching immediately from the earlier loop)
            for matching in into_appendix {
                active_matching_set.insert(matching);
            }
            if full {
                for matching in new_appendix {
                    active_matching_set.insert(matching);
                }
                return true;
            }
            // new_appendix is moved to appendix for further processing
            appendix = new_appendix;
        }
        false
    }
}

impl Autocompleter for Yoke<MetaAutocompleter<'static>, Vec<String>> {
    fn threshold_topk(
        &self,
        query: &str,
        requested: usize,
        max_threshold: usize,
    ) -> Vec<MeasuredPrefix> {
        self.get().threshold_topk(query, requested, max_threshold)
    }
}

impl FromStrings for Yoke<MetaAutocompleter<'static>, Vec<String>> {
    fn from_strings(strings: &[&str]) -> Self {
        let cart = strings.iter().map(|&s| s.to_string()).collect();
        Yoke::attach_to_cart(cart, |strings| {
            let cows: Vec<_> = strings.iter().map(Into::into).collect();
            MetaAutocompleter::new(cows.len(), cows)
        })
    }
}
