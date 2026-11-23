macro_rules! consts {
    (
        $UNIT:ident $UNIT_U128:ident $value:expr;
    ) => {
        pub const $UNIT: u64 = $value;
        pub const $UNIT_U128: u128 = $UNIT as u128;
    };

    (
        $FIRST:ident $FIRST_U128:ident $value:expr;
        $NEXT:ident $NEXT_U128:ident $factor:literal;
        $($REST:ident $REST_U128:ident $rest:literal;)*
    ) => {
        consts! { $FIRST $FIRST_U128 $value; }
        consts! {
            $NEXT $NEXT_U128 ($FIRST * $factor);
            $($REST $REST_U128 $rest;)*
        }
    };
}

consts! {
    NANO NANO_U128 1;
    MICRO MICRO_U128 1000;
    MILLI MILLI_U128 1000;
    SECOND SECOND_U128 1000;
    MINUTE MINUTE_U128 60;
    HOUR HOUR_U128 60;
}
