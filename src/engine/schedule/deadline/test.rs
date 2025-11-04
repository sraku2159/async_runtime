use crate::engine::schedule::deadline::Heap;

#[test]
fn insert_heap_asc() {
    let mut heap = Heap::new();

    (0..10).into_iter().for_each(|v| {
        heap.insert(v);
    });

    (0..10).into_iter().for_each(|v| {
        assert_eq!(v, heap.delete().unwrap());
    });
}

#[test]
fn insert_heap_desc() {
    let mut heap = Heap::new();

    (0..10).into_iter().rev().for_each(|v| heap.insert(v));

    (0..10)
        .into_iter()
        .for_each(|v| assert_eq!(v, heap.delete().unwrap()));
}

#[test]
fn insert_heap_random() {
    let mut heap = Heap::new();
    let values = vec![5, 2, 8, 1, 9, 3, 7, 4, 6, 0];

    values.into_iter().for_each(|v| heap.insert(v));

    (0..10)
        .into_iter()
        .for_each(|v| assert_eq!(v, heap.delete().unwrap()));
}

#[test]
fn insert_heap_with_duplicates() {
    let mut heap = Heap::new();
    let values = vec![5, 2, 5, 1, 2, 3, 1, 4, 3, 2];

    values.into_iter().for_each(|v| heap.insert(v));

    let expected = vec![1, 1, 2, 2, 2, 3, 3, 4, 5, 5];
    expected
        .into_iter()
        .for_each(|v| assert_eq!(v, heap.delete().unwrap()));
}

#[test]
fn delete_from_empty_heap() {
    let mut heap: Heap<i32> = Heap::new();
    assert_eq!(None, heap.delete());
}

#[test]
fn delete_single_element() {
    let mut heap = Heap::new();
    heap.insert(42);
    assert_eq!(Some(42), heap.delete());
    assert_eq!(None, heap.delete());
}

#[test]
fn insert_and_delete_interleaved() {
    let mut heap = Heap::new();

    heap.insert(5);
    heap.insert(3);
    assert_eq!(Some(3), heap.delete());

    heap.insert(7);
    heap.insert(1);
    assert_eq!(Some(1), heap.delete());

    heap.insert(4);
    assert_eq!(Some(4), heap.delete());
    assert_eq!(Some(5), heap.delete());
    assert_eq!(Some(7), heap.delete());
    assert_eq!(None, heap.delete());
}

#[test]
fn large_heap() {
    let mut heap = Heap::new();
    let n = 1000;

    // ランダムっぽい順序で挿入
    for i in 0..n {
        heap.insert((i * 7 + 13) % n);
    }

    // 昇順で取り出されることを確認
    for i in 0..n {
        assert_eq!(Some(i), heap.delete());
    }
}
