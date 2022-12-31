use match_bytes::match_bytes;

macro_rules! test_primative_types(
    { $([$pat:expr, $bytes:expr, $expected_value:expr]),+ } => {
        {
            $(
    let bytes = $bytes;
    match_bytes!([value: $pat] = &bytes);

    assert_eq!(value, $expected_value);
            )+
        }
     };
);

#[test]
fn matches_primative_types() {
    test_primative_types!(
        [u8 / be, [1], 1],
        [i8 / be, [1], 1],
        [u16 / be, [0, 1], 1],
        [i16 / be, [0, 1], 1],
        [u32 / be, [0, 0, 0, 1], 1],
        [u32 / be, [0, 0, 0, 1], 1],
        [i32 / be, [0, 0, 0, 1], 1],
        [u64 / be, [0, 0, 0, 0, 0, 0, 0, 1], 1],
        [isize / be, [0, 0, 0, 0, 0, 0, 0, 1], 1],
        [usize / be, [0, 0, 0, 0, 0, 0, 0, 1], 1],
        [
            i128 / be,
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            1
        ],
        [
            u128 / be,
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            1
        ],
        [f64 / be, [63, 240, 0, 0, 0, 0, 0, 0], 1.0],
        [f32 / be, [63, 128, 0, 0], 1.0]
    );
}

#[test]
fn little_endian_conversion() {
    test_primative_types!([u32 / le, [1, 0, 0, 0], 1])
}

#[test]
fn matches_rest() {
    let bytes = [0, 0, 0, 1, 0, 0, 0, 0];
    match_bytes!([_prefix: u32 / be, rest @ ..] = &bytes);

    assert_eq!(*rest, [0, 0, 0, 0]);
}
