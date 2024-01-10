use std::{borrow::Cow, io::Read, time::Instant};

use strprox::MetaAutocompleter;

fn test_depth(strs: &Vec<&str>) {
    let cows = strs.into_iter().map(|s| Cow::Borrowed(*s));
    let meta = MetaAutocompleter::new(cows.len(), cows);
    println!("Trie strings {}", meta.trie.strings.len());
    let start = Instant::now();
    let v = meta.depth_autocomplete("oange", 5, 2);
    dbg!(v);
    println!("took {} ms", start.elapsed().as_millis());
}


fn main() {
    let mut words_file = std::fs::File::open("./src/tests/words.txt").unwrap();
    let mut words = Default::default();
    words_file.read_to_string(&mut words).unwrap();
    let words: Vec<&str> = words.split("\n").collect();
    dbg!(words.len());
    test_depth(&words);
}
