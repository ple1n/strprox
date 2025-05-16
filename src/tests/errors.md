journal for bugs that ever appeared.

```
test tests::generic::meta::words_bounded_peds has been running for over 60 seconds
2025-05-16T07:43:54.492230Z  INFO strprox::tests::generic: Average time per query: 0.7006274012548025 ms. Failed 2/1000000. Max ED searched 2. Total time: 700s
test tests::generic::meta::words_bounded_peds ... ok

successes:

---- tests::generic::meta::words_bounded_peds stdout ----
[src/tests/mod.rs:339:9] cases = [
    (
        1,
        "见物不见人",
        "物不见人",
    ),
    (
        0,
        "年晚生",
        "年晚生福",
    ),
]

```

```
2025-05-16T11:50:10.311918Z  INFO strprox::tests::generic: Average time per query: 1.6588976693007902 ms. Failed 3/10000. Max ED searched 2. Total time: 16s
test tests::generic::meta::words_bounded_peds ... ok

successes:

---- tests::generic::meta::words_bounded_peds stdout ----
[src/tests/mod.rs:342:9] cases = [
    (
        1,
        "许学",
        "许学轺",
        Some(
            MeasuredPrefix {
                string: "一封轺传",
                prefix_distance: 2,
            },
        ),
    ),
    (
        1,
        "分位",
        "分位垼",
        Some(
            MeasuredPrefix {
                string: "垼",
                prefix_distance: 2,
            },
        ),
    ),
    (
        1,
        "古义",
        "古义鼣",
        Some(
            MeasuredPrefix {
                string: "鼣",
                prefix_distance: 2,
            },
        ),
    ),
]
```