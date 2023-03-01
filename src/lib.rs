use std::ops::{Deref, DerefMut};

// ------- STEP 1 -------

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LocalStorageVec<T, const N: usize> {
    Stack { buf: [T; N], len: usize },
    Heap(Vec<T>),
}

// ------- STEP 2 -------

impl<T: Default + Copy, const N: usize> LocalStorageVec<T, N> {
    // hint: the Default instance on arrays
    pub fn new() -> Self {
        todo!()
    }

    // hint: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.with_capacity
    /// A `LocalStorageVec` with 0 elements, but which has space for `capacity` elements
    pub fn with_capacity(capacity: usize) -> Self {
        todo!()
    }
}

// implements default for any N, and any T that itself implements Default
impl<T: Default + Copy, const N: usize> Default for LocalStorageVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> LocalStorageVec<T, N> {
    // hint: `match self { .. }`
    pub fn is_empty(&self) -> usize {
        todo!()
    }

    pub fn len(&self) -> usize {
        todo!()
    }

    pub fn capacity(&self) -> usize {
        todo!()
    }
}

#[cfg(test)]
mod test2 {
    use super::*;

    #[test]
    fn len_capacity_array() {
        let lsv = LocalStorageVec::Stack {
            buf: [1u8, 2, 3, 4],
            len: 2,
        };

        assert_eq!(lsv.len(), 2);
        assert_eq!(lsv.capacity(), 4);
    }

    #[test]
    fn len_capacity_vec() {
        let lsv: LocalStorageVec<u8, 12> = LocalStorageVec::Heap(vec![1, 2, 3, 4]);

        assert_eq!(lsv.len(), 4);
        assert_eq!(lsv.capacity(), 4);

        let mut vec = Vec::with_capacity(42);
        vec.push(1);
        vec.push(2);

        let lsv: LocalStorageVec<u8, 12> = LocalStorageVec::Heap(vec);

        assert_eq!(lsv.len(), 4);
        assert_eq!(lsv.capacity(), 4);
    }

    #[test]
    fn is_empty() {
        todo!("add asserts for the vec and array case testing is_empty")
    }
}

// ------- STEP 3 -------

impl<T: Default + Copy, const N: usize> LocalStorageVec<T, N> {
    pub fn push(&mut self, value: T) {
        match self {
            LocalStorageVec::Stack { buf, len } if *len < N => {
                todo!()
            }
            LocalStorageVec::Stack { buf, len } => {
                let mut v = Vec::with_capacity(*len + 1);

                for e in buf.iter_mut() {
                    // NOTE this trick: here we are able to take a value out of a `&mut T` reference!
                    // (this works because `T` implements `Default`
                    v.push(std::mem::take(e));
                }

                v.push(value);

                *self = LocalStorageVec::Heap(v);
            }
            LocalStorageVec::Heap(v) => {
                todo!()
            }
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        match self {
            LocalStorageVec::Stack { buf, len } if *len > 0 => {
                // hint: use `std::mem::take` (see above)
                todo!()
            }
            Self::Stack { .. } => None,
            LocalStorageVec::Heap(v) => v.pop(),
        }
    }
}

#[cfg(test)]
mod test3 {
    use super::*;

    #[test]
    fn len_capacity_array() {
        let mut lsv = LocalStorageVec::Stack {
            buf: [1u8, 2, 0xAA, 0xAA],
            len: 2,
        };

        lsv.push(3);
        lsv.push(4);

        assert_eq!(lsv.len(), 4);
        assert!(matches!(lsv, LocalStorageVec::Stack { .. }));

        lsv.push(5);

        assert_eq!(lsv.len(), 5);
        assert!(matches!(lsv, LocalStorageVec::Heap(_)));
    }
}

// ------- STEP 4 -------

impl<T: Default + Copy, const N: usize> Extend<T> for LocalStorageVec<T, N> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for value in iter {
            todo!()
        }
    }
}

#[cfg(test)]
mod test4 {
    use super::*;

    #[test]
    fn dont_bend_extend() {
        let mut lsv = LocalStorageVec::Stack {
            buf: [1u8, 2, 0xAA, 0xAA],
            len: 2,
        };

        lsv.extend([3, 4]);

        assert_eq!(lsv.len(), 4);
        assert!(matches!(lsv, LocalStorageVec::Stack { .. }));

        lsv.extend(5..6);

        assert_eq!(lsv.len(), 5);
        assert!(matches!(lsv, LocalStorageVec::Heap(_)));
    }
}

// ------- STEP 5 -------

// turning our LocalStorageVec into an owned iterator over its elements involves two steps:
//
// - define an iterator type `IntoIter`, that implements Iterator.
// - implement IntoIterator for LocalStorageVec

pub enum IntoIter<T, const N: usize> {
    Stack(std::iter::Take<std::array::IntoIter<T, N>>),
    Heap(std::vec::IntoIter<T>),
}

impl<T, const N: usize> Iterator for IntoIter<T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        // hint: both the stack and heap variants contain an iterator already
        todo!()
    }
}

impl<T, const N: usize> IntoIterator for LocalStorageVec<T, N> {
    type Item = T;

    type IntoIter = IntoIter<T, N>;

    fn into_iter(self) -> Self::IntoIter {
        // hint: use the `it.take(N)` iterator method to only iterate over the first N elements
        todo!();
    }
}

#[cfg(test)]
mod test5 {
    use super::*;

    #[test]
    fn test_iter() {
        let mut lsv = LocalStorageVec::Stack {
            buf: [1u8, 2, 0xAA, 0xAA],
            len: 2,
        };

        lsv.extend([3, 4]);

        let elements: Vec<_> = lsv.into_iter().collect();

        assert_eq!(elements, vec![1, 2, 3, 4]);
    }
}

// ------- STEP 6 -------

impl<T, const N: usize> Deref for LocalStorageVec<T, N> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        todo!()
    }
}

impl<T, const N: usize> DerefMut for LocalStorageVec<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        todo!()
    }
}

#[cfg(test)]
mod test6 {
    use super::*;

    #[test]
    /// sort is implemented on `&mut [T]`, which we can use because of DerefMut
    fn test_sort() {
        let mut lsv = LocalStorageVec::Stack {
            buf: [2, 1u8, 0xAA, 0xAA],
            len: 2,
        };

        lsv.extend([4, 3]);

        lsv.sort();

        let elements: Vec<_> = lsv.into_iter().collect();

        assert_eq!(elements, vec![1, 2, 3, 4]);
    }

    #[test]
    /// indexing is implemented for `&[u8]`
    fn test_indexing() {
        let lsv = LocalStorageVec::Stack {
            buf: [2, 1u8, 0xAA, 0xAA],
            len: 2,
        };

        assert_eq!(lsv[0], 1);
    }
}

// ------- STEP 7 -------

impl<T: Default + Copy, const N: usize> LocalStorageVec<T, N> {
    pub fn insert(&mut self, index: usize, element: T) {
        todo!()
    }

    pub fn remove(&mut self, index: usize) -> T {
        todo!()
    }
}

#[cfg(test)]
mod test7 {
    use super::*;

    // write your own :)
}
