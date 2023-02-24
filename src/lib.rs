#![no_std]

use core::mem::size_of;
use core::num::NonZeroUsize;
use core::ops::{Index, IndexMut};
use core::marker::PhantomData;
use core::ptr::NonNull;

//INVARIANT: self.ptr is valid, well-aligned and not null
pub struct ColumnMutIter<'a, T>{
    ptr: *mut T,
    _lifetime: PhantomData<&'a mut [T]>,

    column_count: NonZeroUsize,
    column_offset: usize,
    row_count: usize,
}

impl<'a, T> ColumnMutIter<'a, T> {
    pub fn new(slice: &'a mut [T], column_count: NonZeroUsize) -> Self {
        //TODO Support ZSTs
        assert!(size_of::<T>() != 0, "ZSTs are not yet supported");

        let row_count = slice.len() / column_count;

        assert!(column_count.get() * row_count == slice.len(), "The length must be divisble by the column_count");

        Self { 
            ptr: slice.as_mut_ptr(), 
            _lifetime: PhantomData,

            column_count, 
            row_count,
            column_offset: 0, 
        }
    }
}

impl<'a, T> Iterator for ColumnMutIter<'a, T> {
    type Item = ColumnMut<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.column_offset < self.column_count.get() {
            let column_len = self.row_count;

            let ptr = if column_len > 0 {
                unsafe{
                    //SAFETY: if the length of the current row is >0, it is safe to construct its pointer

                    NonNull::new_unchecked(self.ptr.add(self.column_offset))
                }
            } else {
                //SAFETY: this will only happen when the length is 0, so the resulting pointer will never be dereferenced

                NonNull::dangling()
            };

            let col = ColumnMut{
                ptr, 
                _lifetime: PhantomData,
                len: column_len,
                column_count: self.column_count
            };

            self.column_offset += 1;

            Some(col)
        } else {
            None
        }
    }
}

//INVARIANT: offset < column_count
//INVARIANT: ptr is well-aligned and points to a valid instance of [T]
//INVARIANT: len * column_count + offset <= [T].len()
//INVARIANT: all Column structs sharing the same slice of data simultaneously
//           must have equal `period`s and distinct `offset`s
pub struct ColumnMut<'a, T>{
    ptr: NonNull<T>,
    _lifetime: PhantomData<&'a mut [T]>,
    len: usize,
    
    column_count: NonZeroUsize,
}

impl<'a, T> ColumnMut<'a, T> {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn map_index(&self, index: usize) -> usize {
        index * self.column_count.get()
    }
    
    unsafe fn get_ptr(&self, index: usize) -> *const T {
        //SAFETY: This function must be called with a valid index

        self.ptr.as_ptr().add(self.map_index(index))
    }
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len() {
            return None;
        }

        unsafe{
            //SAFETY: Access is bound-checked

            //TODO: Ask if `.cast_mut()` is safe here (it should be...)
            Some(&mut *self.get_ptr(index).cast_mut())
        }
    }
    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len() {
            return None;
        }

        unsafe{
            //SAFETY: Access is bound-checked

            Some(& *self.get_ptr(index))
        }
    }
}

impl<'a, T> Index<usize> for ColumnMut<'a, T> {
    type Output = T;
    
    fn index(&self, index: usize) -> &Self::Output {
        let len = self.len();

        self.get(index).unwrap_or_else(|| {
            panic!("index out of bounds: the len is {len} but the index is {index}")
        })
    }
}

impl<'a, T> IndexMut<usize> for ColumnMut<'a, T> {    
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let len = self.len();

        self.get_mut(index).unwrap_or_else(|| {
            panic!("index out of bounds: the len is {len} but the index is {index}")
        })
    }
}

unsafe impl<'a, T> Send for ColumnMut<'a, T> where &'a mut [T]: Send {}
unsafe impl<'a, T> Sync for ColumnMut<'a, T> where &'a mut [T]: Sync {}
