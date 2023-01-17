use core::ops::{Index, IndexMut};
use std::{marker::PhantomData};

struct ColumnIterMut<'a, T>{
    data: &'a mut [T],
    period: usize,
    offset: usize
}

impl<'a, T> ColumnIterMut<'a, T> {
    fn new(data: &'a mut [T], period: usize) -> Self {
        assert!(period > 0);

        Self { data, period, offset: 0 }
    }
}

impl<'a, T> Iterator for ColumnIterMut<'a, T> {
    type Item = Column<'a, [T]>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset < self.period {
            let col = Column{
                ptr: self.data as *mut [T],
                len: (self.data.len() + self.period - self.offset - 1) / self.period, 
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

struct Column<'a, C: ?Sized>{
    ptr: *mut C,
    _lifetime: PhantomData<&'a mut C>,
    
    //INVARIANT: period > 0
    period: usize,
    //INVARIANT: offset < period
    offset: usize,
    len: usize
}

impl<'a, T> Column<'a, T> {
    fn len(&self) -> usize {
        self.len
    }
    /*
    fn new(data: &'a mut [T], period: usize, offset: usize) -> Self {
        assert!(period > 0);
        assert!(offset < period);
        
        Self { 
            ptr: data.as_ptr() as *mut T, 
            len: data.len(),
            _lifetime: PhantomData,

            period, 
            offset,
        }
    }
    */
    /*
    fn create_pair(data: &'a mut C) -> (Self, Self) {
        (
            Self { 
                ptr: data as *mut C, 
                _lifetime: PhantomData,
                period: 2, 
                offset: 0 
            },
            Self { 
                ptr: data as *mut C, 
                _lifetime: PhantomData,
                period: 2, 
                offset: 1
            }
        )
    }
    */
}

impl<'a, C: ?Sized> Index<usize> for Column<'a, C>
where C: Index<usize>
{
    type Output = C::Output;
    
    fn index(&self, index: usize) -> &Self::Output {
        unsafe{
            &(*self.ptr)[index * self.period + self.offset]
        }
    }
}

impl<'a, C: ?Sized> IndexMut<usize> for Column<'a, C>
where C: IndexMut<usize>
{    
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe{
            &mut (*self.ptr)[index * self.period + self.offset]
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;
    /*
    #[test]
    fn immutable_indexing() {
        let mut data = vec![0, 1, 2, 3, 4];

        let column = Column::new(&mut data, 2, 0);

        assert_eq!(column[0], 0);
        assert_eq!(column[1], 2);
        assert_eq!(column[2], 4);

        let column = Column::new(&mut data, 2, 1);

        assert_eq!(column[0], 1);
        assert_eq!(column[1], 3);
    }

    #[test]
    fn mutable_indexing() {
        let mut data = vec![0, 1, 2, 3, 4];

        let mut column = Column::new(&mut data, 2, 0);

        column[1] = 12;
        assert_eq!(data, vec![0, 1, 12, 3, 4]);
    }
    */
    #[test]
    fn column_iter_mut() {
        let mut data = vec![0, 1, 2, 3, 4, 5, 6, 7];

        let mut cols = ColumnIterMut::new(&mut data, 3).collect::<Vec<_>>();
        
        cols[0][0] = 10;
        cols[1][0] = 11;
        cols[2][0] = 12;
        cols[0][1] = 13;
        cols[1][1] = 14;
        cols[2][1] = 15;
        cols[0][2] = 16;
        cols[1][2] = 17;
        

        assert_eq!(data, vec![10, 11, 12, 13, 14, 15, 16, 17]);
    }
}