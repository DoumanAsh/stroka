#[test]
pub fn should_not_reserve_within_sso_capacity() {
    const MAX_CAP: usize = core::mem::size_of::<usize>() * 2 - 2;

    let mut stroka = stroka::String::new();

    for idx in 0..=MAX_CAP {
        stroka.reserve(idx);
        assert!(!stroka.is_alloc());
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

#[test]
pub fn should_remove_from_sso_string() {
    const TEXT: &str = "1単語8";
    let mut stroka = stroka::String::new_str(TEXT);

    assert!(!stroka.is_alloc());
    assert_eq!(stroka, TEXT);

    stroka.remove(1);
    assert_eq!(stroka, "1語8");

    stroka.remove(4);
    assert_eq!(stroka, "1語");
    stroka.remove(0);
    assert_eq!(stroka, "語");
    stroka.remove(0);
    assert_eq!(stroka, "");
}

#[test]
#[should_panic]
pub fn should_panic_on_non_char_bound_remove_from_sso_string() {
    const TEXT: &str = "1単語8";
    let mut stroka = stroka::String::new_str(TEXT);
    stroka.remove(2);
}

#[test]
#[should_panic]
pub fn should_panic_on_remove_from_outside_of_sso_string() {
    const TEXT: &str = "1単語8";
    let mut stroka = stroka::String::new_str(TEXT);
    stroka.remove(usize::max_value());
}

#[test]
pub fn should_remove_from_heap_string() {
    const TEXT: &str = "123456789単語123456789";
    let mut stroka = stroka::String::new_str(TEXT);

    assert!(stroka.is_alloc());
    assert_eq!(stroka, TEXT);

    stroka.remove(1);
    assert_eq!(stroka, "13456789単語123456789");
    stroka.remove(8);
    assert_eq!(stroka, "13456789語123456789");
    stroka.remove(19);
    assert_eq!(stroka, "13456789語12345678");
    stroka.remove(0);
    assert_eq!(stroka, "3456789語12345678");
    stroka.remove(6);
    assert_eq!(stroka, "345678語12345678");
    stroka.remove(6);
    assert_eq!(stroka, "34567812345678");
}

#[test]
#[should_panic]
pub fn should_panic_on_non_char_bound_remove_from_heap_string() {
    const TEXT: &str = "123456789単語123456789";
    let mut stroka = stroka::String::new_str(TEXT);
    stroka.remove(10);
}

#[test]
#[should_panic]
pub fn should_panic_on_remove_from_outside_of_heap_string() {
    const TEXT: &str = "123456789単語123456789";
    let mut stroka = stroka::String::new_str(TEXT);
    stroka.remove(usize::max_value());
}

#[test]
pub fn should_insert_at_any_valid_position() {
    const TEXT: &str = "1単語8";

    let mut stroka = stroka::String::new();

    stroka.insert_str(0, TEXT);
    assert_eq!(stroka, TEXT);

    stroka.insert_str(1, TEXT);
    assert_eq!(stroka, "11単語8単語8");
    stroka.insert_str(stroka.len(), TEXT);
    assert_eq!(stroka, "11単語8単語81単語8");
    stroka.insert_str(stroka.len() - 1, TEXT);
    assert_eq!(stroka, "11単語8単語81単語1単語88");
    stroka.insert(2, '-');
    assert_eq!(stroka, "11-単語8単語81単語1単語88");
    stroka.insert(0, '-');
    assert_eq!(stroka, "-11-単語8単語81単語1単語88");
    stroka.insert(stroka.len(), '-');
    assert_eq!(stroka, "-11-単語8単語81単語1単語88-");
    stroka.insert(stroka.len() - 2, '+');
    assert_eq!(stroka, "-11-単語8単語81単語1単語8+8-");
}

#[test]
pub fn should_retain_within_sso() {
    const TEXT: &str = "1--単語8-";
    let mut stroka = stroka::String::new_sso(TEXT);
    assert!(!stroka.is_alloc());

    stroka.retain(|ch| ch.len_utf8() == 1);
    assert_eq!(stroka, "1--8-");
    stroka.retain(|ch| ch != '-');
    assert_eq!(stroka, "18");
}

#[test]
pub fn should_retain_within_heap() {
    const TEXT: &str = "-1++1-単語8単語81単語1単語8+8-++";
    let mut stroka = stroka::String::new_str(TEXT);
    assert!(stroka.is_alloc());

    assert_eq!(stroka.len(), TEXT.len());
    stroka.retain(|ch| ch.len_utf8() == 1);
    assert_eq!(stroka, "-1++1-88118+8-++");
    stroka.retain(|ch| ch != '+');
    assert_eq!(stroka, "-11-881188-");
}


#[test]
#[should_panic]
pub fn should_panic_on_insert_outside_of_bound() {
    const TEXT: &str = "123456789単語123456789";
    let mut stroka = stroka::String::new_str(TEXT);
    stroka.insert_str(usize::max_value(), TEXT);
}
