use std::ops::{Deref, DerefMut};

// ------- STEP 1 -------

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LocalStorageVec<T, const N: usize> {
    Stack { buf: [T; N], len: usize },
    Heap(Vec<T>),
}

// ------- STEP 2 -------

impl<T: Default, const N: usize> LocalStorageVec<T, N> {
    pub fn new() -> Self {
        Self::Stack { buf:
        // alternatively `[0; N].map(|_| T::default()`
        // Just `Default::default` does not work (limitation in std)
        std::array::from_fn(|_| T::default()) , len: 0 }
    }

    /// A `LocalStorageVec` with 0 elements, but which has space for `capacity` elements
    pub fn with_capacity(capacity: usize) -> Self {
        if capacity <= N {
            Self::new()
        } else {
            Self::Heap(Vec::with_capacity(capacity))
        }
    }
}

// implements default for any N, and any T that itself implements Default
impl<T: Default, const N: usize> Default for LocalStorageVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> LocalStorageVec<T, N> {
    // hint: `match self { .. }`
    pub fn is_empty(&self) -> bool {
        match self {
            LocalStorageVec::Stack { len, .. } => *len == 0,
            LocalStorageVec::Heap(vec) => vec.is_empty(),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            LocalStorageVec::Stack { len, .. } => *len,
            LocalStorageVec::Heap(vec) => vec.len(),
        }
    }

    pub fn capacity(&self) -> usize {
        match self {
            LocalStorageVec::Stack { .. } => N,
            LocalStorageVec::Heap(vec) => vec.capacity(),
        }
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

        assert_eq!(lsv.len(), 2);
        assert_eq!(lsv.capacity(), 42);
    }

    #[test]
    fn is_empty() {
        let lsv = LocalStorageVec::Stack {
            buf: [1u8, 2, 3, 4],
            len: 2,
        };
        assert!(!lsv.is_empty());

        let lsv = LocalStorageVec::Stack {
            buf: [1u8, 2, 3, 4],
            len: 0,
        };
        assert!(lsv.is_empty());

        let lsv: LocalStorageVec<u8, 12> = LocalStorageVec::Heap(vec![1, 2, 3, 4]);
        assert!(!lsv.is_empty());

        let lsv: LocalStorageVec<u8, 12> = LocalStorageVec::Heap(vec![]);
        assert!(lsv.is_empty());
    }
}

// ------- STEP 3 -------

impl<T: Default, const N: usize> LocalStorageVec<T, N> {
    pub fn push(&mut self, value: T) {
        match self {
            LocalStorageVec::Stack { buf, len } if *len < N => {
                buf[*len] = value;
                *len += 1;
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
            LocalStorageVec::Heap(v) => v.push(value),
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        match self {
            LocalStorageVec::Stack { buf, len } if *len > 0 => {
                // hint: use `std::mem::take` (see above)
                *len -= 1;
                Some(std::mem::take(&mut buf[*len]))
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

impl<T: Default, const N: usize> Extend<T> for LocalStorageVec<T, N> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for value in iter {
            // NOTE you could be smarter here with capacity
            self.push(value)
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
        match self {
            IntoIter::Stack(it) => it.next(),
            IntoIter::Heap(it) => it.next(),
        }
    }
}

impl<T, const N: usize> IntoIterator for LocalStorageVec<T, N> {
    type Item = T;

    type IntoIter = IntoIter<T, N>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            LocalStorageVec::Stack { buf, len } => IntoIter::Stack(buf.into_iter().take(len)),
            LocalStorageVec::Heap(vec) => IntoIter::Heap(vec.into_iter()),
        }
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
        match self {
            LocalStorageVec::Stack { buf, len } => &buf[..*len],
            LocalStorageVec::Heap(vec) => vec.deref(),
        }
    }
}

impl<T, const N: usize> DerefMut for LocalStorageVec<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            LocalStorageVec::Stack { buf, len } => &mut buf[..*len],
            LocalStorageVec::Heap(vec) => vec.deref_mut(),
        }
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

        assert_eq!(lsv[0], 2);
    }
}

// ------- STEP 7 -------

impl<T: Default, const N: usize> LocalStorageVec<T, N> {
    pub fn insert(&mut self, index: usize, element: T) {
        self.push(element);

        let length = self.len();

        // move element into position, move all later elements one over
        for i in index..length {
            self.swap(i, length - 1);
        }
    }

    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.len());

        // move the item from index to the back
        for i in index..self.len() - 1 {
            self.swap(i, i + 1);
        }

        // then pop it
        self.pop().unwrap()
    }
}

#[cfg(test)]
mod test7 {
    use super::*;

    // write your own :)
}
