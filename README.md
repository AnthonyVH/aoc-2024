My solutions for Advent of Code 2024. Mainly used as a playground to learn Rust.
Some rules I set for myself:

- Any trips/tricks/solutions heard/read/discussed can't be used. I.e. I have to
  come up with everything myself. That goes for optimizations as well.
- All solutions (together) should run in less than 1 second.
- That 1 second rule turned out to be rather trivial, so the new goal is for
  each problem (individually) to run in less than 1 ms. (Work in progress.)

Last measured runtimes:

- Intel Core i5-12400
- 64 GB DDR4-PC3200
- rustc 1.85.0-nightly (4363f9b6f 2025-01-02) (profile: `bench`)

```
╭─────────────────┬───────────╮
│ Problem         ┆ Time [µs] │
╞═════════════════╪═══════════╡
│ Day 01 - Part A ┆        45 │
│ Day 01 - Part B ┆        62 │
│ Day 02 - Part A ┆       138 │
│ Day 02 - Part B ┆       361 │
│ Day 03 - Part A ┆       162 │
│ Day 03 - Part B ┆       226 │
│ Day 04 - Part A ┆       519 │
│ Day 04 - Part B ┆       110 │
│ Day 05 - Part A ┆       416 │
│ Day 05 - Part B ┆      2161 │
│ Day 06 - Part A ┆       295 │
│ Day 06 - Part B ┆       718 │
│ Day 07 - Part A ┆        99 │
│ Day 07 - Part B ┆       121 │
│ Day 08 - Part A ┆        22 │
│ Day 08 - Part B ┆        71 │
│ Day 09 - Part A ┆        65 │
│ Day 09 - Part B ┆       134 │
│ Day 10 - Part A ┆        77 │
│ Day 10 - Part B ┆        98 │
│ Day 11 - Part A ┆        33 │
│ Day 11 - Part B ┆      2659 │
│ Day 12 - Part A ┆       438 │
│ Day 12 - Part B ┆       827 │
│ Day 13 - Part A ┆        49 │
│ Day 13 - Part B ┆        53 │
│ Day 14 - Part A ┆        37 │
│ Day 14 - Part B ┆        30 │
│ Day 15 - Part A ┆       352 │
│ Day 15 - Part B ┆       733 │
│ Day 16 - Part A ┆      1467 │
│ Day 16 - Part B ┆      1483 │
│ Day 17 - Part A ┆         0 │
│ Day 17 - Part B ┆        39 │
│ Day 18 - Part A ┆        96 │
│ Day 18 - Part B ┆       142 │
│ Day 19 - Part A ┆       431 │
│ Day 19 - Part B ┆       740 │
│ Day 20 - Part A ┆       147 │
│ Day 20 - Part B ┆       315 │
│ Day 21 - Part A ┆         1 │
│ Day 21 - Part B ┆        31 │
│ Day 22 - Part A ┆        85 │
│ Day 22 - Part B ┆      1391 │
│ Day 23 - Part A ┆       888 │
│ Day 23 - Part B ┆       775 │
│ Day 24 - Part A ┆        46 │
│ Day 24 - Part B ┆        51 │
│ Day 25 - Part A ┆        62 │
├─────────────────┴───────────┤
│ Total                 19325 │
╰─────────────────────────────╯
```