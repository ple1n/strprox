use std::{
    fs,
    io::Write,
    time::{Duration, Instant},
};

use fst::Set;
use plotly::{
    Layout, Plot, Scatter,
    common::{HoverInfo, Marker, Mode, Title},
    layout::Axis,
};
use rand::{Rng, distributions::Uniform, thread_rng};
use tracing::{Level, debug, info, warn};
use tracing_subscriber::FmtSubscriber;
use yoke::Yoke;

use metacomplete::{
    Autocompleter, MeasuredPrefix,
    levenshtein::{
        edit_distance, prefix_edit_distance, sample_edited_string, unindexed_autocomplete,
    },
    prefix::FromStrings,
    strprox::{FstAutocompleter, MetaAutocompleter},
};
type YokedMetaAutocompleter = Yoke<MetaAutocompleter<'static>, Vec<String>>;

pub struct Point {
    query_len: usize,
    time: Duration,
}

fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::WARN)
        .with_line_number(false)
        .without_time()
        .with_file(false)
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let pts = bench::<YokedMetaAutocompleter>(1e4 as usize, 1);
    let mut plot = Plot::new();
    let mut vx = vec![];
    let mut vy = vec![];
    for p in pts {
        vx.push(p.query_len);
        vy.push(p.time.as_micros());
    }
    let trace = Scatter::new(vx, vy)
        .mode(Mode::Markers)
        .marker(Marker::new().size(3))
        .name("PED<=1");
    plot.add_trace(trace);

    let l = Layout::new()
        .y_axis(Axis::new().title(Title::with_text("Time in micro seconds")))
        .show_legend(true)
        .x_axis(Axis::new().title("Query length"));
    plot.set_layout(l);

    plot.write_html("plot_len.html");
}

const WORDS: &str = include_str!("../../../src/tests/words.txt");

fn bench<A>(iters: usize, ed: usize) -> Vec<Point>
where
    A: Autocompleter + FromStrings,
{
    let source: Vec<_> = WORDS.lines().collect();
    let autocompleter = A::from_strings(&source);
    let mut state: A::STATE = Default::default();
    let mut rng = rand::thread_rng();

    let mut fails = 0;

    let mut ped_results = [0].repeat(ed + 1);
    let mut ped_given = [0].repeat(ed + 1);
    let mut cases = Vec::new();

    let mut total_duration = Duration::new(0, 0);
    let mut pts = vec![];

    for _i in 0..iters {
        let (string, query, edits) = sample_edited_string(&source, &mut rng, ed);
        let ped_g = prefix_edit_distance(&query, &string);
        let qlen = query.chars().count();
        ped_given[ped_g] += 1;

        let time = Instant::now();
        let result = &autocompleter.threshold_topk(query.as_str(), 1, ed, &mut state);
        let t = time.elapsed();
        total_duration += t;

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
        pts.push(Point {
            query_len: qlen,
            time: t,
        });
        ped_results[ped] += 1;
    }
    warn!(
        "Average time per query: {} ms. Failed {fails}/{iters}. Max ED searched {ed}. Total time: {}s. PED: {:?}. PED_Given {:?}",
        total_duration.as_millis() as f64 / (iters) as f64,
        total_duration.as_secs(),
        ped_results,
        ped_given
    );
    cases.truncate(2);
    dbg!(cases);

    pts
}
