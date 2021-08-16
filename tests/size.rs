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
    assert!(!stroka.is_alloc());

    let sso_ptr = stroka.as_ptr();
    assert_eq!(sso_ptr, stroka.as_mut_ptr());

    let mut expected_string = String::new();
    for idx in  0..sso_capaicty {
        let idx = format!("{}", (idx + 1) % 9);
        expected_string.push_str(&idx);
        stroka.push_str(&idx);

        assert!(!stroka.is_alloc());
        assert_eq!(stroka.as_ptr(), sso_ptr);
        assert_eq!(sso_ptr, stroka.as_mut_ptr());
        assert_eq!(stroka.as_bytes(), expected_string.as_bytes());
        assert_eq!(stroka.as_str(), expected_string.as_str());
        assert_eq!(stroka.as_mut_str(), expected_string.as_mut_str());
        assert_eq!(stroka.len(), expected_string.len());
    }

    let idx = format!("{}", (5 + 1) % 9);
    expected_string.push_str(&idx);
    stroka.push_str(&idx);

    assert!(stroka.is_alloc());
    assert_eq!(stroka.len(), sso_capaicty+1);
    assert_eq!(stroka, expected_string);
    assert_ne!(stroka.as_ptr(), sso_ptr);
    assert_ne!(sso_ptr, stroka.as_mut_ptr());
    assert_eq!(stroka.as_bytes(), expected_string.as_bytes());
    assert_eq!(stroka.as_str(), expected_string.as_str());
    assert_eq!(stroka.as_mut_str(), expected_string.as_mut_str());
    assert_eq!(stroka.len(), expected_string.len());

    //Should remain heap allocated
    assert!(stroka.pop().is_some());
    assert!(stroka.pop().is_some());
    assert!(stroka.is_alloc());
    assert_eq!(stroka.len(), sso_capaicty-1);
}

#[test]
pub fn should_create_sso_string() {
    let stroka = stroka::String::new_str("1");
    assert_eq!(stroka.len(), 1);
    assert!(!stroka.is_alloc());
}

#[test]
pub fn should_clear_sso_string() {
    let mut stroka = stroka::String::new_sso("test");
    assert!(!stroka.is_alloc());
    assert_eq!(stroka.len(), 4);
    assert_eq!(stroka, "test");
    assert!(!stroka.is_empty());
    assert!(stroka.pop().is_some());

    stroka.clear();
    assert!(!stroka.is_alloc());
    assert_eq!(stroka.len(), 0);
    assert_eq!(stroka, "");
    assert!(stroka.is_empty());
    assert!(stroka.pop().is_none());
}

#[test]
pub fn should_clear_heap_string() {
    const TEXT: &str = "123456789123456789123456789";
    let mut stroka = stroka::String::new_str(TEXT);
    assert!(stroka.is_alloc());
    assert_eq!(stroka.len(), TEXT.len());
    assert_eq!(stroka, TEXT);
    assert_eq!(format!("{:?}", stroka), format!("{:?}", TEXT));
    assert!(!stroka.is_empty());
    assert!(stroka.pop().is_some());

    stroka.clear();
    assert!(stroka.is_alloc());
    assert_eq!(stroka.len(), 0);
    assert_eq!(stroka, "");
    assert_eq!(format!("{:?}", stroka), format!("{:?}", ""));
    assert!(stroka.is_empty());
    assert!(stroka.pop().is_none());
}

#[test]
pub fn should_create_non_heap_within_sso_capacity() {
    const MAX_CAP: usize = core::mem::size_of::<usize>() * 2 - 2;

    for idx in 0..=MAX_CAP {
        let stroka = stroka::String::with_capacity(idx);
        assert_eq!(stroka.capacity(), MAX_CAP);
        assert!(!stroka.is_alloc());
    }

    //Overflow capacity
    let stroka = stroka::String::with_capacity(MAX_CAP+1);
    assert!(stroka.is_alloc());
}

#[test]
pub fn should_create_not_reserve_heap_within_sso_capacity() {
    const MAX_CAP: usize = core::mem::size_of::<usize>() * 2 - 2;
    let mut stroka = stroka::String::with_capacity(0);

    for idx in 0..=MAX_CAP {
        stroka.reserve(idx);
        stroka.reserve_exact(idx);
        assert_eq!(stroka.capacity(), MAX_CAP);
        assert!(!stroka.is_alloc());
    }

    //Overflow capacity
    stroka.reserve(MAX_CAP+1);
    assert!(stroka.is_alloc());
    let mut stroka = stroka::String::with_capacity(0);
    stroka.reserve_exact(MAX_CAP+1);
    assert!(stroka.is_alloc());
}

#[test]
pub fn should_shrink_heap_capacity() {
    const MAX_CAP: usize = core::mem::size_of::<usize>() * 2 - 2;
    let mut stroka = stroka::String::with_capacity(MAX_CAP+1);
    assert!(stroka.capacity() > MAX_CAP);

    assert!(stroka.is_alloc());
    stroka.shrink_to_fit();
    assert!(stroka.is_alloc());
    assert_eq!(stroka.capacity(), 0);
}

#[test]
pub fn should_truncate_sso_string() {
    let mut stroka = stroka::String::new_sso("ろり");
    assert!(!stroka.is_alloc());
    assert_eq!(stroka.len(), 6);
    stroka.truncate(3);
    assert_eq!(stroka.len(), 3);
    stroka.clear();
    assert_eq!(stroka.len(), 0);
}

#[test]
pub fn should_truncate_heap_string() {
    let mut stroka = stroka::String::new_str("123456789123456789ろり");
    assert!(stroka.is_alloc());
    assert_eq!(stroka.len(), 24);
    stroka.truncate(21);
    assert_eq!(stroka.len(), 21);
    stroka.clear();
    assert_eq!(stroka.len(), 0);
}
