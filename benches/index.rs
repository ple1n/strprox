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

use strprox::{
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

fn build<A, B>(c: &mut Criterion)
where
    A: Autocompleter + FromStrings,
    B: Autocompleter + FromStrings,
{
    let source: Vec<_> = WORDS.lines().collect();

    let mut group = c.benchmark_group("compare_index_building");

    // largesr measurement time allows Criterion to index larger word sets
    group.measurement_time(Duration::from_secs(30));

    group.bench_function(A::NAME, |b| {
        b.iter_custom(|iters| {
            let mut total_duration = Duration::new(0, 0);
            let arr: Vec<&str> = source
                .iter()
                .cycle()
                .map(|x| *x)
                .take(iters as usize)
                .collect();
            let time = Instant::now();
            let autocompleter = A::from_strings(&arr);
            total_duration += time.elapsed();

            warn!("built index of {} words, with {:?}", iters, total_duration);
            total_duration
        });
    });

    group.bench_function(B::NAME, |b| {
        b.iter_custom(|iters| {
            let mut total_duration = Duration::new(0, 0);
            let arr: Vec<&str> = source
                .iter()
                .cycle()
                .map(|x| *x)
                .take(iters as usize)
                .collect();
            let time = Instant::now();
            let autocompleter = B::from_strings(&arr[..]);
            total_duration += time.elapsed();

            warn!("built index of {} words, with {:?}", iters, total_duration);
            total_duration
        });
    });

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

    build::<FstAutocompleter<Vec<u8>>, YokedMetaAutocompleter>(c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
