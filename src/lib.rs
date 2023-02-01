#![no_std]

use core::mem::size_of;
use core::ops::{Index, IndexMut};
use core::marker::PhantomData;
use core::ptr::NonNull;

pub struct ColumnMutIter<'a, T>{
    ptr: NonNull<[T]>,
    _lifetime: PhantomData<&'a mut [T]>,

    column_count: usize,
    offset: usize
}

impl<'a, T> ColumnMutIter<'a, T> {
    pub fn new(slice: &'a mut [T], column_count: usize) -> Self {
        assert!(column_count > 0);

        //TODO Support ZSTs
        assert!(size_of::<T>() != 0, "ZSTs are not yet supported");

        Self { 
            ptr: NonNull::from(slice), 
            _lifetime: PhantomData,
            column_count, 
            offset: 0 
        }
    }
}

impl<'a, T> Iterator for ColumnMutIter<'a, T> {
    type Item = ColumnMut<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset < self.column_count {
            let casted_ptr = unsafe{
                //TODO Add safety comment
                NonNull::new_unchecked(self.ptr.as_ptr() as *mut T)
            };

            let col = ColumnMut{
                ptr: casted_ptr, 
                _lifetime: PhantomData,
                //TODO pre-compute len
                len: (self.ptr.len() + self.column_count - self.offset - 1) / self.column_count,
                offset: self.offset,
                column_count: self.column_count
            };

            self.offset += 1;

            Some(col)
        } else {
            None
        }
    }
}

//INVARIANT: period > 0
//INVARIANT: offset < period
//INVARIANT: ptr is always non-null, well-aligned and points to a valid instance of [T]
//INVARIANT: all Column structs sharing the same slice of data simultaneously
//           must have equal `period`s and distinct `offset`s
pub struct ColumnMut<'a, T>{
    ptr: NonNull<T>,
    _lifetime: PhantomData<&'a mut [T]>,
    len: usize,
    
    column_count: usize,
    offset: usize,
}

impl<'a, T> ColumnMut<'a, T> {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn map_index(&self, index: usize) -> usize {
        index * self.column_count + self.offset
    }
    
    /*pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        
    }*/
}

impl<'a, T> Index<usize> for ColumnMut<'a, T> {
    type Output = T;
    
    fn index(&self, index: usize) -> &Self::Output {
        unsafe{
            //FIXME Add bounds checking
            & *self.ptr.as_ptr().add(self.map_index(index))
        }
    }
}

impl<'a, T> IndexMut<usize> for ColumnMut<'a, T> {    
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe{
            //FIXME Add bounds checking
            &mut *self.ptr.as_ptr().add(self.map_index(index))
        }
    }
}

unsafe impl<'a, T> Send for ColumnMut<'a, T> where [T]: Send {}
unsafe impl<'a, T> Sync for ColumnMut<'a, T> where [T]: Sync {}
