
#![cfg(test)]

use std::{num::NonZeroUsize, fmt::Debug};

use column_iter::*;

//TODO: Add a LOT MORE tests!

#[test]
fn read() {
    test_usize_read(1, 1);
    test_usize_read(5, 6);
    test_usize_read(297, 13);
    test_usize_read(13, 297);
}

#[test]
fn zero_length_slice(){
    test_usize_read(1, 0);
    test_usize_read(10000, 0);
}

#[test]
fn zst_read(){
    let mut data = vec![(); 30*45];

    test_read(&mut data, 30, 45, |_, _| ());

    //TODO Test isize::MAX-sized ZST slices

    /*
    let mut data: Vec<()> = vec![(); isize::MAX as usize];

    {

    let mut columns: Vec<ColumnMut<T>> = ColumnMutIter::new(
        &mut data, 
        NonZeroUsize::new(col_count).unwrap()
    ).collect();
    */
}

fn test_read<T, F>(data: &mut [T], col_count: usize, row_count: usize, mut validator: F) 
    where 
        F: FnMut(usize, usize) -> T,
        T: Debug + PartialEq,
{

    let mut columns: Vec<ColumnMut<T>> = ColumnMutIter::new(
        data, 
        NonZeroUsize::new(col_count).unwrap()
    ).collect();
    
    assert_eq!(columns.len(), col_count);

    for col_idx in 0..col_count {
        let col: &mut ColumnMut<T> = &mut columns[col_idx];

        assert_eq!(col.len(), row_count);
        
        for row_idx in 0..row_count {
            assert_eq!(col[row_idx], validator(col_idx, row_idx))
        }
    }
}

fn test_usize_read (col_count: usize, row_count: usize) {
    let mut data: Vec<usize> = (0..col_count * row_count).collect();

    test_read(&mut data, col_count, row_count, | col_idx, row_idx| row_idx * col_count + col_idx);
}