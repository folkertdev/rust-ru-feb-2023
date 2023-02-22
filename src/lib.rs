use std::{
    ops::{Deref, DerefMut, Index, IndexMut, Range, RangeFrom, RangeTo},
    slice::SliceIndex,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LocalStorageVec<T, const N: usize> {
    Stack { buf: [T; N], len: usize },
    Heap(Vec<T>),
}

impl<T, const N: usize> LocalStorageVec<T, N> {
    pub fn clear(&mut self) {
        match self {
            LocalStorageVec::Stack { len, .. } => *len = 0,
            LocalStorageVec::Heap(v) => v.clear(),
        }
    }

    // Done
    pub fn len(&self) -> usize {
        match self {
            LocalStorageVec::Stack { len, .. } => *len,
            LocalStorageVec::Heap(v) => v.len(),
        }
    }
}

impl<T: Default, const N: usize> LocalStorageVec<T, N> {
    // Done
    pub fn new() -> Self {
        Self::Stack {
            buf: [(); N].map(|_| Default::default()),
            len: 0,
        }
    }

    // Done
    pub fn with_capacity(capacity: usize) -> Self {
        if capacity <= N {
            Self::new()
        } else {
            Self::Heap(Vec::with_capacity(capacity))
        }
    }

    // Done
    pub fn push(&mut self, value: T) {
        match self {
            LocalStorageVec::Stack { buf, len } if *len < N => {
                buf[*len] = value;
                *len += 1;
            }
            LocalStorageVec::Stack { buf, len } => {
                let mut v = Vec::with_capacity(*len + 1);

                for e in buf.iter_mut() {
                    v.push(std::mem::take(e));
                }

                v.push(value);

                *self = LocalStorageVec::Heap(v);
            }
            LocalStorageVec::Heap(v) => v.push(value),
        }
    }

    // Done
    pub fn pop(&mut self) -> Option<T> {
        match self {
            LocalStorageVec::Stack { buf, len } if *len > 0 => {
                let value = std::mem::take(&mut buf[*len - 1]);
                *len -= 1;
                Some(value)
            }
            Self::Stack { .. } => None,
            LocalStorageVec::Heap(v) => v.pop(),
        }
    }

    fn swap(&mut self, i: usize, j: usize) {
        assert!(i < self.len());
        assert!(j < self.len());

        match self {
            LocalStorageVec::Stack { buf, .. } => buf.swap(i, j),
            LocalStorageVec::Heap(vec) => vec.swap(i, j),
        }
    }

    pub fn insert(&mut self, index: usize, element: T) {
        self.push(element);

        // move element into position, move all later elements one over
        for i in index..self.len() {
            self.swap(i, self.len() - 1);
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

impl<T, const N: usize> From<Vec<T>> for LocalStorageVec<T, N> {
    fn from(v: Vec<T>) -> Self {
        Self::Heap(v)
    }
}

impl<T: Default, const N: usize, const M: usize> From<[T; N]> for LocalStorageVec<T, M> {
    fn from(value: [T; N]) -> Self {
        if N <= M {
            let mut it = value.into_iter();
            Self::Stack {
                buf: [(); M].map(|_| it.next().unwrap_or_default()),
                len: N,
            }
        } else {
            Self::Heap(Vec::from(value))
        }
    }
}

impl<T, const N: usize> Deref for LocalStorageVec<T, N> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T, const N: usize> DerefMut for LocalStorageVec<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl<T, const N: usize> AsRef<[T]> for LocalStorageVec<T, N> {
    fn as_ref(&self) -> &[T] {
        match self {
            LocalStorageVec::Stack { buf, len } => &buf[..*len],
            LocalStorageVec::Heap(v) => v,
        }
    }
}

impl<T, const N: usize> AsMut<[T]> for LocalStorageVec<T, N> {
    fn as_mut(&mut self) -> &mut [T] {
        match self {
            LocalStorageVec::Stack { buf, len } => &mut buf[..*len],
            LocalStorageVec::Heap(v) => v,
        }
    }
}

impl<T, const N: usize> IntoIterator for LocalStorageVec<T, N> {
    type Item = T;

    type IntoIter = StackVecIter<T, N>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            LocalStorageVec::Stack { buf, .. } => StackVecIter::Stack(buf.into_iter()),
            LocalStorageVec::Heap(vec) => StackVecIter::Heap(vec.into_iter()),
        }
    }
}

pub enum StackVecIter<T, const N: usize> {
    Stack(std::array::IntoIter<T, N>),
    Heap(std::vec::IntoIter<T>),
}

impl<T, const N: usize> Iterator for StackVecIter<T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            StackVecIter::Stack(it) => it.next(),
            StackVecIter::Heap(it) => it.next(),
        }
    }
}

impl<T, const N: usize> ExactSizeIterator for StackVecIter<T, N> {
    fn len(&self) -> usize {
        match self {
            StackVecIter::Stack(it) => it.len(),
            StackVecIter::Heap(it) => it.len(),
        }
    }
}

impl<T: Default, const N: usize> FromIterator<T> for LocalStorageVec<T, N> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let (size_min, size_max) = iter.size_hint();
        let mut vec = Self::with_capacity(size_max.unwrap_or(size_min));
        match &mut vec {
            LocalStorageVec::Stack { .. } => vec.extend(iter),
            LocalStorageVec::Heap(v) => v.extend(iter),
        }
        vec
    }
}

impl<T: Default, const N: usize> Extend<T> for LocalStorageVec<T, N> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let iter = iter.into_iter();
        let (size_min, size_max) = iter.size_hint();
        let mut vec = Self::with_capacity(size_max.unwrap_or(size_min));
        match self {
            LocalStorageVec::Stack { .. } => {
                for item in iter {
                    vec.push(item);
                }
            }
            LocalStorageVec::Heap(v) => v.extend(iter),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::LocalStorageVec;

    #[test]
    // #[cfg(feature = "disabled")]
    fn it_pushes() {
        let mut vec: LocalStorageVec<_, 128> = LocalStorageVec::new();
        for value in 0..128 {
            vec.push(value);
        }
        assert!(matches!(vec, LocalStorageVec::Stack { len: 128, .. }));
        for value in 128..256 {
            vec.push(value);
        }
        assert!(matches!(vec, LocalStorageVec::Heap(v) if v.len() == 256))
    }

    #[test]
    fn it_lens() {
        let vec: LocalStorageVec<_, 3> = LocalStorageVec::from([0, 1, 2]);
        assert_eq!(vec.len(), 3);
        let vec: LocalStorageVec<_, 2> = LocalStorageVec::from([0, 1, 2]);
        assert_eq!(vec.len(), 3);
    }

    #[test]
    // #[cfg(feature = "disabled")]
    fn it_pops() {
        let mut vec: LocalStorageVec<_, 128> = LocalStorageVec::from([0; 128]);
        for _ in 0..128 {
            assert_eq!(vec.pop(), Some(0))
        }
        assert_eq!(vec.pop(), None);

        let mut vec: LocalStorageVec<_, 128> = LocalStorageVec::from([0; 256]);
        for _ in 0..256 {
            assert_eq!(vec.pop(), Some(0))
        }
        assert_eq!(vec.pop(), None);

        let mut vec: LocalStorageVec<_, 128> = LocalStorageVec::from(vec![0; 256]);
        for _ in 0..256 {
            assert_eq!(vec.pop(), Some(0))
        }
        assert_eq!(vec.pop(), None);
    }

    #[test]
    // #[cfg(feature = "disabled")]
    fn it_inserts() {
        let mut vec: LocalStorageVec<_, 4> = LocalStorageVec::from([0, 1, 2]);
        vec.insert(1, 3);
        assert!(matches!(
            vec,
            LocalStorageVec::Stack {
                buf: [0, 3, 1, 2],
                len: 4
            }
        ));

        let mut vec: LocalStorageVec<_, 4> = LocalStorageVec::from([0, 1, 2, 3]);
        vec.insert(1, 3);
        assert!(matches!(vec, LocalStorageVec::Heap { .. }));
        assert_eq!(vec.as_ref(), &[0, 3, 1, 2, 3]);

        let mut vec: LocalStorageVec<_, 4> = LocalStorageVec::from([0, 1, 2, 3, 4]);
        vec.insert(1, 3);
        assert!(matches!(vec, LocalStorageVec::Heap { .. }));
        assert_eq!(vec.as_ref(), &[0, 3, 1, 2, 3, 4])
    }

    #[test]
    #[cfg(feature = "disabled")]
    fn it_removes() {
        let mut vec: LocalStorageVec<_, 4> = LocalStorageVec::from([0, 1, 2]);
        let elem = vec.remove(1);
        dbg!(&vec);
        assert!(matches!(
            vec,
            LocalStorageVec::Stack {
                buf: [0, 2, _, _],
                len: 2
            }
        ));
        assert_eq!(elem, 1);

        let mut vec: LocalStorageVec<_, 2> = LocalStorageVec::from([0, 1, 2]);
        let elem = vec.remove(1);
        assert!(matches!(vec, LocalStorageVec::Heap(..)));
        assert_eq!(vec.as_ref(), &[0, 2]);
        assert_eq!(elem, 1);
    }

    #[test]
    // #[cfg(feature = "disabled")]
    fn it_iters() {
        let vec: LocalStorageVec<_, 128> = LocalStorageVec::from([0; 128]);
        let mut iter = vec.into_iter();
        for item in &mut iter {
            assert_eq!(item, 0);
        }
        assert_eq!(iter.next(), None);

        let vec: LocalStorageVec<_, 128> = LocalStorageVec::from(vec![0; 128]);
        let mut iter = vec.into_iter();
        for item in &mut iter {
            assert_eq!(item, 0);
        }
        assert_eq!(iter.next(), None);
    }
}

trait LocalStorageVecIndex<I> {}

impl LocalStorageVecIndex<usize> for usize {}

impl LocalStorageVecIndex<usize> for RangeTo<usize> {}

impl LocalStorageVecIndex<usize> for Range<usize> {}

impl LocalStorageVecIndex<usize> for RangeFrom<usize> {}

impl<T: Default + Copy, I: LocalStorageVecIndex<usize>, const N: usize> Index<I>
    for LocalStorageVec<T, N>
where
    [T]: Index<I>,
{
    type Output = <[T] as Index<I>>::Output;

    fn index(&self, index: I) -> &Self::Output {
        let slice: &[T] = self.as_ref();
        slice.index(index)
    }
}

impl<T: Default + Copy, I: LocalStorageVecIndex<usize> + SliceIndex<[T]>, const N: usize>
    IndexMut<I> for LocalStorageVec<T, N>
where
    [T]: Index<I>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        let slice: &mut [T] = self.as_mut();
        slice.index_mut(index)
    }
}
