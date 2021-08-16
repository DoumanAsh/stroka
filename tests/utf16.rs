#[test]
pub fn should_convert_from_utf16() {
    const TEXT: &str = "ろり text";
    let buf = TEXT.encode_utf16().collect::<Vec<_>>();
    let res = stroka::String::from_utf16(&buf).expect("To parse utf-16");
    assert_eq!(TEXT, res);

    let res = stroka::String::from_utf16_lossy(&buf);
    assert_eq!(TEXT, res);
}

#[test]
pub fn should_fail_from_invalid_utf16() {
    let buf = [0xD834u16, 0xDD1E, 0x006d, 0x0075, 0xD800, 0x0069, 0x0063];
    stroka::String::from_utf16(&buf).expect_err("Should fail to parse invalid utf-16");

    let buf = [0xD834u16, 0xDD1E, 0x006d, 0x0075, 0x0073, 0xDD1E, 0x0069, 0x0063, 0xD834];
    let res = stroka::String::from_utf16_lossy(&buf);
    assert_eq!(res, "𝄞mus\u{FFFD}ic\u{FFFD}");
}
