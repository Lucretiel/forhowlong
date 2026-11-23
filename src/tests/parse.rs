use core::{assert_matches, str::FromStr as _};

use const_format::{concatc, str_repeat};

use crate::{
    Duration, ErrorKind, HOUR_U128, MICRO_U128, MILLI_U128, MINUTE_U128, NANO_U128, SECOND_U128,
    StdDuration,
};

macro_rules! d {
    (
        $(h $h:literal )?
        $(m $m:literal )?
        $(s $s:literal )?
        $(ms $ms:literal )?
        $(us $us:literal )?
        $(ns $ns:literal )?
    ) => {
        StdDuration::from_nanos_u128(
           $( $h * HOUR_U128+)?
           $( $m * MINUTE_U128 +)?
           $( $s * SECOND_U128 +)?
           $( $ms * MILLI_U128 +)?
           $( $us * MICRO_U128 +)?
           $( $ns * NANO_U128 +)?
           0
        )
    };
}

macro_rules! parse_success_tests {
    ($(
        $name:ident: $input:expr => $expected:expr;
    )*) => {$(
        #[test]
        fn $name() {
            const INPUT: &str = $input;

            let duration: Duration = INPUT.parse().unwrap();
            let duration = duration.as_std_duration();

            assert_eq!(duration, $expected);
        }
    )*};
}

macro_rules! parse_failure_tests {
    ($(
        $name:ident: $input:expr => $kind:ident @ $position:literal;
    )*) => {$(
        #[test]
        fn $name() {
            const INPUT: &str = $input;

            let error = Duration::from_str(INPUT).unwrap_err();

            assert_matches!(error.kind(), ErrorKind::$kind, "wrong error kind");
            assert_eq!(error.index(), $position, "wrong error position");
        }
    )*}
}

const TWENTY_ZEROES: &str = str_repeat!("0", 20);

parse_success_tests! {
    second: "1s" => d!(s 1);
    leading_zeroes: concatc!(TWENTY_ZEROES, "5s") => d!(s 5);
    decimal: "0.5s" => d!(ms 500);
}

parse_failure_tests! {
    reject_sub_nano_1: "0.5ns" => SubNanosecond @ 0;
    reject_sub_nano_2: "0.00000000000000005ms" => SubNanosecond @ 0;
}

/// One test for every individual supported unit word.
mod unit_words {
    use super::*;

    parse_success_tests! {
        // Nanoseconds
        ns: "3ns" => d!(ns 3);
        ns_square: "3㎱" => d!(ns 3);
        nanos: "3nanos" => d!(ns 3);
        nanosec: "3nanosec" => d!(ns 3);
        nanosecs: "3nanosecs" => d!(ns 3);
        nanosecond: "1nanosecond" => d!(ns 1);
        nanoseconds: "3nanoseconds" => d!(ns 3);

        // Microseconds
        us: "5us" => d!(us 5);
        us_micro_sign: "5\u{b5}s" => d!(us 5);
        us_greek_mu: "5\u{3bc}s" => d!(us 5);
        us_square: "5㎲" => d!(us 5);
        micros: "5micros" => d!(us 5);
        microsec: "5microsec" => d!(us 5);
        microsecs: "5microsecs" => d!(us 5);
        microsecond: "1microsecond" => d!(us 1);
        microseconds: "5microseconds" => d!(us 5);

        // Milliseconds
        ms: "7ms" => d!(ms 7);
        ms_square: "7㎳" => d!(ms 7);
        millis: "7millis" => d!(ms 7);
        millisec: "7millisec" => d!(ms 7);
        millisecs: "7millisecs" => d!(ms 7);
        millisecond: "1millisecond" => d!(ms 1);
        milliseconds: "7milliseconds" => d!(ms 7);

        // Seconds
        s: "11s" => d!(s 11);
        sec: "11sec" => d!(s 11);
        secs: "11secs" => d!(s 11);
        second_word: "1second" => d!(s 1);
        seconds: "11seconds" => d!(s 11);

        // Minutes
        m: "13m" => d!(m 13);
        min: "13min" => d!(m 13);
        mins: "13mins" => d!(m 13);
        minute: "1minute" => d!(m 1);
        minutes: "13minutes" => d!(m 13);

        // Hours
        h: "17h" => d!(h 17);
        hr: "17hr" => d!(h 17);
        hrs: "17hrs" => d!(h 17);
        hour: "1hour" => d!(h 1);
        hours: "17hours" => d!(h 17);
    }
}

/// The same unit words, with a space between the number and the unit, and
/// followed by another component. Makes sure each word stops where it should.
mod unit_words_spaced_and_followed {
    use super::*;

    parse_success_tests! {
        ns: "2 ns 1s" => d!(s 1 ns 2);
        ns_square: "2 ㎱1s" => d!(s 1 ns 2);
        nanos: "2 nanos, 1s" => d!(s 1 ns 2);
        nanosec: "2 nanosec 1s" => d!(s 1 ns 2);
        nanosecs: "2 nanosecs 1s" => d!(s 1 ns 2);
        nanosecond: "2 nanosecond 1s" => d!(s 1 ns 2);
        nanoseconds: "2 nanoseconds 1s" => d!(s 1 ns 2);

        us: "2 us 1s" => d!(s 1 us 2);
        us_micro_sign: "2 \u{b5}s1s" => d!(s 1 us 2);
        us_greek_mu: "2 \u{3bc}s1s" => d!(s 1 us 2);
        us_square: "2 ㎲1s" => d!(s 1 us 2);
        micros: "2 micros 1s" => d!(s 1 us 2);
        microsec: "2 microsec 1s" => d!(s 1 us 2);
        microsecs: "2 microsecs 1s" => d!(s 1 us 2);
        microsecond: "2 microsecond 1s" => d!(s 1 us 2);
        microseconds: "2 microseconds 1s" => d!(s 1 us 2);

        ms: "2 ms 1s" => d!(s 1 ms 2);
        ms_square: "2 ㎳1s" => d!(s 1 ms 2);
        millis: "2 millis 1s" => d!(s 1 ms 2);
        millisec: "2 millisec 1s" => d!(s 1 ms 2);
        millisecs: "2 millisecs 1s" => d!(s 1 ms 2);
        millisecond: "2 millisecond 1s" => d!(s 1 ms 2);
        milliseconds: "2 milliseconds 1s" => d!(s 1 ms 2);

        s: "2 s 1h" => d!(h 1 s 2);
        sec: "2 sec 1h" => d!(h 1 s 2);
        secs: "2 secs 1h" => d!(h 1 s 2);
        second_word: "2 second 1h" => d!(h 1 s 2);
        seconds: "2 seconds 1h" => d!(h 1 s 2);

        m: "2 m 1s" => d!(m 2 s 1);
        min: "2 min 1s" => d!(m 2 s 1);
        mins: "2 mins 1s" => d!(m 2 s 1);
        minute: "2 minute 1s" => d!(m 2 s 1);
        minutes: "2 minutes 1s" => d!(m 2 s 1);

        h: "2 h 1s" => d!(h 2 s 1);
        hr: "2 hr 1s" => d!(h 2 s 1);
        hrs: "2 hrs 1s" => d!(h 2 s 1);
        hour: "2 hour 1s" => d!(h 2 s 1);
        hours: "2 hours 1s" => d!(h 2 s 1);
    }
}

/// Things that look like units, but aren't.
mod unit_rejects {
    use super::*;

    parse_failure_tests! {
        uppercase_s: "1S" => ExpectedUnit @ 1;
        uppercase_ms: "1MS" => ExpectedUnit @ 1;
        mixed_case_ms: "1mS" => ExpectedUnit @ 1;
        uppercase_h: "1H" => ExpectedUnit @ 1;
        capital_minutes: "1Minutes" => ExpectedUnit @ 1;
        day: "1d" => ExpectedUnit @ 1;
        days: "1days" => ExpectedUnit @ 1;
        week: "1w" => ExpectedUnit @ 1;
        year: "1y" => ExpectedUnit @ 1;
        msec: "1msec" => ExpectedUnit @ 1;
        usec: "1usec" => ExpectedUnit @ 1;
        nsec: "1nsec" => ExpectedUnit @ 1;
        greek_mu_sec: "1\u{3bc}sec" => ExpectedUnit @ 1;
        bare_nano: "1nano" => ExpectedUnit @ 1;
        bare_micro: "1micro" => ExpectedUnit @ 1;
        bare_milli: "1milli" => ExpectedUnit @ 1;
        bare_micro_sign: "1\u{b5}" => ExpectedUnit @ 1;
        bare_greek_mu: "1\u{3bc}" => ExpectedUnit @ 1;
        bare_n: "1n" => ExpectedUnit @ 1;
        bare_u: "1u" => ExpectedUnit @ 1;
        nano_space_s: "1nano s" => ExpectedUnit @ 1;
        micro_hyphen_seconds: "1micro-seconds" => ExpectedUnit @ 1;
        millimeters: "1millimeters" => ExpectedUnit @ 1;
        mil: "1mil" => ExpectedUnit @ 1;
        mseconds: "1mseconds" => ExpectedUnit @ 1;
        double_plural: "1secss" => ExpectedUnit @ 1;
        minss: "1minss" => ExpectedUnit @ 1;
        hourss: "1hourss" => ExpectedUnit @ 1;
        hs: "1hs" => ExpectedUnit @ 1;
        trailing_letter: "1sx" => ExpectedUnit @ 1;
        doubled_square: "1㎳㎳" => ExpectedUnit @ 1;
        square_then_letter: "1㎳x" => ExpectedUnit @ 1;
        emoji: "1🕐" => ExpectedUnit @ 1;
        exponent: "1e3s" => ExpectedUnit @ 1;
        underscore: "1_000s" => ExpectedUnit @ 1;
        decimal_comma: "1,5s" => ExpectedUnit @ 1;
        thousands_space: "1 000s" => ExpectedUnit @ 2;
        tab_separator: "1\ts" => ExpectedUnit @ 1;
        nbsp_separator: "1\u{a0}s" => ExpectedUnit @ 1;
        ideographic_space_separator: "1\u{3000}s" => ExpectedUnit @ 1;
        newline_separator: "1\ns" => ExpectedUnit @ 1;
        fullwidth_digit_after_digit: "1\u{ff15}s" => ExpectedUnit @ 1;
        second_unit_bad: "1h 30x" => ExpectedUnit @ 5;
        unit_error_after_multibyte: "1\u{3bc}s 2q" => ExpectedUnit @ 6;
    }
}

/// Numbers with no unit, or no number at all.
mod missing_pieces {
    use super::*;

    parse_failure_tests! {
        empty: "" => ExpectedDigit @ 0;
        just_space: " " => ExpectedDigit @ 0;
        just_unit: "s" => ExpectedDigit @ 0;
        just_dot: "." => ExpectedDigit @ 0;
        no_unit: "1" => ExpectedUnit @ 1;
        no_unit_trailing_space: "1 " => ExpectedUnit @ 2;
        no_unit_trailing_spaces: "1   " => ExpectedUnit @ 4;
        fraction_no_unit: "1.5" => ExpectedUnit @ 3;
        fraction_no_unit_trailing_space: "1.5 " => ExpectedUnit @ 4;
        dangling_dot: "1." => ExpectedDigit @ 2;
        dot_then_unit: "1.s" => ExpectedDigit @ 2;
        leading_dot: ".5s" => ExpectedDigit @ 0;
        double_dot: "1..5s" => ExpectedDigit @ 2;
        double_fraction: "1.5.5s" => ExpectedUnit @ 3;
        space_before_dot: "1 .5s" => ExpectedUnit @ 2;
        space_after_dot: "1. 5s" => ExpectedDigit @ 2;
        negative: "-1s" => ExpectedDigit @ 0;
        positive_sign: "+1s" => ExpectedDigit @ 0;
        leading_space: " 1s" => ExpectedDigit @ 0;
        fullwidth_digit: "\u{ff15}s" => ExpectedDigit @ 0;
        arabic_indic_digit: "\u{661}s" => ExpectedDigit @ 0;
        superscript_digit: "\u{b2}s" => ExpectedDigit @ 0;
        fullwidth_fraction_digit: "1.\u{ff15}s" => ExpectedDigit @ 2;
        second_component_missing_unit: "1h30" => ExpectedUnit @ 4;
        second_component_missing_number: "1h ms" => ExpectedDigit @ 3;
        trailing_space: "1s " => ExpectedDigit @ 3;
        trailing_spaces: "1s   " => ExpectedDigit @ 5;
        trailing_comma: "1s," => ExpectedDigit @ 3;
        trailing_comma_space: "1s, " => ExpectedDigit @ 4;
        double_comma: "1s,,2s" => ExpectedDigit @ 3;
        double_comma_spaced: "1s, , 2s" => ExpectedDigit @ 4;
        leading_comma: ",1s" => ExpectedDigit @ 0;
        conjunction: "1 hour and 30 minutes" => ExpectedDigit @ 7;
        tab_between_components: "1s\t2s" => ExpectedDigit @ 2;
        newline_between_components: "1s\n2s" => ExpectedDigit @ 2;
        trailing_dot_after_unit: "1s." => ExpectedDigit @ 2;
    }
}

mod numbers {
    use super::*;

    parse_success_tests! {
        zero: "0s" => d!();
        zero_hours: "0h" => d!();
        double_zero: "00s" => d!();
        leading_zero: "007s" => d!(s 7);
        leading_zeroes_hours: concatc!(TWENTY_ZEROES, "1h") => d!(h 1);
        zero_point_zero: "0.0s" => d!();
        zero_point_zero_ns: "0.0ns" => d!();
        many_zero_fraction_digits: concatc!("0.", TWENTY_ZEROES, "s") => d!();
        big_seconds: "1000000s" => d!(s 1_000_000);
        big_nanos: "1000000000ns" => d!(s 1);
        thousand_ms: "1000ms" => d!(s 1);
        sixty_minutes: "60m" => d!(h 1);
        ninety_minutes: "90m" => d!(h 1 m 30);
        huge_nanos: "18446744073709551615ns" => StdDuration::from_nanos(u64::MAX);
        nanos_beyond_u64: "18446744073709551616ns" => d!(ns 18446744073709551616);
    }
}

mod fractions {
    use super::*;

    parse_success_tests! {
        half_second: "1.5s" => d!(s 1 ms 500);
        trailing_fraction_zero: "1.50s" => d!(s 1 ms 500);
        many_trailing_zeroes: concatc!("1.5", TWENTY_ZEROES, "s") => d!(s 1 ms 500);
        one_nano_in_seconds: "0.000000001s" => d!(ns 1);
        full_precision_seconds: "1.123456789s" => d!(s 1 ms 123 us 456 ns 789);
        one_nano_in_millis: "0.000001ms" => d!(ns 1);
        one_nano_in_micros: "0.001us" => d!(ns 1);
        micros_fraction: "1.5us" => d!(us 1 ns 500);
        micros_full_precision: "1.001us" => d!(us 1 ns 1);
        whole_nanos_fraction: "1.0ns" => d!(ns 1);
        whole_nanos_long_fraction: "1.000ns" => d!(ns 1);
        half_minute: "0.5m" => d!(s 30);
        tenth_minute: "0.1m" => d!(s 6);
        hundredth_minute: "0.01m" => d!(ms 600);
        tiny_minute: "0.00000000005m" => d!(ns 3);
        smallest_minute: "0.0000000001m" => d!(ns 6);
        quarter_hour: "0.25h" => d!(m 15);
        hour_and_half: "1.5h" => d!(h 1 m 30);
        tenth_hour: "0.1h" => d!(m 6);
        tiny_hour: "0.0000000000025h" => d!(ns 9);
        fraction_with_space: "1.5 s" => d!(s 1 ms 500);
        fraction_word_unit: "2.5 milliseconds" => d!(ms 2 us 500);
        fraction_then_more: "1.5h 30m" => d!(h 2);
        fraction_square_unit: "1.5㎳" => d!(ms 1 us 500);
    }

    parse_failure_tests! {
        half_nano: "0.5ns" => SubNanosecond @ 0;
        tenth_nano: "0.1ns" => SubNanosecond @ 0;
        nano_and_a_half: "2.5ns" => SubNanosecond @ 0;
        ten_digit_seconds: "1.1234567891s" => SubNanosecond @ 0;
        ten_digit_seconds_trailing_zeroes: "0.0000000001000s" => SubNanosecond @ 0;
        seven_digit_millis: "0.0000001ms" => SubNanosecond @ 0;
        four_digit_micros: "1.0001us" => SubNanosecond @ 0;
        too_small_minute: "0.00000000001m" => SubNanosecond @ 0;
        third_of_nano_hour: "0.000000000001h" => SubNanosecond @ 0;
        long_nonsense_fraction: "1.123456789012345678901234567890ns" => SubNanosecond @ 0;
        sub_nano_in_second_component: "1s 0.5ns" => SubNanosecond @ 3;
        sub_nano_after_comma: "1h, 2m, 0.5ns" => SubNanosecond @ 8;
    }
}

/// Multiple components, separators, and summing.
mod multi_component {
    use super::*;

    parse_success_tests! {
        hours_minutes: "1h20m" => d!(h 1 m 20);
        hours_minutes_spaced: "1h 20m" => d!(h 1 m 20);
        hours_minutes_comma: "1h,20m" => d!(h 1 m 20);
        hours_minutes_comma_space: "1h, 20m" => d!(h 1 m 20);
        space_before_comma: "1h ,20m" => d!(h 1 m 20);
        spaces_around_comma: "1h   ,   20m" => d!(h 1 m 20);
        many_spaces: "1h      20m" => d!(h 1 m 20);
        all_units: "1h2m3s4ms5us6ns" => d!(h 1 m 2 s 3 ms 4 us 5 ns 6);
        all_units_spaced: "1h 2m 3s 4ms 5us 6ns" => d!(h 1 m 2 s 3 ms 4 us 5 ns 6);
        all_units_commas: "1h, 2m, 3s, 4ms, 5us, 6ns" => d!(h 1 m 2 s 3 ms 4 us 5 ns 6);
        all_units_long: "1 hour, 2 minutes, 3 seconds, 4 milliseconds, 5 microseconds, 6 nanoseconds"
            => d!(h 1 m 2 s 3 ms 4 us 5 ns 6);
        all_units_squares: "3s4㎳5㎲6㎱" => d!(s 3 ms 4 us 5 ns 6);
        long_words_spaced: "1 hour 30 minutes" => d!(h 1 m 30);
        reverse_order: "6ns 5us 4ms 3s 2m 1h" => d!(h 1 m 2 s 3 ms 4 us 5 ns 6);
        repeated_unit: "1s 1s" => d!(s 2);
        repeated_unit_many: "1s1s1s1s1s" => d!(s 5);
        carry_seconds: "59s 1s" => d!(m 1);
        carry_millis: "999ms 1ms" => d!(s 1);
        carry_nanos: "999999999ns 1ns" => d!(s 1);
        mixed_fraction_components: "0.5s 0.5s" => d!(s 1);
        mixed_spellings: "1hr 2mins 3secs" => d!(h 1 m 2 s 3);
        ms_then_m: "1ms1m" => d!(m 1 ms 1);
        m_then_ms: "1m1ms" => d!(m 1 ms 1);
        min_then_s: "1min1s" => d!(m 1 s 1);
        mixed_mu_signs: "1\u{b5}s 1\u{3bc}s" => d!(us 2);
        zero_components: "0h 0m 0s" => d!();
    }
}

mod overflow {
    use super::*;

    /// The largest whole number of hours that fits in a `Duration`.
    const MAX_HOURS: u64 = u64::MAX / 3600;

    parse_success_tests! {
        max_seconds: concatc!(u64::MAX, "s") => StdDuration::new(u64::MAX, 0);
        max_duration: concatc!(u64::MAX, ".999999999s") => StdDuration::MAX;
        max_duration_in_parts: concatc!(u64::MAX, "s 999ms 999us 999ns") => StdDuration::MAX;
        max_hours: concatc!(MAX_HOURS, "h") => StdDuration::from_secs(MAX_HOURS * 3600);
        max_hours_plus_remainder: concatc!(MAX_HOURS, "h 15s") => StdDuration::new(u64::MAX, 0);
    }

    parse_failure_tests! {
        seconds_past_max: "18446744073709551616s" => Overflow @ 0;
        hours_past_max: concatc!(MAX_HOURS + 1, "h") => Overflow @ 0;
        beyond_u128: "999999999999999999999999999999999999999999h" => Overflow @ 0;
        max_plus_nano: concatc!(u64::MAX, ".999999999s 1ns") => Overflow @ 32;
        sum_overflow: concatc!(u64::MAX, "s 1s") => Overflow @ 22;
        sum_overflow_comma: concatc!("1s, ", u64::MAX, "s") => Overflow @ 4;
        hours_sum_overflow: concatc!(MAX_HOURS, "h 16s") => Overflow @ 18;
        overflow_beats_later_syntax_error: "18446744073709551616s nonsense" => Overflow @ 0;
    }
}
