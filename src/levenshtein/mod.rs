use crate::{MeasuredPrefix, TreeString};
use std::{cmp::min, collections::BinaryHeap};

#[cfg(test)]
use rand::{
    distributions::{Distribution, Uniform},
    Rng,
};

/// Returns the last row of the Levenshtein edit distance matrix between two strings as char-slices,
/// where the row is for the edit distances between varying prefixes of `second` with a certain prefix of `first`
pub(super) fn final_lev_row(first: &[char], second: &[char]) -> Vec<usize> {
    // using a two-row memoization https://en.wikipedia.org/wiki/Levenshtein_distance#Iterative_with_two_matrix_rows
    let row_len = second.len() + 1;
    let mut prev_row = Vec::with_capacity(row_len);
    prev_row.resize(row_len, 0);
    let mut current_row = prev_row.clone();

    for j in 0..=second.len() {
        prev_row[j] = j;
    }

    for row in 1..=first.len() {
        current_row[0] = row;
        for column in 1..=second.len() {
            // compare the characters at i - 1 and j - 1
            // (edit_matrix[0, 0] is the edit distance between two empty strings, so i and j are offset by 1)
            let first_char = &first[row - 1];
            let second_char = &second[column - 1];
            let diff = (first_char != second_char) as usize;

            let replace_dist = prev_row[column - 1] + diff;
            let insert_dist = prev_row[column] + 1;
            let erase_dist = current_row[column - 1] + 1;

            let dist = min(replace_dist, min(insert_dist, erase_dist));

            current_row[column] = dist;
        }
        std::mem::swap(&mut prev_row, &mut current_row);
    }
    prev_row
}

/// Returns `string` as a Vec of its characters
pub(crate) fn to_char_vec(string: &str) -> Vec<char> {
    string.chars().collect()
}

/// Returns the prefix edit distance between two strings, where the prefixes of `second` vary
/// (refer to the paper by Deng et al.)
pub fn prefix_edit_distance(first: &str, second: &str) -> usize {
    let first: Vec<char> = to_char_vec(first);
    let second: Vec<char> = to_char_vec(second);
    match final_lev_row(&first[..], &second[..]).into_iter().min() {
        Some(distance) => distance,
        None => {
            // If it's None, it means that at least one of the strings are empty, so the edit distance
            // is the number of characters to insert into an empty string from `first`
            debug_assert!(first.is_empty() || second.is_empty());
            first.len()
        }
    }
}

pub fn prefix_edit_distance_chars(first: &Vec<char>, second: &Vec<char>) -> usize {
    match final_lev_row(&first[..], &second[..]).into_iter().min() {
        Some(distance) => distance,
        None => {
            // If it's None, it means that at least one of the strings are empty, so the edit distance
            // is the number of characters to insert into an empty string from `first`
            debug_assert!(first.is_empty() || second.is_empty());
            first.len()
        }
    }
}

/// Returns the edit distance between two char slices
pub fn edit_distance(first: &str, second: &str) -> usize {
    let first: Vec<char> = to_char_vec(first);
    let second: Vec<char> = to_char_vec(second);
    match final_lev_row(&first[..], &second[..]).last() {
        Some(&distance) => distance,
        None => {
            debug_assert!(second.is_empty());
            first.len()
        }
    }
}

pub fn edit_distance_chars(first: &[char], second: &[char]) -> usize {
    match final_lev_row(&first[..], &second[..]).last() {
        Some(&distance) => distance,
        None => {
            debug_assert!(second.is_empty());
            first.len()
        }
    }
}

/// Baseline autocomplete using the PED that doesn't use an index
pub fn unindexed_autocomplete(
    query: &str,
    requested: usize,
    strings: &[TreeString],
) -> Vec<MeasuredPrefix> {
    let mut best = BinaryHeap::new();
    strings.iter().for_each(|string| {
        let measure = MeasuredPrefix {
            string: string.to_string(),
            prefix_distance: prefix_edit_distance(query, &string),
        };
        best.push(measure);
        if best.len() > requested {
            best.pop();
        }
    });
    best.into_sorted_vec()
}

/// Returns a string with a number of random `edits` to `string`
///
/// Does not guarantee that the edits are non-overlapping (edit distance may be less than `edits`)
#[cfg(test)]
pub(crate) fn random_edits(string: &str, edits: usize) -> String {
    use rand::distributions::Standard;

    let mut string: Vec<char> = string.chars().collect();

    let edit_type_distribution = Uniform::new_inclusive(1, 3);
    let char_distribution = Standard;
    let mut rng = rand::thread_rng();

    for _i in 0..edits {
        let character = char_distribution.sample(&mut rng);
        let edit_type;
        if string.is_empty() {
            // can only insert
            edit_type = 1;
        } else {
            // can insert, delete, or replace
            edit_type = edit_type_distribution.sample(&mut rng);
        }
        let index;
        if edit_type != 1 {
            // index for delete and replace
            index = rng.gen_range(0..string.len());
        } else {
            index = rng.gen_range(0..=string.len());
        }
        match edit_type {
            // insert
            1 => {
                string.insert(index, character);
            }
            // delete
            2 => {
                string.remove(index);
            }
            // replace
            3 => {
                string[index] = character;
            }
            _ => {}
        }
    }

    let mut result = String::with_capacity(string.len());
    for character in string {
        result.push(character);
    }
    result
}

/// Returns
/// (a random string from `source`,
/// a string with a random number of edits between 0 and 5,
/// number of edits made)
#[cfg(test)]
pub(crate) fn sample_edited_string<'s>(
    source: &'s [&str],
    rng: &mut impl Rng,
    ed: usize,
) -> (&'s str, String, usize) {
    use rand::seq::SliceRandom;

    let &string = source.choose(rng).unwrap();
    let edits_distribution = Uniform::new(0, ed);
    let edits = edits_distribution.sample(rng);
    let edited_string = random_edits(string, edits);
    (string, edited_string, edits)
}
