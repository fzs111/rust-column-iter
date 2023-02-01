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

//INVARIANT: column_count > 0
//INVARIANT: offset < column_count
//INVARIANT: ptr is well-aligned and points to a valid instance of [T]
//INVARIANT: len * column_count + offset <= [T].len()
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
