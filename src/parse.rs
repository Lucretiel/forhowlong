/*!
Low-level nom-style parsers
*/

use core::{
    fmt::{self, Display, Formatter},
    time::Duration as StdDuration,
};

use ascii_digit::{AsciiDigit, StrExt as _};

use crate::consts::*;

/**
The types of errors that can occur when parsing a [`Duration`][crate::Duration].
 */
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum ErrorKind {
    /// We expected a unit here: got `100` instead of `100 ms`
    ExpectedUnit,

    /**
    We expected a space separator here. Currently unused, reserved for potential
    future syntax updates
    */
    ExpectedSpace,

    /**
    We expected some digits here. Got `ms` instead of `10ms`, or`10. seconds`
    instead of `10.0 seconds`.
    */
    ExpectedDigit,

    /**
    We overflowed the maximum amount of time representable by a
    [`time::Duration`][core::time::Duration]. This is pretty impressive, given
    that a `Duration` can hold a span of time roughly 50× the age of the
    universe
     */
    Overflow,

    /**
    You tried, for some reason, to represent an amount of time less than a
    single nanosecond.
    */
    SubNanosecond,
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            Self::ExpectedUnit => write!(f, "expected unit (ms, s, etc)"),
            Self::ExpectedSpace => write!(f, "expected space separating components"),
            Self::ExpectedDigit => write!(f, "expected a digit"),
            Self::Overflow => write!(f, "duration overflowed"),
            Self::SubNanosecond => write!(f, "duration had sub-nanosecond precision"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Error<I> {
    pub kind: ErrorKind,
    pub location: I,
}

type ParseResult<I, O> = Result<(O, I), Error<I>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Nanosecond,
    Microsecond,
    Millisecond,
    Second,
    Minute,
    Hour,
}

impl Unit {
    #[inline]
    #[must_use]
    pub const fn factor(self) -> u64 {
        match self {
            Unit::Nanosecond => NANO,
            Unit::Microsecond => MICRO,
            Unit::Millisecond => MILLI,
            Unit::Second => SECOND,
            Unit::Minute => MINUTE,
            Unit::Hour => HOUR,
        }
    }
}

macro_rules! seconds_pattern {
    ($($short:literal)* s, $long:literal seconds $(, $weird:literal)?) => {
        | concat!($long, "seconds")
        | concat!($long, "second")
        | concat!($long, "secs")
        | concat!($long, "sec")
        | concat!($long, "s")

        $(| concat!($short, "s") )*

        $(| concat!($weird))?
    }
}

/// Split the string into two chunks, such that the first chunk contains
/// the prefix where all characters match the predicate
#[inline]
#[must_use]
fn split_matching(input: &str, pred: impl Fn(char) -> bool) -> (&str, &str) {
    let i = input.find(|c| !pred(c)).unwrap_or(input.len());
    input.split_at(i)
}

fn parse_unit(input: &str) -> ParseResult<&str, Unit> {
    let (unit, tail) = split_matching(input, |c| {
        c.is_alphabetic() || matches!(c, '㎱' | '㎳' | '㎲')
    });

    let unit = match unit {
        seconds_pattern!('n' s, "nano" seconds, '㎱') => Unit::Nanosecond,
        seconds_pattern!('u' 'µ' 'μ' s, "micro" seconds, '㎲') => Unit::Microsecond,
        seconds_pattern!('m' s, "milli" seconds, '㎳') => Unit::Millisecond,
        seconds_pattern!(s, "" seconds) => Unit::Second,
        "minutes" | "minute" | "mins" | "min" | "m" => Unit::Minute,
        "hours" | "hour" | "hrs" | "hr" | "h" => Unit::Hour,
        _ => {
            return Err(Error {
                kind: ErrorKind::ExpectedUnit,
                location: input,
            });
        }
    };

    Ok((unit, tail))
}

/// Parse one or more digits
fn parse_digits(input: &str) -> ParseResult<&str, &[AsciiDigit]> {
    match input.split_leading_ascii_digits() {
        (&[], _) => Err(Error {
            kind: ErrorKind::ExpectedDigit,
            location: input,
        }),
        (digits, tail) => Ok((digits, tail)),
    }
}

struct Component<'a> {
    int: &'a [AsciiDigit],
    fraction: Option<&'a [AsciiDigit]>,
    unit: Unit,
}

impl Component<'_> {
    pub fn as_nanos(&self) -> Result<u128, ErrorKind> {
        let factor = self.unit.factor();

        let int_part: u128 = self
            .int
            .iter()
            .copied()
            .map(|digit| digit.value())
            .try_fold(0u128, |accum, digit| {
                accum.checked_mul(10)?.checked_add(digit.into())
            })
            .and_then(|accum| accum.checked_mul(factor.into()))
            .ok_or(ErrorKind::Overflow)?;

        let frac_part: u64 = self
            .fraction
            .unwrap_or(&[])
            .iter()
            .copied()
            .map(|digit| digit.value() as u64)
            // This multiply is safe because the 9 * our largest unit of
            // measure definitely fits in a u64
            .map(|digit| digit * factor)
            .rev()
            .try_fold(0u64, |accum, scaled_digit| {
                // This add is safe because this accumulator is always less
                // than 10x our largest unit, which easily fits in a u64ß
                let accum = accum + scaled_digit;
                match accum % 10 {
                    0 => Some(accum / 10),
                    _ => None,
                }
            })
            .ok_or(ErrorKind::SubNanosecond)?;

        int_part
            .checked_add(frac_part.into())
            .ok_or(ErrorKind::Overflow)
    }

    pub fn as_std_duration(&self) -> Result<StdDuration, ErrorKind> {
        nanos_to_secs_subsecs(self.as_nanos()?)
            .map(|(sec, subsec)| StdDuration::new(sec, subsec))
            .ok_or(ErrorKind::Overflow)
    }
}

#[must_use]
#[inline]
fn nanos_to_secs_subsecs(nanos: u128) -> Option<(u64, u32)> {
    let secs = nanos / SECOND_U128;
    let subsec = nanos % SECOND_U128;

    let secs = secs.try_into().ok();
    let subsec = subsec.try_into().ok();

    Option::zip(secs, subsec)
}

#[inline]
fn parse_fraction(input: &str) -> ParseResult<&str, Option<&[AsciiDigit]>> {
    match input.strip_prefix('.') {
        None => Ok((None, input)),
        Some(suffix) => parse_digits(suffix).map(|(digits, tail)| (Some(digits), tail)),
    }
}

#[inline]
fn parse_component(input: &str) -> ParseResult<&str, Component<'_>> {
    let (int, input) = parse_digits(input)?;
    let (fraction, input) = parse_fraction(input)?;
    let input = input.trim_start_matches(' ');
    let (unit, input) = parse_unit(input)?;

    let component = Component {
        int,
        fraction,
        unit,
    };

    Ok((component, input))
}

/// Spaces and an optional comma. Can't fail.
#[inline]
#[must_use]
fn take_inter_component_separator(input: &str) -> &str {
    let input = input.trim_start_matches(' ');
    input
        .strip_prefix(',')
        .map(|suffix| suffix.trim_start_matches(' '))
        .unwrap_or(input)
}

#[inline]
pub fn parse_components(mut input: &str) -> Result<StdDuration, Error<&str>> {
    let mut accum = StdDuration::ZERO;

    loop {
        input = {
            let (component, tail) = parse_component(input)?;

            accum = component
                .as_std_duration()
                .and_then(|duration| accum.checked_add(duration).ok_or(ErrorKind::Overflow))
                .map_err(|kind| Error {
                    kind,
                    location: input,
                })?;

            let input = tail;

            if input.is_empty() {
                break Ok(accum);
            }

            take_inter_component_separator(input)
        };
    }
}
