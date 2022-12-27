use match_bytes::match_bytes;

#[test]
fn matches_u32() {
    let bytes = [0, 0, 0, 1, 0, 0, 0, 0];
    match_bytes!([length: u32 / be, rest @ ..] = &bytes);

    assert_eq!(length, 1);
    assert_eq!(*rest, [0, 0, 0, 0]);
}
