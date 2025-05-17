# State of art prefix fuzzy autocompletion

See [math](./topk_search.typ) in Typst.

The time complexity is dependent on the active matching set, and the size of tree overall, which doesn't explode as you type in longer strings.

`PED <= 3` is the common and reasonabnle parameter for fuzzy autocompletion.

At this parameter, META outperforms the common FST autocompletion algorithm used in non-commercial softwares, by 14x. (avg. 5ms for META, 70ms for FST).

There are only *exact prefix search* and *fuzzy search* available non-commercially, which for SOTA, are released by https://github.com/wolfgarbe 

This algorithm deals with the *prefix fuzzy search* problem, which competes with the commercial SOTA.

## Benchmark with Criterion

![](././docs/meta_varied_ed/report/violin.svg)

PED of 3 is enough for most autocompletion use cases, which as the graphs shows, usually completes within 20ms.

For comparison, the best former opensource implemenation is based on Levenshtein Finite State Automata. 

![](./docs/fst_varied_ed/report/violin.svg)

Indexing is both linear to META autocompletion and FST

![](./docs/compare_index_building/report/violin.svg)

Indexing with 1.4M unique words that are repeated to expand to up to 2M, takes 600ms

![](./docs/compare_index_building/meta/report/regression.svg)

## Benchmark reports

- https://ple1n.github.io/strprox/fst_varied_ed/ped_bounded/4/report/index.html
- https://ple1n.github.io/strprox/fst_varied_ed/ped_bounded/3/report/index.html
- https://ple1n.github.io/strprox/fst_varied_ed/ped_bounded/2/report/index.html
- https://ple1n.github.io/strprox/fst_varied_ed/ped_bounded/1/report/index.html
- https://ple1n.github.io/strprox/fst_varied_ed/ped_bounded/report/index.html
- https://ple1n.github.io/strprox/fst_varied_ed/report/index.html
- https://ple1n.github.io/strprox/compare_index_building/meta/report/index.html
- https://ple1n.github.io/strprox/compare_index_building/fst/report/index.html
- https://ple1n.github.io/strprox/compare_index_building/report/index.html
- https://ple1n.github.io/strprox/meta_varied_ed/ped_bounded/4/report/index.html
- https://ple1n.github.io/strprox/meta_varied_ed/ped_bounded/3/report/index.html
- https://ple1n.github.io/strprox/meta_varied_ed/ped_bounded/2/report/index.html
- https://ple1n.github.io/strprox/meta_varied_ed/ped_bounded/1/report/index.html
- https://ple1n.github.io/strprox/meta_varied_ed/ped_bounded/report/index.html
- https://ple1n.github.io/strprox/meta_varied_ed/report/index.html

## Testing 

Tracing flags from `INFO` to `TRACING` can enable code paths for self verification during algorithm execution.

Example runs

```
warning: `strprox` (lib test) generated 21 warnings (run `cargo fix --lib -p strprox --tests` to apply 5 suggestions)
    Finished `test` profile [optimized + debuginfo] target(s) in 7.12s
     Running unittests src/lib.rs (target/debug/deps/strprox-4173846f43ac9c14)

running 1 test
 INFO vapour return >1> Zvapour retur, vapour return, 1
 INFO 轘 >0> 轘, 轘, 0
 INFO concrete-lined canal >1> concrete-lzned canal, concrete-lined canal, 1
 INFO track-type tractor >0> track-type tractor, track-type tractor, 0
 INFO Arrhenius theory of dissociation >1> Arrhenius theory oaf dissociation, Arrhenius theory of dissociation, 1
 INFO ovenbird >1> ovnnbird, ovenbird, 1
 INFO 他侬 >1> 他r, 他, 1
 INFO 秦梁 >0> 秦梁, 秦梁, 0
 INFO hymenopter >0> hymenopter, hymenopter, 0
 INFO be offended at >1> be offendedt, be offended at, 1
 INFO sweeping machine >0> sweeping machine, sweeping machine, 0
 INFO centum >0> centum, centum, 0
 INFO weight parameter >1> weight puarameter, weight parameter, 1
 INFO record access mode >0> record access mode, record access mode, 0
 INFO 金屋娇娘 >1> 屋娇娘, 唤娇娘, 1
 INFO local variable >2> local v9rilable, local variable, 2
 INFO sojourner >0> sojourner, sojourner, 0
 INFO abrasive jet cleaning >2> abrasiv jet 8leaning, abrasive jet cleaning, 2
 INFO 仁皇 >0> 仁皇, 仁皇, 0
 INFO cab brace >1> cb brace, cab brace, 1
 INFO beater-refiner >1> bveater-refiner, beater-refiner, 1
 INFO 贯朽粟陈 >1> 朽粟陈, 朽陈, 1
 INFO 区闻陬见 >1> S闻陬见, 区闻陬见, 1
 INFO inexorable law >2> ineorableU law, inexorable law, 2
 INFO mark contact >2> mark conOct, mark contact, 2
 INFO acoasm >1> acosm, acosmia, 0
 INFO 宪批 >1> 批, 批, 0
 INFO zero-deviation situation >1> zero-dTeviation situation, zero-deviation situation, 1
 INFO index card >1> indSex card, index card, 1
 INFO 曙鼓 >0> 曙鼓, 曙鼓, 0
 INFO hybrid micro circuit >0> hybrid micro circuit, hybrid micro circuit, 0
 INFO 明确 >0> 明确, 明确, 0
 INFO hyperposia >2> hypekosia, hyperoside, 2
 INFO left residue class >2> left residub cass, left residue class, 2
 INFO graduated rate schedule >0> graduated rate schedule, graduated rate schedule, 0
 INFO Canales palatini minores >0> Canales palatini minores, Canales palatini minores, 0
 INFO Jack Robinson >0> Jack Robinson, Jack Robinson, 0
 INFO scamper off >0> scamper off, scamper off, 0
 INFO chondrogenetic >2> choRndrgenetic, chondrogenetic, 2
 INFO single instruction stream single data stream >2> sinfle instruction stram single data stream, single instruction stream single data stream, 2
 INFO 儒宗 >1> h宗, h, 1
 INFO addressing >2> admdrssing, addressing, 2
 INFO antijump baffle >0> antijump baffle, antijump baffle, 0
 INFO turning shovel >0> turning shovel, turning shovel, 0
 INFO subgrade squeeze-out >0> subgrade squeeze-out, subgrade squeeze-out, 0
 INFO quantum-mechanical memory machine >2> quanStuNm-mechanical memory machine, quantum-mechanical memory machine, 2
 INFO subsidiary rod >2> subsMdTary rod, subsidiary rod, 2
 INFO semi-liquid phase process >0> semi-liquid phase process, semi-liquid phase process, 0
 INFO noise trap >0> noise trap, noise trap, 0
 INFO 殿阙 >2> 殿Nc, Ncctuidae, 1
 INFO shock absorber function >1> shock absorber4function, shock absorber function, 1
 INFO coconut capacitor >0> coconut capacitor, coconut capacitor, 0
 INFO servo-control mechanism >0> servo-control mechanism, servo-control mechanism, 0
 INFO 蚕箪 >0> 蚕箪, 蚕箪, 0
 INFO versin >0> versin, versin, 0
 INFO pterygopalatine fossa >0> pterygopalatine fossa, pterygopalatine fossa, 0
 INFO swamp with >2> s7warmp with, swamp with, 2
 INFO cold junction temperature >2> old junction emperature, cold junction temperature, 2
 INFO matured dough >1> matured tdough, matured dough, 1
 INFO electric spark >1> el9ctric spark, electric spark, 1
 INFO authority for the report >0> authority for the report, authority for the report, 0
 INFO 素辉 >0> , 'd, 0
 INFO nephrocapsectomy >0> nephrocapsectomy, nephrocapsectomy, 0
 INFO 累心 >1> 累g心, 累心, 1
 INFO 笃瘵 >1> 瘵, 瘵, 0
 INFO sporogenous yeasts >0> sporogenous yeasts, sporogenous yeasts, 0
 INFO optical dividing head >1> optical dividBng head, optical dividing head, 1
 INFO projection comparator >2> proIjection compaator, projection comparator, 2
 INFO secondary sun wheel >1> secondary sun whel, secondary sun wheel, 1
 INFO inutile >0> inutile, inutile, 0
 INFO 牛头阿傍 >1> 牛头Z, 牛头, 1
 INFO hemoplastic >0> hemoplastic, hemoplastic, 0
 INFO ooecial vesicle >0> ooecial vesicle, ooecial vesicle, 0
 INFO double-crank press >1> douWle-crank press, double-crank press, 1
 INFO straightway pump >2> strad1ghtway pump, straightway pump, 2
 INFO 月灵 >1> d灵, d, 1
 INFO 幸恩 >0> , 'd, 0
 INFO 朝聘 >1> 聘, 聘, 0
 INFO service channel >0> service channel, service channel, 0
 INFO muscle of mastication >0> muscle of mastication, muscle of mastication, 0
 INFO efficiency >0> efficiency, efficiency, 0
 INFO reflecting square >1> reflectin square, reflecting square, 1
 INFO 鼻头出火 >2> 鼻Om头出火, 鼻头出火, 2
 INFO 悦乐 >0> 悦乐, 悦乐, 0
 INFO shadow microdiffraction >0> shadow microdiffraction, shadow microdiffraction, 0
 INFO alternate month >2> alternae onth, alternate month, 2
 INFO wedge clip >0> wedge clip, wedge clip, 0
 INFO CATB >2> CZAUB, CAU, 2
 INFO 帆布潜鸭 >1> 帆布鸭, 帆布, 1
 INFO file organization >0> file organization, file organization, 0
 INFO naphtha gas reversion >0> naphtha gas reversion, naphtha gas reversion, 0
 INFO grillroom >2> rillrom, Billroth Ⅰ anastomosis, 2
 INFO -ridden >2> -rsddLen, -ridden, 2
 INFO spellable >2> spLeQllable, spellable, 2
 INFO heavy pumping >0> heavy pumping, heavy pumping, 0
 INFO soda glass >0> soda glass, soda glass, 0
 INFO paludament >1> paludaent, paludament, 1
 INFO assignable instrument >1> assigEnable instrument, assignable instrument, 1
 INFO external taper turning >2> external aper turniAng, external taper turning, 2
 INFO pretend to >0> pretend to, pretend to, 0
 WARN Total words 1442910
 WARN Average time per query: 2.41 ms. Failed 0/100. Max ED searched 2. Total time: 0s. PED: [49, 29, 22]. PED_Given [45, 32, 23]
test tests::generic::meta::words_bounded_peds ... ok

successes:

---- tests::generic::meta::words_bounded_peds stdout ----
[src/tests/mod.rs:355:9] cases = []



running 1 test
 WARN Total words 1442910
 WARN Average time per query: 16.2 ms. Failed 1/100. Max ED searched 3. Total time: 1s. PED: [39, 26, 25, 9]. PED_Given [33, 32, 24, 11]
test tests::generic::meta::words_bounded_peds ... ok

successes:

---- tests::generic::meta::words_bounded_peds stdout ----
[src/tests/mod.rs:355:9] cases = [
    (
        3,
        "sterilized water",
        "erilXzed water",
        None,
    ),
]

running 1 test
 WARN Total words 1442910
 WARN Average time per query: 75.67 ms. Failed 0/100. Max ED searched 4. Total time: 7s. PED: [25, 23, 22, 19, 11]. PED_Given [23, 24, 22, 19, 12]
test tests::generic::meta::words_bounded_peds ... ok

running 1 test
 WARN Total words 1442910
 WARN Average time per query: 249.84 ms. Failed 0/100. Max ED searched 5. Total time: 24s. PED: [20, 16, 26, 19, 11, 8]. PED_Given [16, 16, 23, 20, 17, 8]
test tests::generic::meta::words_bounded_peds ... ok

```

## Known errors

There are known bugs that result in *non-exhaustion* that some results exist but are not found. 

You can benchmark error rates by `fn words_bounded_peds` in the tests directory

```
 WARN Total words 1442910
 WARN Average time per query: 0.12648 ms. Failed 0/100000. Max ED searched 1. Total time: 12s. PED: [55647, 44353]. PED_Given [53522, 46478]
test tests::generic::meta::words_bounded_peds ... ok


 WARN Total words 1442910
 WARN Average time per query: 0.101086 ms. Failed 0/1000000. Max ED searched 1. Total time: 101s. PED: [553811, 446189]. PED_Given [532239, 467761]
test tests::generic::meta::words_bounded_peds ... ok
```

As above, with low PED parameter the algorithm rarely misses results. 

```
 WARN Total words 1442910
 WARN Average time per query: 1.23232 ms. Failed 8/100000. Max ED searched 2. Total time: 123s. PED: [40395, 36192, 23405]. PED_Given [37201, 37210, 25589]
test tests::generic::meta::words_bounded_peds ... ok

successes:

---- tests::generic::meta::words_bounded_peds stdout ----
[src/tests/mod.rs:356:9] cases = [
    (
        2,
        "overdrive housing adapter plate",
        "erdrive housing adapter plate",
        None,
    ),
    (
        2,
        "mantle rock",
        "ntle rock",
        None,
    ),
    (
        2,
        "taxpayer personally filling returns",
        "xpayer personally filling returns",
        None,
    ),
    (
        2,
        "frame keystone",
        "ame keystone",
        None,
    ),
    (
        2,
        "acid esters",
        "id esters",
        None,
    ),
    (
        2,
        "vanillic aldehyde",
        "nillic aldehyde",
        None,
    ),
    (
        2,
        "double insurance",
        "uble insurance",
        None,
    ),
    (
        2,
        "comprehensive system",
        "mprehensive system",
        None,
    ),
]


 WARN Total words 1442910
 WARN Average time per query: 13.0908 ms. Failed 3/10000. Max ED searched 3. Total time: 130s. PED: [3299, 3112, 2284, 1303]. PED_Given [2895, 3159, 2460, 1486]
test tests::generic::meta::words_bounded_peds ... ok

successes:

---- tests::generic::meta::words_bounded_peds stdout ----
[src/tests/mod.rs:356:9] cases = [
    (
        3,
        "widen differences in personal income",
        "den differencDes in personal income",
        None,
    ),
    (
        2,
        "cloudless air",
        "oudless air",
        Some(
            MeasuredPrefix {
                string: "endless abrasive belt",
                prefix_distance: 3,
            },
        ),
    ),
    (
        3,
        "muscular socket",
        "scular sncket",
        None,
    ),
]

```

When `PED <= 2` or `PED <=3` is set as the bound, errors happen at `1e-4` rate perhaps due to implementation error.

```
 WARN Total words 1442910
 WARN Average time per query: 66.3074 ms. Failed 7/10000. Max ED searched 4. Total time: 663s. PED: [2831, 2766, 2078, 1556, 765]. PED_Given [2340, 2743, 2242, 1777, 898]
test tests::generic::meta::words_bounded_peds ... ok

successes:

---- tests::generic::meta::words_bounded_peds stdout ----
[src/tests/mod.rs:356:9] cases = [
    (
        3,
        "soil heat conduction",
        "il hea9 conduction",
        Some(
            MeasuredPrefix {
                string: "heat conduction",
                prefix_distance: 4,
            },
        ),
    ),
    (
        3,
        "inverse-flame",
        "verse-Ilame",
        Some(
            MeasuredPrefix {
                string: "Persea americana Mill.",
                prefix_distance: 4,
            },
        ),
    ),
    (
        3,
        "angle globe valve",
        "gle globe vlve",
        Some(
            MeasuredPrefix {
                string: "hose globe valve",
                prefix_distance: 4,
            },
        ),
    ),
    (
        4,
        "data file converter",
        "ta fie con2verter",
        None,
    ),
    (
        4,
        "familial hypophosphatemia",
        "milial hypophsEphatemia",
        None,
    ),
    (
        4,
        "maximum-minimum principle",
        "ximumNA-minimum principle",
        None,
    ),
    (
        4,
        "physical goods",
        "ysicap goFods",
        None,
    ),
]

```

When `PED<=4`, the error rate is `1/1000`. In the log above it either returned suboptimal results or did not find the answer.

I do not want to invest more time in this algorithm because I don't like it despite the good-enough results.

## [Citations](#citations)
```bibtex
@article{10.14778/2977797.2977808,
author = {Deng, Dong and Li, Guoliang and Wen, He and Jagadish, H. V. and Feng, Jianhua},
title = {META: An Efficient Matching-Based Method for Error-Tolerant Autocompletion},
year = {2016},
issue_date = {June 2016},
publisher = {VLDB Endowment},
volume = {9},
number = {10},
issn = {2150-8097},
url = {https://doi.org/10.14778/2977797.2977808},
doi = {10.14778/2977797.2977808},
journal = {Proc. VLDB Endow.},
month = {jun},
pages = {828–839},
numpages = {12}
}
```
This paper is also available from the conference website [here](http://www.vldb.org/pvldb/vol9/p828-deng.pdf).

## [License](#license)
Dual-licensed under [MIT](LICENSE-MIT) and [Apache-2.0](LICENSE-APACHE). You may choose either license.

### [Contributions](#contrib)
Contributions to this project are likewise understood to be dual-licensed under MIT and Apache-2.0.

## Notes 

`cargo bench  --bench index -- --plotting-backend plotters` if you want to generate benchs with plotters.

Run `deno -A gen_links.ts` to update links in this file