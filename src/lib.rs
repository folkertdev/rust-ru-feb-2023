use std::{
    ops::{Index, IndexMut, Range, RangeFrom, RangeTo},
    slice::SliceIndex,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LocalStorageVec<T: Default + Copy, const N: usize> {
    Stack { buf: [T; N], len: usize },
    Heap(Vec<T>),
}

impl<T: Default + Copy, const N: usize> LocalStorageVec<T, N> {
    pub fn new() -> Self {
        Self::Stack {
            buf: [Default::default(); N],
            len: 0,
        }
    }

    pub fn push(&mut self, value: T) {
        match self {
            LocalStorageVec::Stack { buf, len } if *len < 128 => {
                buf[*len] = value;
                *len += 1;
            }
            LocalStorageVec::Stack { buf, len } => {
                let mut v = Vec::with_capacity(*len + 1);
                v.extend_from_slice(&buf[..]);
                v.push(value);
                *self = LocalStorageVec::Heap(v);
            }
            LocalStorageVec::Heap(v) => v.push(value),
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        match self {
            LocalStorageVec::Stack { buf, len } if *len > 0 => {
                let value = buf[*len - 1];
                *len -= 1;
                Some(value)
            }
            Self::Stack { .. } => None,
            LocalStorageVec::Heap(v) => v.pop(),
        }
    }

    pub fn insert(&mut self, index: usize, element: T) {
        match self {
            LocalStorageVec::Stack { buf, len } if *len < N => {
                buf.copy_within(index..*len, index + 1);
                buf[index] = element;
                *len += 1;
            }
            LocalStorageVec::Stack { buf, len } => {
                let mut v = Vec::with_capacity(*len + 1);
                v.extend_from_slice(&buf[..index]);
                v.push(element);
                v.extend_from_slice(&buf[index..]);
                *self = Self::Heap(v);
            }
            LocalStorageVec::Heap(v) => v.insert(index, element),
        }
    }

    pub fn remove(&mut self, index: usize) -> T {
        match self {
            LocalStorageVec::Stack { buf, len } => {
                let element = buf[index];
                buf.copy_within(index + 1.., index);
                *len -= 1;
                element
            }
            LocalStorageVec::Heap(v) => v.remove(index),
        }
    }

    pub fn clear(&mut self) {
        match self {
            LocalStorageVec::Stack { len, .. } => *len = 0,
            LocalStorageVec::Heap(v) => v.clear(),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            LocalStorageVec::Stack { len, .. } => *len,
            LocalStorageVec::Heap(v) => v.len(),
        }
    }
}

impl<T: Default + Copy, const N: usize> From<Vec<T>> for LocalStorageVec<T, N> {
    fn from(v: Vec<T>) -> Self {
        Self::Heap(v)
    }
}

impl<T: Default + Copy, const N: usize, const M: usize> From<[T; N]> for LocalStorageVec<T, M> {
    fn from(value: [T; N]) -> Self {
        if N <= M {
            match <[T; M]>::try_from(&value[..]) {
                Ok(buf) => Self::Stack { buf, len: M },
                Err(_) => {
                    let mut buf = [T::default(); M];
                    buf[..N].copy_from_slice(&value[..]);
                    Self::Stack { buf, len: N }
                }
            }
        } else {
            let v = value.to_vec();
            Self::Heap(v)
        }
    }
}

impl<T: Default + Copy, const N: usize> AsRef<[T]> for LocalStorageVec<T, N> {
    fn as_ref(&self) -> &[T] {
        match self {
            LocalStorageVec::Stack { buf, len } => &buf[..*len],
            LocalStorageVec::Heap(v) => v,
        }
    }
}

impl<T: Default + Copy, const N: usize> AsMut<[T]> for LocalStorageVec<T, N> {
    fn as_mut(&mut self) -> &mut [T] {
        match self {
            LocalStorageVec::Stack { buf, len } => &mut buf[..*len],
            LocalStorageVec::Heap(v) => v,
        }
    }
}

impl<T: Default + Copy, const N: usize> IntoIterator for LocalStorageVec<T, N> {
    type Item = T;

    type IntoIter = StackVecIter<T, N>;

    fn into_iter(self) -> Self::IntoIter {
        StackVecIter {
            vec: self,
            counter: 0,
        }
    }
}

pub struct StackVecIter<T: Default + Copy, const N: usize> {
    vec: LocalStorageVec<T, N>,
    counter: usize,
}

impl<T: Default + Copy, const N: usize> Iterator for StackVecIter<T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.counter >= self.vec.len() {
            return None;
        }
        let value = match &self.vec {
            LocalStorageVec::Stack { buf, .. } => buf[self.counter],
            LocalStorageVec::Heap(v) => v[self.counter],
        };
        self.counter += 1;
        Some(value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.vec.len() - self.counter;
        (remaining, Some(remaining))
    }
}

impl<T: Default + Copy, const N: usize> ExactSizeIterator for StackVecIter<T, N> {
    fn len(&self) -> usize {
        self.vec.len()
    }
}

impl<T: Default + Copy, const N: usize> FromIterator<T> for LocalStorageVec<T, N> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        // TODO
    }
}

impl<T: Default + Copy, const N: usize> Extend<T> for LocalStorageVec<T, N> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        // TODO
    }
}

#[cfg(test)]
mod test {
    use crate::LocalStorageVec;

    #[test]
    #[cfg(feature = "disabled")]
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
    #[cfg(feature = "disabled")]
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
    #[cfg(feature = "disabled")]
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
    // #[cfg(feature = "disabled")]
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
    #[cfg(feature = "disabled")]
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
