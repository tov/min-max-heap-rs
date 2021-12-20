use min_max_heap::MinMaxHeap;

#[test]
fn random_20211219() {
    let mut h = MinMaxHeap::<usize>::new();

    // 0
    assert_eq!( h.replace_max(0), None );
    assert_eq!( h.len(), 1 );

    // 0 1
    h.push(1);
    assert_eq!( h.len(), 2 );
    assert_eq!( h.peek_min(), Some(&0) );
    assert_eq!( h.peek_max(), Some(&1) );

    // 0 0 1
    h.push(0);
    assert_eq!( h.len(), 3 );
    assert_eq!( h.peek_min(), Some(&0) );
    assert_eq!( h.peek_max(), Some(&1) );

    // 0 0 1 1
    h.push(1);
    assert_eq!( h.len(), 4 );
    assert_eq!( h.peek_min(), Some(&0) );
    assert_eq!( h.peek_max(), Some(&1) );

    // 0 0 0 1
    assert_eq!( h.replace_max(0), Some(1) );
    assert_eq!( h.len(), 4 );
    assert_eq!( h.peek_min(), Some(&0) );
    assert_eq!( h.peek_max(), Some(&1) );

    assert_eq!( h.into_vec_asc(), vec![0, 0, 0, 1] );
}
