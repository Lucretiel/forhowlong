// Currently can't be no_std, because nom_supreme doesn't have a no_std version
// yet.

use core::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use ascii_digit::{AsciiDigit, StrExt};
use nom::{
    IResult, Parser, bytes::complete::take_while1, character::complete::char, combinator::eof,
};
use nom_supreme::{
    ParserExt as _,
    final_parser::{ExtractContext, final_parser},
    multi::parse_separated_terminated_res,
};
use thiserror::Error;

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
        const NANO: u64 = 1;
        const MICRO: u64 = NANO * 1000;
        const MILLI: u64 = MICRO * 1000;
        const SECOND: u64 = MILLI * 1000;
        const MINUTE: u64 = SECOND * 60;
        const HOUR: u64 = MINUTE * 60;

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

#[non_exhaustive]
#[derive(Debug, Clone)]
enum ErrorKind {
    ExpectedUnit,
    ExpectedSpace,
    ExpectedDigit,
    Overflow,
    SubNanosecond,
    Other,
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            Self::ExpectedUnit => write!(f, "expected unit (ms, s, etc)"),
            Self::ExpectedSpace => write!(f, "expected space separating components"),
            Self::ExpectedDigit => write!(f, "expected a digit"),
            Self::Overflow => write!(f, "component overflowed integer"),
            Self::SubNanosecond => write!(f, "component had sub-nanosecond precision"),
            Self::Other => write!(f, "parse error"),
        }
    }
}

#[derive(Debug, Clone)]
struct ParseError<I> {
    kind: ErrorKind,
    location: I,
}

impl<I> nom::error::ParseError<I> for ParseError<I> {
    fn from_error_kind(location: I, _kind: nom::error::ErrorKind) -> Self {
        Self {
            kind: ErrorKind::Other,
            location,
        }
    }

    fn append(_: I, _: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

impl<I> nom::error::FromExternalError<I, ErrorKind> for ParseError<I> {
    fn from_external_error(location: I, _: nom::error::ErrorKind, kind: ErrorKind) -> Self {
        Self { location, kind }
    }
}

macro_rules! seconds_pattern {
    ($($short:literal)* sec, $long:literal seconds $(, $weird:literal)?) => {
        | concat!($long, "seconds")
        | concat!($long, "second")
        | concat!($long, "secs")
        | concat!($long, "sec")
        | concat!($long, "s")

        $(| concat!($short, "secs") )*
        $(| concat!($short, "sec") )*
        $(| concat!($short, "s") )*

        $(| concat!($weird))?
    }
}

fn parse_unit(input: &str) -> IResult<&str, Unit, ParseError<&str>> {
    take_while1(|c: char| c.is_alphabetic() || matches!(c, '㎱' | '㎳' | '㎲'))
        .map_res(|unit: &str| match unit {
            seconds_pattern!('n' sec, "nano" seconds, '㎱') => Ok(Unit::Nanosecond),
            seconds_pattern!('u' 'µ' 'μ' sec, "micro" seconds, '㎲') => Ok(Unit::Microsecond),
            seconds_pattern!('m' sec, "milli" seconds, '㎳') => Ok(Unit::Millisecond),
            seconds_pattern!(sec, "" seconds) => Ok(Unit::Second),
            "minutes" | "minute" | "mins" | "min" | "m" => Ok(Unit::Minute),
            "hours" | "hour" | "hrs" | "hr" | "h" => Ok(Unit::Hour),
            _ => Err(()),
        })
        .parse(input)
        .map_err(|err| {
            err.map(|()| ParseError {
                kind: ErrorKind::ExpectedUnit,
                location: input,
            })
        })
}

/// Parse one or more space characters
fn parse_spaces(input: &str) -> IResult<&str, (), ParseError<&str>> {
    match input.trim_start_matches(' ') {
        tail if tail.len() == input.len() => Err(nom::Err::Error(ParseError {
            kind: ErrorKind::ExpectedSpace,
            location: input,
        })),
        tail => Ok((tail, ())),
    }
}

/// Parse one or more digits
fn parse_digits(input: &str) -> IResult<&str, &[AsciiDigit], ParseError<&str>> {
    match input.split_leading_ascii_digits() {
        (&[], _) => Err(nom::Err::Error(ParseError {
            kind: ErrorKind::ExpectedDigit,
            location: input,
        })),
        (digits, tail) => Ok((tail, digits)),
    }
}

struct Component<'a> {
    int: &'a [AsciiDigit],
    fraction: Option<&'a [AsciiDigit]>,
    unit: Unit,
}

impl Component<'_> {
    pub fn as_nanos(&self) -> Result<i128, ErrorKind> {
        let factor = self.unit.factor();

        let int_part: i128 = self
            .int
            .iter()
            .copied()
            .map(|digit| digit.value())
            .try_fold(0i128, |accum, digit| {
                accum.checked_mul(10)?.checked_add(digit.into())
            })
            .and_then(|accum| accum.checked_mul(factor.into()))
            .ok_or(ErrorKind::Overflow)?;

        let frac_part: u64 = self
            .fraction
            .unwrap_or(&[])
            .iter()
            .copied()
            .rev()
            .map(|digit| digit.value() as u64)
            // This multiply is safe because the 9 * our largest unit of
            // measure definitely fits in a u64
            .map(|digit| digit * factor)
            .try_fold(0u64, |accum, scaled_digit| {
                // This add is safe because this accumulator is always less
                // than 10x our largest unit, which easily fits in a i128
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
}

fn parse_component(input: &str) -> IResult<&str, Component<'_>, ParseError<&str>> {
    parse_digits
        .and(parse_digits.cut().preceded_by(char('.')).opt())
        .and(parse_spaces.opt().precedes(parse_unit))
        .map(|((int, fraction), unit)| Component {
            int,
            fraction,
            unit,
        })
        .parse(input)
}

fn parse_components(input: &str) -> IResult<&str, i128, ParseError<&str>> {
    parse_separated_terminated_res(
        parse_component,
        parse_spaces.opt(),
        eof,
        || 0i128,
        |accum, component| {
            accum
                .checked_add(component.as_nanos()?)
                .ok_or(ErrorKind::Overflow)
        },
    )
    .parse(input)
}

#[derive(Debug, Clone, Error)]
#[error("{msg} at index {index}", msg = self.repr.kind, index = self.repr.location)]
pub struct DurationParseError {
    repr: ParseError<usize>,
}

impl ExtractContext<&str, DurationParseError> for ParseError<&str> {
    fn extract_context(self, original_input: &str) -> DurationParseError {
        DurationParseError {
            repr: ParseError {
                kind: self.kind,
                location: original_input.len() - self.location.len(),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Duration(pub ::core::time::Duration);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WideDuration {
    nanos: i128,
}

impl FromStr for WideDuration {
    type Err = DurationParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        final_parser(parse_components)(s).map(|nanos| WideDuration { nanos })
    }
}
