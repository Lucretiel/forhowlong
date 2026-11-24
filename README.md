# forhowlong

[![CI](https://github.com/Lucretiel/forhowlong/actions/workflows/ci.yml/badge.svg)](https://github.com/Lucretiel/forhowlong/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/forhowlong.svg)](https://crates.io/crates/forhowlong)
[![docs.rs](https://img.shields.io/docsrs/forhowlong)](https://docs.rs/forhowlong)
[![MSRV](https://img.shields.io/crates/msrv/forhowlong)](https://crates.io/crates/forhowlong)

<!-- cargo-rdme start -->

`forhowlong` is an implementation of a friendly, human-readable duration format,
in the spirit of [humantime](https://docs.rs/humantime/) and
[`jiff::fmt::friendly`](https://docs.rs/jiff/latest/jiff/fmt/friendly/). It
is intended to be used in config files, command line arguments, and environment
variables for unambiguous representation of short durations: things like
connection timeouts and task intervals, where the complexity of civil time
doesn't really apply.

`forhowlong` believes that `--timeout=100` is bad, `--timeout-ms=100` is a bit
better, but `--timeout=100ms` is ideal: it's perfectly unambiguous and is much
more likely to survive transitions between tools that forward arguments to
each other.

## Example

```rust
use forhowlong::Duration;
use std::time;

let d1: Duration = "1s 500ms".parse().unwrap();
let d2: Duration = "1500ms".parse().unwrap();

assert_eq!(d1.as_std_duration(), time::Duration::from_millis(1500));
assert_eq!(d1, d2);
```

## Syntax

A `forhowlong` duration is a simple series of magnitude-unit pairs, separated
by optional spaces and an optional comma, and the total duration is the sum of
all the pairs. `100ms`, `1h20m`, `1s, 1.5ms`, and `1000000 nanoseconds` are all
valid timestamps.

It supports the following units. It is case sensitive and expects all-lowercase.

- Nanosecond: `ns`, `㎱`, `nanos`, `nanosec`, `nanosecs`, `nanosecond`,
  `nanoseconds`
- Microsecond (1,000 nanoseconds): `us`, `µs`, `μs`, `㎲`, `micros`, `microsec`,
  `microsecs`, `microsecond`, `microseconds`
- Millisecond (1,000 microseconds): `ms`, `㎳`, `millis`, `millisec`,
  `millisecs`, `millisecond`, `milliseconds`
- Seconds (1,000 milliseconds): `s`, `sec`, `secs`, `second`, `seconds`
- Minutes (60 seconds): `m`, `min`, `mins`, `minute`, `minutes`
- Hours (60 minutes): `h`, `hr`, `hrs`, `hour`, `hours`

We don't support any unit of time above hour, like "Day", because those units
have variable durations in practice (leap-days and DST and so on). Prefer to use
[jiff](http://docs.rs/jiff) if you need these kinds of time scales.

It supports fractional units, like `0.5s`, but will not accept any durations
with sub-nanosecond precision.

<!-- cargo-rdme end -->
