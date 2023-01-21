use core::ops::{Index, IndexMut};
use core::marker::PhantomData;

pub struct ColumnIterMut<'a, T>{
    data: &'a mut [T],
    period: usize,
    offset: usize
}

impl<'a, T> ColumnIterMut<'a, T> {
    pub fn new(data: &'a mut [T], period: usize) -> Self {
        assert!(period > 0);

        Self { data, period, offset: 0 }
    }
}

impl<'a, T> Iterator for ColumnIterMut<'a, T> {
    type Item = Column<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset < self.period {
            let col = Column{
                ptr: self.data as *mut [T], 
                _lifetime: PhantomData,
                offset: self.offset,
                period: self.period
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
pub struct Column<'a, T>{
    ptr: *mut [T],
    _lifetime: PhantomData<&'a mut [T]>,
    
    period: usize,
    offset: usize,
}

impl<'a, T> Column<'a, T> {
    pub fn len(&self) -> usize {
        unsafe{
            ((*self.ptr).len() + self.period - self.offset - 1) / self.period
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn map_index(&self, index: usize) -> usize {
        index * self.period + self.offset
    }
}

impl<'a, T> Index<usize> for Column<'a, T> {
    type Output = T;
    
    fn index(&self, index: usize) -> &Self::Output {
        unsafe{
            //SAFETY: if the invariants are maintained, the indices returned by 
            //        `Self::map_index()` will be exclusive to this instance of the struct

            //TODO: move bound-checking to here to provide more helpful panic messages
            &(*self.ptr)[self.map_index(index)]
        }
    }
}

impl<'a, T> IndexMut<usize> for Column<'a, T> {    
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe{
            //SAFETY: if the invariants are maintained, the indices returned by 
            //        `Self::map_index()` will be exclusive to this instance of the struct

            //TODO: move bound-checking to here to provide more helpful panic messages
            &mut (*self.ptr)[self.map_index(index)]
        }
    }
}

unsafe impl<'a, T> Send for Column<'a, T> where [T]: Send {}
unsafe impl<'a, T> Sync for Column<'a, T> where [T]: Sync {}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn column_iter_mut() {
        let mut data = vec![0, 1, 2, 3, 4, 5, 6, 7];

        let mut cols = ColumnIterMut::new(&mut data, 3).collect::<Vec<_>>();
        
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

        let cols = ColumnIterMut::new(&mut data, 3).collect::<Vec<_>>();


        assert_eq!(cols[0][0], 0);
        assert_eq!(cols[0][1], 3);
        assert_eq!(cols[0][2], 6);
        cols[0][3];
    }
}
