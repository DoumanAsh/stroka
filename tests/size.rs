use core::mem;

#[test]
pub fn should_have_size_of_2_words() {
    let stroka = stroka::String::new();
    assert_eq!(mem::size_of::<stroka::String>(), mem::size_of::<usize>() * 2);
    assert_eq!(stroka.capacity(), mem::size_of::<usize>() * 2 - 2);
}

#[test]
pub fn should_become_heap_allocated_on_buffer_overflow() {
    let mut stroka = stroka::String::new();
    let sso_capaicty = stroka.capacity();
    assert!(!stroka.is_heap());

    let mut expected_string = String::new();
    for idx in  0..sso_capaicty {
        let idx = format!("{}", (idx + 1) % 9);
        expected_string.push_str(&idx);
        stroka.push_str(&idx);
    }

    assert!(stroka.is_heap());
    assert_eq!(stroka.len(), sso_capaicty);
    assert_eq!(stroka, expected_string);
}

#[test]
pub fn should_create_sso_string() {
    let stroka = stroka::String::new_str("1");
    assert_eq!(stroka.len(), 1);
    assert!(!stroka.is_heap());
}

#[test]
pub fn should_clear_sso_string() {
    let mut stroka = stroka::String::new_sso("test");
    assert!(!stroka.is_heap());
    assert_eq!(stroka.len(), 4);
    assert_eq!(stroka, "test");

    stroka.clear();
    assert!(!stroka.is_heap());
    assert_eq!(stroka.len(), 0);
    assert_eq!(stroka, "");
}

#[test]
pub fn should_clear_heap_string() {
    const TEXT: &str = "123456789123456789123456789";
    let mut stroka = stroka::String::new_str(TEXT);
    assert!(stroka.is_heap());
    assert_eq!(stroka.len(), TEXT.len());
    assert_eq!(stroka, TEXT);

    stroka.clear();
    assert!(stroka.is_heap());
    assert_eq!(stroka.len(), 0);
    assert_eq!(stroka, "");
}

#[test]
pub fn should_create_non_heap_within_sso_capacity() {
    const MAX_CAP: usize = core::mem::size_of::<usize>() * 2 - 2;

    for idx in 0..=MAX_CAP {
        let stroka = stroka::String::with_capacity(idx);
        assert!(!stroka.is_heap());
    }
}
