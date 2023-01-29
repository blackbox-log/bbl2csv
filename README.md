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

As of [5ca6f6c](https://github.com/blackbox-log/blackbox-log/commit/5ca6f6cd43011323bc0358182546c0a7071ad546):

```shell
$ exa -lbs size --no-time --no-permissions --no-user blackbox-log/tests/logs/gimbal-ghost/LOG00001.BFL
6.6Mi blackbox-log/tests/logs/gimbal-ghost/LOG00001.BFL

$ hyperfine -w 10 -L bin ./blackbox-log-5ca6f6c,blackbox_decode '{bin} blackbox-log/tests/logs/gimbal-ghost/LOG00001.BFL'
Benchmark #1: ./blackbox-log-5ca6f6c blackbox-log/tests/logs/gimbal-ghost/LOG00001.BFL
  Time (mean ± σ):     674.1 ms ±   9.2 ms    [User: 613.8 ms, System: 61.8 ms]
  Range (min … max):   656.3 ms … 687.9 ms    10 runs

Benchmark #2: blackbox_decode blackbox-log/tests/logs/gimbal-ghost/LOG00001.BFL
  Time (mean ± σ):      1.077 s ±  0.010 s    [User: 1.025 s, System: 0.046 s]
  Range (min … max):    1.064 s …  1.090 s    10 runs

Summary
  './blackbox-log-5ca6f6c blackbox-log/tests/logs/gimbal-ghost/LOG00001.BFL' ran
    1.60 ± 0.03 times faster than 'blackbox_decode blackbox-log/tests/logs/gimbal-ghost/LOG00001.BFL'
```

`…/gimbal-ghost/LOG00001.BFL` contains only one log. Files with multiple logs
will see even larger improvements since logs are decoded in parallel using
[`rayon`](https://lib.rs/crates/rayon).

## License

In accordance with the [GNU FAQ][gpl-ports]'s guidance that ports are
derivative works, all code is licensed under the GPLv3 to match the Betaflight
and INAV projects.

[bb-log]: https://github.com/blackbox-log/blackbox-log
[gpl-ports]: https://www.gnu.org/licenses/gpl-faq.html#TranslateCode
