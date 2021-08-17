use stroka::StrExt;

#[test]
fn should_make_uppercase() {
    let s = "Grüße, Jürgen ❤";

    assert_eq!("GRüßE, JüRGEN ❤", StrExt::to_ascii_uppercase(s));
}

#[test]
fn should_make_lowercase() {
    let s = "Grüße, Jürgen ❤";

    assert_eq!("grüße, jürgen ❤", StrExt::to_ascii_lowercase(s));
}

#[test]
fn should_repeat() {
    assert_eq!(StrExt::repeat("0123456789abcdef", 0), "");
    assert_eq!(StrExt::repeat("0123456789abcdef", 1), "0123456789abcdef");
    assert_eq!(StrExt::repeat("0123456789abcdef", 2), "0123456789abcdef0123456789abcdef");
    assert_eq!(StrExt::repeat("0123456789abcdef", 3), "0123456789abcdef0123456789abcdef0123456789abcdef");

    assert_eq!(StrExt::repeat("", 0), "");
    assert_eq!(StrExt::repeat("", 1), "");
    assert_eq!(StrExt::repeat("", 2), "");
}

#[test]
#[should_panic]
fn should_panic_on_repeat_overflow() {
    StrExt::repeat("0123456789abcdef", usize::MAX);
}
