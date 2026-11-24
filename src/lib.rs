/*!
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

# Example

```
use forhowlong::Duration;
use std::time;

let d1: Duration = "1s 500ms".parse().unwrap();
let d2: Duration = "1500ms".parse().unwrap();

assert_eq!(d1.as_std_duration(), time::Duration::from_millis(1500));
assert_eq!(d1, d2);
```

# Syntax

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
*/

#![no_std]
#![warn(missing_docs)]

mod consts;
mod parse;

#[cfg(test)]
mod tests;

use core::{
    fmt::{self, Display, Formatter},
    str::FromStr,
    time::Duration as StdDuration,
};

use consts::*;
use parse::{Error, parse_components};

pub use parse::ErrorKind;

/// An error that can occur when parsing a duration.
#[derive(Debug, Clone)]
pub struct ParseError {
    repr: Error<usize>,
}

impl ParseError {
    /// The type of error this is.
    #[inline]
    #[must_use]
    pub fn kind(&self) -> ErrorKind {
        self.repr.kind.clone()
    }

    /// The byte index in the string where this error occurred.
    #[inline]
    #[must_use]
    pub fn index(&self) -> usize {
        self.repr.location
    }
}

impl Display for ParseError {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.repr.kind, f)
    }
}

/**
A wrapper around a [`time::Duration`][core::time::Duration] with a friendly,
human-readable [`FromStr`] implementation. See [crate docs][crate] for details.
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Duration {
    repr: StdDuration,
}

impl Duration {
    /**
    Create a new [`forhowlong::Duration`][crate::Duration] wrapping a
    [`time::Duration`][core::time::Duration].
    */
    #[inline]
    #[must_use]
    pub const fn new(duration: StdDuration) -> Self {
        Self { repr: duration }
    }

    /**
    Get the underlying parsed [`time::Duration`][core::time::Duration].
     */
    #[inline]
    #[must_use]
    pub const fn as_std_duration(&self) -> StdDuration {
        self.repr
    }
}

impl From<StdDuration> for Duration {
    #[inline]
    fn from(value: StdDuration) -> Self {
        Self { repr: value }
    }
}

impl From<Duration> for StdDuration {
    #[inline]
    fn from(value: Duration) -> Self {
        value.repr
    }
}

impl AsRef<StdDuration> for Duration {
    #[inline]
    fn as_ref(&self) -> &StdDuration {
        &self.repr
    }
}

impl AsMut<StdDuration> for Duration {
    #[inline]
    fn as_mut(&mut self) -> &mut StdDuration {
        &mut self.repr
    }
}

macro_rules! format_units {
    ($fmt:expr, $nanos:expr;
        $($UNIT:ident $name:literal)*
    ) => {{
        let nanos = $nanos;
        let mut any = false;

        $(
            let nanos = {
                const UNIT: u128 = $UNIT as u128;

                let unit = nanos / UNIT;

                #[allow(clippy::modulo_one)]
                let remain = nanos % UNIT;

                if unit > 0 {
                    write!($fmt, "{space}{unit}{name}",
                        space = if any {" "} else {""},
                        name = $name,
                    )?;

                    any = true;
                }

                remain
            };
        )*

        let _ = nanos;

        if !any {
            write!($fmt, "0s")?
        }

        Ok(())
    }}
}

impl Display for Duration {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let nanos = self.repr.as_nanos();

        format_units!(
            f, nanos;

            HOUR_U128 "h"
            MINUTE_U128 "m"
            SECOND_U128 "s"
            MILLI_U128 "ms"
            MICRO_U128 "μs"
            NANO_U128 "ns"
        )
    }
}

impl FromStr for Duration {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_components(s)
            .map(|repr| Duration { repr })
            .map_err(|err| ParseError {
                repr: Error {
                    kind: err.kind,
                    location: s.len() - err.location.len(),
                },
            })
    }
}
