# `bbl2csv`

[![CI](https://github.com/blackbox-log/bbl2csv/actions/workflows/ci.yaml/badge.svg)](https://github.com/blackbox-log/bbl2csv/actions/workflows/ci.yaml)
[![dependency status](https://deps.rs/repo/github/blackbox-log/bbl2csv/status.svg)](https://deps.rs/repo/github/blackbox-log/bbl2csv)
[![license](https://img.shields.io/github/license/blackbox-log/bbl2csv)](https://github.com/blackbox-log/bbl2csv/blob/main/COPYING)

This is a cli frontend for [`blackbox-log`][bb-log] inspired by the original
`blackbox_decode`.

## Feature comparison

|                          | `blackbox_decode` | `bbl2csv` |
|--------------------------|:-----------------:|:---------:|
| Log format v1            | ✔️ | ❌ |
| Recent Betaflight logs   | ❌ | ✔️ |
| Raw output               | ✔️ | ❌ |
| Write output to stdout   | ✔️ | ❌ |
| GPS data                 | merged, separate, or gpx | separate |
| Current meter simulation | ✔️ | ❌ |
| IMU simulation           | ✔️ | ❌ |
| Change output units      | ✔️ | ❌ |
| Filter output fields     | ❌ | ✔️ |
| Parallel log parsing     | ❌ | ✔️ |

## Benchmarks

As of [`2b28331`](https://github.com/blackbox-log/bbl2csv/commit/2b2833133bd99b40247f9d3b267b22e1e00d8cf8), with [this log](https://github.com/gimbal-ghost/gimbal-ghost/blob/49d774a9f18f1ac8055e636d4dfa95090c9b2cb8/test/LOG00001.BFL):

```shell
$ exa -lbs size --no-time --no-permissions --no-user LOG00001.BFL
6.6Mi LOG00001.BFL

$ hyperfine -w 10 -L bin ./bbl2csv,blackbox_decode '{bin} LOG00001.BFL'
Benchmark #1: ./bbl2csv LOG00001.BFL
  Time (mean ± σ):     598.2 ms ±  13.6 ms    [User: 542.6 ms, System: 46.6 ms]
  Range (min … max):   574.4 ms … 622.6 ms    10 runs

Benchmark #2: blackbox_decode LOG00001.BFL
  Time (mean ± σ):      1.072 s ±  0.013 s    [User: 1.019 s, System: 0.044 s]
  Range (min … max):    1.056 s …  1.098 s    10 runs

Summary
  './bbl2csv LOG00001.BFL' ran
    1.79 ± 0.05 times faster than 'blackbox_decode LOG00001.BFL'
```

`LOG00001.BFL` contains only one log. Files with multiple logs will see even
larger improvements since logs are decoded in parallel using
[`rayon`](https://lib.rs/crates/rayon).

[bb-log]: https://github.com/blackbox-log/blackbox-log
