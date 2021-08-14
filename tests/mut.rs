#[test]
pub fn should_not_reserve_within_sso_capacity() {
    const MAX_CAP: usize = core::mem::size_of::<usize>() * 2 - 2;

    let mut stroka = stroka::String::new();

    for idx in 0..=MAX_CAP {
        stroka.reserve(idx);
        assert!(!stroka.is_heap());
    }
}

#[test]
pub fn should_push_various_chunks() {
    let chunks = [
        '1', '2', '3', '4', '5', '6', '7', '8', '9'
    ];

    let mut expected_string = String::new();
    let mut stroka: stroka::String = chunks.iter().collect();
    assert_eq!(stroka, "123456789");
    expected_string.push_str("123456789");

    for idx in 1..chunks.len() {
        let text: String = chunks[..idx].iter().collect();
        expected_string.push_str(&text);
        stroka.push_str(&text);
    }

    assert_eq!(stroka, expected_string);

    for ch in chunks {
        expected_string.push(ch);
        stroka.push(ch);
    }

    assert_eq!(stroka, expected_string);

    stroka.clear();
}
