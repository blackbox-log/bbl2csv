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

As of [`0177b5e`](https://github.com/blackbox-log/bbl2csv/commit/0177b5effc338284e09dd50fc5756fd1120f6613), with [this log](https://github.com/gimbal-ghost/gimbal-ghost/blob/49d774a9f18f1ac8055e636d4dfa95090c9b2cb8/test/LOG00001.BFL):

```shell
$ exa -lbs size --no-time --no-permissions --no-user LOG00001.BFL
6.6Mi LOG00001.BFL

$ hyperfine -w 10 -L bin ./bbl2csv,blackbox_decode '{bin} LOG00001.BFL'
Benchmark #1: ./bbl2csv LOG00001.BFL
  Time (mean ± σ):     613.4 ms ±  16.6 ms    [User: 547.8 ms, System: 50.5 ms]
  Range (min … max):   586.2 ms … 647.1 ms    10 runs

Benchmark #2: blackbox_decode LOG00001.BFL
  Time (mean ± σ):      1.080 s ±  0.007 s    [User: 1.028 s, System: 0.047 s]
  Range (min … max):    1.072 s …  1.092 s    10 runs

Summary
  './bbl2csv LOG00001.BFL' ran
    1.76 ± 0.05 times faster than 'blackbox_decode LOG00001.BFL'
```

`LOG00001.BFL` contains only one log. Files with multiple logs will see even
larger improvements since logs are decoded in parallel using
[`rayon`](https://lib.rs/crates/rayon).

## License

In accordance with the [GNU FAQ][gpl-ports]'s guidance that ports are
derivative works, all code is licensed under the GPLv3 to match the Betaflight
and INAV projects.

[bb-log]: https://github.com/blackbox-log/blackbox-log
[gpl-ports]: https://www.gnu.org/licenses/gpl-faq.html#TranslateCode
