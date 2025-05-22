use std::{
    fs,
    io::Write,
    time::{Duration, Instant},
};

use fst::Set;
use rand::{distributions::Uniform, thread_rng, Rng};
use tracing::{debug, info, warn, Level};
use tracing_subscriber::FmtSubscriber;
use yoke::Yoke;

use metacomplete::{
    levenshtein::{
        edit_distance, prefix_edit_distance, sample_edited_string, unindexed_autocomplete,
    },
    prefix::FromStrings,
    strprox::{FstAutocompleter, MetaAutocompleter},
    Autocompleter, MeasuredPrefix,
};

type YokedMetaAutocompleter = Yoke<MetaAutocompleter<'static>, Vec<String>>;

/// Returns whether any MeasuredPrefix in `measures` has the `expected` string
fn contains_string(measures: &Vec<MeasuredPrefix>, expected: &str) -> bool {
    measures.iter().any(|measure| measure.string == expected)
}

const WORDS: &str = include_str!("../src/tests/words.txt");

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;

fn words_bounded_peds<A>(c: &mut Criterion)
where
    A: Autocompleter + FromStrings,
{
    let source: Vec<_> = WORDS.lines().collect();
    let autocompleter = A::from_strings(&source);
    let mut state: A::STATE = Default::default();
    let mut rng = rand::thread_rng();

    let mut group = c.benchmark_group(A::NAME.to_owned() + "_varied_ed");

    let mut run = |iters, ed: usize| {
        let mut fails = 0;

        let mut ped_results = [0].repeat(ed + 1);
        let mut ped_given = [0].repeat(ed + 1);
        let mut cases = Vec::new();

        let mut total_duration = Duration::new(0, 0);

        for _i in 0..iters {
            let (string, query, edits) = sample_edited_string(&source, &mut rng, ed);
            let ped_g = prefix_edit_distance(&query, &string);
            ped_given[ped_g] += 1;

            let time = Instant::now();
            let result = &autocompleter.threshold_topk(query.as_str(), 1, ed, &mut state);
            total_duration += time.elapsed();

            if result.len() == 0 {
                fails += 1;
                cases.push((prefix_edit_distance(string, &query), string, query, None));
                continue;
            }
            let r1 = &result[0];

            info!(
                "{} >{}> {}, {}, {}",
                string, ped_g, query, r1.string, r1.prefix_distance
            );

            // Depending on what edits were made, the result may not necessarily be equal to `string` (e.g. 5 edits to a string with a length of 5)
            // so we do not check that
            let ped = prefix_edit_distance(query.as_str(), r1.string.as_str());
            let mut assertions_hold = true;
            assertions_hold &= r1.prefix_distance <= ed;
            assertions_hold &= ped <= r1.prefix_distance;
            assertions_hold &= r1.prefix_distance <= edits;
            if !assertions_hold {
                fails += 1;
                cases.push((
                    edit_distance(string, &query),
                    string,
                    query,
                    Some(r1.to_owned()),
                ));
            }
            ped_results[ped] += 1;
        }
        warn!(
        "Average time per query: {} ms. Failed {fails}/{iters}. Max ED searched {ed}. Total time: {}s. PED: {:?}. PED_Given {:?}",
        total_duration.as_millis() as f64 / (iters) as f64,
        total_duration.as_secs(),
        ped_results,
        ped_given
        );
        dbg!(cases);
        total_duration
    };

    for ed in [1, 2, 3, 4] {
        group.bench_with_input(BenchmarkId::new("ped_bounded", ed), &ed, |b, ed| {
            b.iter_custom(|iters| run(iters, *ed));
        });
    }

    warn!("Total words {}", source.len());
}

fn criterion_benchmark(c: &mut Criterion) {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::WARN)
        .with_line_number(false)
        .without_time()
        .with_file(false)
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");


    words_bounded_peds::<YokedMetaAutocompleter>(c);
    words_bounded_peds::<FstAutocompleter<Vec<u8>>>(c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
