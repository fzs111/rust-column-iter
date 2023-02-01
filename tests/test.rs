
#![cfg(test)]

use column_iter::*;

//TODO: Add a LOT MORE tests!

#[test]
fn column_iter_mut() {
    let mut data = vec![0, 1, 2, 3, 4, 5, 6, 7];

    let mut cols = ColumnMutIter::new(&mut data, 3).collect::<Vec<_>>();
    
    assert_eq!(cols.len(), 3);

    assert_eq!(cols[0].len(), 3);
    assert_eq!(cols[1].len(), 3);
    assert_eq!(cols[2].len(), 2);

    cols[0][0] = 10;
    cols[1][0] = 11;
    cols[2][0] = 12;
    assert_eq!(cols[0][0], 10);
    assert_eq!(cols[1][0], 11);
    assert_eq!(cols[2][0], 12);
    assert_eq!(cols[0][1], 3);
    cols[0][1] = 13;
    cols[1][1] = 14;
    cols[2][1] = 15;
    cols[0][2] = 16;
    cols[1][2] = 17;
    
    assert_eq!(data, vec![10, 11, 12, 13, 14, 15, 16, 17]);
}
#[test]
#[should_panic(expected = "index out of bounds: the len is 8 but the index is 9")]
fn column_index_overflow() {
    let mut data = vec![0, 1, 2, 3, 4, 5, 6, 7];

    let cols = ColumnMutIter::new(&mut data, 3).collect::<Vec<_>>();


    assert_eq!(cols[0][0], 0);
    assert_eq!(cols[0][1], 3);
    assert_eq!(cols[0][2], 6);
    cols[0][3];
}