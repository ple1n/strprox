use std::{
    fs,
    io::Write,
    time::{Duration, Instant},
};

use fst::Set;
use rand::thread_rng;
use tracing::{debug, info};
use yoke::Yoke;

use crate::{
    levenshtein::{prefix_edit_distance, sample_edited_string, unindexed_autocomplete},
    prefix::FromStrings,
    strprox::FstAutocompleter,
    strprox::MetaAutocompleter,
    Autocompleter, MeasuredPrefix,
};

type YokedMetaAutocompleter = Yoke<MetaAutocompleter<'static>, Vec<String>>;

/// Returns whether any MeasuredPrefix in `measures` has the `expected` string
fn contains_string(measures: &Vec<MeasuredPrefix>, expected: &str) -> bool {
    measures.iter().any(|measure| measure.string == expected)
}

// Words from https://github.com/dwyl/english-words/blob/master/words.txt
const WORDS: &str = include_str!("words.txt");

#[generic_tests::define]
mod generic {
    use tracing::Level;
    use tracing_subscriber::FmtSubscriber;

    use super::*;
    use crate::{levenshtein, prefix::FromStrings, Autocompleter};

    #[test]
    /// Example input from the paper on META (see the citations)
    fn meta_paper_example<A>()
    where
        A: Autocompleter + FromStrings,
    {
        let source: Vec<_> = vec!["soho", "solid", "solo", "solve", "soon", "throw"];
        let mut state: A::STATE = Default::default();
        let autocompleter = A::from_strings(&source);
        let result = autocompleter.autocomplete("ssol", 3, &mut state);
        for measure in &result {
            println!("{:#?}", measure);
        }
        // these are the only strings with PEDs of 1
        assert!(contains_string(&result, "solid"));
        assert!(contains_string(&result, "solo"));
        assert!(contains_string(&result, "solve"));
    }
    #[test]
    /// Tests that autocomplete can return exact associated categories
    fn two_categories<A>()
    where
        A: Autocompleter + FromStrings,
    {
        let source: Vec<_> = vec![
            "success",
            "successor",
            "successive",
            "decrement",
            "decrease",
            "decreasing",
        ];
        let autocompleter = A::from_strings(&source);
        let query = "zucc";
        let mut state: A::STATE = Default::default();
        let result = autocompleter.autocomplete(query, 3, &mut state);
        println!("{}\n", query);
        for measure in &result {
            println!("{:#?}", measure);
        }
        let _ = std::io::stdout().flush();

        assert!(contains_string(&result, "success"));
        assert!(contains_string(&result, "successor"));
        assert!(contains_string(&result, "successive"));

        let cows: Vec<_> = source.iter().map(|&s| s.into()).collect();
        assert_eq!(result, unindexed_autocomplete("zucc", 3, &cows));

        let query = "deck";
        let result = autocompleter.autocomplete("deck", 3, &mut state);
        println!("{}\n", query);
        for measure in &result {
            println!("{:#?}", measure);
        }
        assert!(contains_string(&result, "decrement"));
        assert!(contains_string(&result, "decrease"));
        assert!(contains_string(&result, "decreasing"));

        assert_eq!(result, unindexed_autocomplete("deck", 3, &cows));
    }

    #[test]
    /// The example in the README
    fn example<A>()
    where
        A: Autocompleter + FromStrings,
    {
        let source: Vec<_> = vec![
            "success",
            "successive",
            "successor",
            "decrease",
            "decreasing",
            "decrement",
        ];
        let autocompleter = A::from_strings(&source);
        let mut state: A::STATE = Default::default();
        let query = "luck";
        let result = autocompleter.autocomplete(query, 3, &mut state);
        for measured_prefix in &result {
            println!("{}", measured_prefix);
        }
        let result_strings: Vec<&str> = result
            .iter()
            .map(|measured_prefix| measured_prefix.string.as_str())
            .collect();
        assert_eq!(result_strings, vec!["success", "successive", "successor"]);
    }

    /// Tests that autocomplete works for a prefix that only requires an insertion at the beginning
    #[test]
    fn insertion_ped<A>()
    where
        A: Autocompleter + FromStrings,
    {
        let query = "foob";
        // PEDs: [1, 2, 2]
        let source: Vec<_> = vec!["oobf", "fbor", "bobf"]
            .into_iter()
            .map(|k| k.into())
            .collect();
        let autocompleter = A::from_strings(&source);
        let mut state: A::STATE = Default::default();
        let result = autocompleter.autocomplete(query, 1, &mut state);
        for measure in &result {
            println!("{:#?}", measure);
        }
        assert_eq!(result[0].string, "oobf");
    }

    #[test]
    /// Tests for correction of a misspelling against a large database
    fn words_misspelling<A>()
    where
        A: Autocompleter + FromStrings,
    {
        let source: Vec<_> = WORDS.lines().collect();
        let time = Instant::now();
        let autocompleter = A::from_strings(&source);
        let mut state: A::STATE = Default::default();
        println!("Indexing took: {:#?}", time.elapsed());
        let requested = 10;
        let result = autocompleter.autocomplete("abandonned", requested, &mut state);
        assert_eq!(result.len(), requested);

        for measure in &result {
            println!("{:#?}", measure);
        }
        assert!(contains_string(&result, "abandoned"));
    }

    #[test]
    /// Tests for error-tolerant autocompletion against a large database
    fn words_autocomplete<A>()
    where
        A: Autocompleter + FromStrings,
    {
        let source: Vec<_> = WORDS.lines().collect();
        let autocompleter = A::from_strings(&source);
        let mut state: A::STATE = Default::default();
        let requested = 10;
        let query = "oberr";
        const MAX_PED: usize = 1;
        let result = autocompleter.threshold_topk(query, requested, MAX_PED, &mut state);

        /// Max PED of any top-10 strings from the dataset against the query "oberr"
        for measure in &result {
            println!("{:#?}", measure);
            assert!(
                levenshtein::prefix_edit_distance(query, measure.string.as_str())
                    <= measure.prefix_distance
            );
        }

        // this requires increasing the requested number for the fst implementation,
        // because there are 10 strings that start with "aberr", which also have PED of 1
        //assert!(contains_string(&result, "overrank"));
    }

    #[test]
    /// Tests for error-tolerant autocompletion against a large database
    fn words_long_query<A>()
    where
        A: Autocompleter + FromStrings,
    {
        let source: Vec<_> = WORDS.lines().collect();
        let autocompleter = A::from_strings(&source);
        let mut state: A::STATE = Default::default();
        let requested = 3;
        println!("begin");
        let result =
            autocompleter.autocomplete("asfdasdvSDVASDFEWWEFWDASDAS", requested, &mut state);
        assert_eq!(result.len(), requested);

        for measure in &result {
            println!("{:#?}", measure);
        }
    }

    #[test]
    /// Tests that any result has a PED under the given threshold
    fn words_threshold_topk<A>()
    where
        A: Autocompleter + FromStrings,
    {
        let source: Vec<_> = WORDS.lines().collect();
        let autocompleter = A::from_strings(&source);
        let mut state: A::STATE = Default::default();
        let requested = 3;
        println!("begin");
        let query = "asfdasdvSDVASDFEWWEFWDASDAS";
        let result = autocompleter.threshold_topk(query, requested, 5, &mut state);
        dbg!(&result);
        assert_eq!(
            result.len(),
            0,
            "PEDs of results above 20 but threshold is 10"
        );
    }

    #[test]
    /// Tests for error-tolerant autocompletion against a large database
    fn words_long_query_exist<A>()
    where
        A: Autocompleter + FromStrings,
    {
        let source: Vec<_> = WORDS.lines().collect();
        let autocompleter = A::from_strings(&source);
        let mut state: A::STATE = Default::default();
        let requested = 3;
        println!("begin"); // nonsyntactically
        let result = autocompleter.autocomplete("nonsyntacticallz", requested, &mut state);
        assert_eq!(result.len(), requested);

        for measure in &result {
            println!("{:#?}", measure);
        }
    }

    #[test]
    /// Tests that the result from an empty query still has strings
    fn empty_query_test<A>()
    where
        A: Autocompleter + FromStrings,
    {
        let source: Vec<_> = WORDS.lines().collect();
        println!("words {}", source.len());
        let autocompleter = A::from_strings(&source);
        let mut state: A::STATE = Default::default();
        let result = autocompleter.autocomplete("", 1, &mut state);
        assert_ne!(result.len(), 0);
    }

    #[test]
    /// Tests that prefix edit distances are within the number of edits made to strings from a database
    /// using 1000 random data points
    ///
    /// Simultaneously tests that the prefix edit distances are correct
    fn words_bounded_peds<A>()
    where
        A: Autocompleter + FromStrings,
    {
        let subscriber = FmtSubscriber::builder()
            .with_max_level(Level::INFO)
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");

        let source: Vec<_> = WORDS.lines().collect();
        let autocompleter = A::from_strings(&source);
        let mut state: A::STATE = Default::default();
        let mut rng = rand::thread_rng();
        let mut total_duration = Duration::new(0, 0);
        let mut fails = 0;
        const ITERATIONS: usize = 1e4 as usize;
        let ed_max = 2;
        let mut cases = Vec::new();

        for _i in 0..ITERATIONS {
            let (string, edited_string, edits) = sample_edited_string(&source, &mut rng, ed_max);

            let time = Instant::now();
            let result =
                &autocompleter.threshold_topk(edited_string.as_str(), 1, ed_max, &mut state);
            if result.len() == 0 {
                fails += 1;
                cases.push((
                    prefix_edit_distance(string, &edited_string),
                    string,
                    edited_string,
                    None,
                ));
                continue;
            }
            let r1 = &result[0];
            total_duration += time.elapsed();

            debug!("{:?}", r1);

            // Depending on what edits were made, the result may not necessarily be equal to `string` (e.g. 5 edits to a string with a length of 5)
            // so we do not check that

            let mut assertions_hold = true;
            assertions_hold &= r1.prefix_distance <= ed_max;
            assertions_hold &= prefix_edit_distance(edited_string.as_str(), r1.string.as_str())
                <= r1.prefix_distance;
            if !assertions_hold {
                fails += 1;
                cases.push((
                    prefix_edit_distance(string, &edited_string),
                    string,
                    edited_string,
                    Some(r1.to_owned()),
                ));
            }
        }
        info!(
            "Average time per query: {} ms. Failed {fails}/{ITERATIONS}. Max ED searched {ed_max}. Total time: {}s",
            total_duration.as_millis() as f64 / (ITERATIONS - fails) as f64,
            total_duration.as_secs()
        );
        dbg!(cases);
    }

    // a former bug
    #[test]
    fn bug_prefix_drop<A>()
    where
        A: Autocompleter + FromStrings,
    {
        let source: Vec<_> = WORDS.lines().collect();
        let autocompleter = A::from_strings(&source);
        let mut state: A::STATE = Default::default();
        let result = &autocompleter.threshold_topk("elaxation curve", 1, 2, &mut state);
        dbg!(result);
        let result = &autocompleter.threshold_topk("elaxation curve", 1, 3, &mut state);
        dbg!(result);
        let result = &autocompleter.threshold_topk("elaxation curve", 1, 4, &mut state);
        dbg!(result);
    }

    #[test]
    fn bug2<A>()
    where
        A: Autocompleter + FromStrings,
    {
        let subscriber = FmtSubscriber::builder()
            .with_max_level(Level::TRACE)
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");

        let source: Vec<_> = WORDS.lines().collect();
        let autocompleter = A::from_strings(&source);
        let mut state: A::STATE = Default::default();
        let q = "物不见人";
        let result = &autocompleter.threshold_topk("物不见人", 10, 2, &mut state);
        let pd = prefix_edit_distance(q, &result[0].string);
        dbg!(result, pd);
    }

    #[instantiate_tests(<YokedMetaAutocompleter>)]
    mod meta {}
    #[instantiate_tests(<FstAutocompleter<Vec<u8>>>)]
    mod fst {}
}

// ideally this would use the #[bench] attribute but it's unstable
#[ignore]
#[test]
/// Benchmark the unindexed autocomplete against the request sampling used in the words_bounded_peds test
fn bench_unindexed() {
    let source: Vec<_> = WORDS.lines().collect();
    let cows: Vec<_> = source.iter().map(|&s| s.into()).collect();
    let mut rng = rand::thread_rng();
    let mut total_duration = Duration::new(0, 0);
    const ITERATIONS: usize = 1e3 as usize;
    for _i in 0..ITERATIONS {
        let (_, edited_string, _) = sample_edited_string(&source, &mut rng, 5);
        let time = Instant::now();
        let result = &unindexed_autocomplete(edited_string.as_str(), 1, &cows)[0];
        total_duration += time.elapsed();
        dbg!(result);
    }
    println!(
        "Average time per query: {} ms",
        total_duration.as_millis() as f64 / ITERATIONS as f64
    );
}

#[ignore]
#[test]
/// Check the performance of the autocomplete methods against the noise dataset
fn bench_noise() {
    const TEXT: &str = include_str!("noise.txt");
    let mut source: Vec<&str> = TEXT.lines().collect();
    let cows: Vec<_> = source.iter().map(|&s| s.into()).collect();
    let mut start = Instant::now();
    let mut state = ();
    source.sort();

    println!("Testing without a max threshold");

    let fst_autocomp = FstAutocompleter::from_strings(&source);
    println!("FST indexing took {} ms", start.elapsed().as_millis());

    let requested = 10;
    let query = &"z".repeat(35);

    start = Instant::now();
    let mut result = fst_autocomp.autocomplete(query, requested, &mut state);
    println!("Autocomplete took {} ms", start.elapsed().as_millis());

    for measure in result {
        println!("{}", measure);
    }

    start = Instant::now();
    let fst_data = fs::read("src/tests/noise.fst").expect("noise.fst should exist");
    let fst_autocomp = FstAutocompleter::new(Set::new(fst_data).unwrap().into_fst());
    println!(
        "Construction from file took {} ms",
        start.elapsed().as_millis()
    );

    start = Instant::now();
    result = unindexed_autocomplete(query, requested, &cows);
    println!(
        "Unindexed autocomplete took {} ms",
        start.elapsed().as_millis()
    );

    for measure in result {
        println!("{}", measure);
    }

    // my implementation of META is too slow for the zzz query

    start = Instant::now();
    let meta_autocomp = MetaAutocompleter::new(cows.len(), cows);
    println!("META indexing took {} ms", start.elapsed().as_millis());

    println!("\nTesting with a maximum threshold");

    const ITERATIONS: usize = 1e2 as usize;
    // test random queries with a PED of at most `MAX_THRESHOLD`
    const MAX_THRESHOLD: usize = 4;
    let mut rng = thread_rng();
    let ed = 5;
    for _i in 0..ITERATIONS {
        let edited_string = sample_edited_string(&source, &mut rng, ed).1;
        let query = edited_string.as_str();

        start = Instant::now();
        let result = fst_autocomp.threshold_topk(query, requested, MAX_THRESHOLD, &mut state);
        println!("Fst autocomplete took {} ms", start.elapsed().as_millis());
        dbg!(result);

        start = Instant::now();
        // let result = meta_autocomp.threshold_topk(query, requested, MAX_THRESHOLD);
        // println!("META autocomplete took {} ms", start.elapsed().as_millis());
        // dbg!(result);
    }
}
