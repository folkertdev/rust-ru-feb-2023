use std::ops::{Index, IndexMut, Range, RangeFrom, RangeTo};

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

// impl<T: Default + Copy, const N: usize> Index<usize> for LocalStorageVec<T, N> {
//     type Output = T;

//     fn index(&self, index: usize) -> &Self::Output {
//         let slice: &[T] = self.as_ref();
//         slice.index(index)
//     }
// }

// impl<T: Default + Copy, const N: usize> Index<RangeTo<usize>> for LocalStorageVec<T, N> {
//     type Output = [T];

//     fn index(&self, index: RangeTo<usize>) -> &Self::Output {
//         let slice: &[T] = self.as_ref();
//         slice.index(index)
//     }
// }

// impl<T: Default + Copy, const N: usize> Index<RangeFrom<usize>> for LocalStorageVec<T, N> {
//     type Output = [T];

//     fn index(&self, index: RangeFrom<usize>) -> &Self::Output {
//         let slice: &[T] = self.as_ref();
//         slice.index(index)
//     }
// }

// impl<T: Default + Copy, const N: usize> IndexMut<usize> for LocalStorageVec<T, N> {
//     fn index_mut(&mut self, index: usize) -> &mut Self::Output {
//         let slice: &mut [T] = self.as_mut();
//         slice.index_mut(index)
//     }
// }

// impl<T: Default + Copy, const N: usize> IndexMut<RangeFrom<usize>> for LocalStorageVec<T, N> {
//     fn index_mut(&mut self, index: RangeFrom<usize>) -> &mut Self::Output {
//         let slice: &mut [T] = self.as_mut();
//         slice.index_mut(index)
//     }
// }

// impl<T: Default + Copy, const N: usize> IndexMut<RangeTo<usize>> for LocalStorageVec<T, N> {
//     fn index_mut(&mut self, index: RangeTo<usize>) -> &mut Self::Output {
//         let slice: &mut [T] = self.as_mut();
//         slice.index_mut(index)
//     }
// }

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

fn main() {
    let vec: LocalStorageVec<_, 5> = LocalStorageVec::from([0i32, 1, 2, 3]);
    let x = &vec[..2];
    dbg!(vec);
}

#[cfg(test)]
mod test {
    use crate::LocalStorageVec;

    #[test]
    #[cfg(feature = "enabled")]
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
    #[cfg(feature = "enabled")]
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
    #[cfg(feature = "enabled")]
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

trait Indexer<T> {}

impl Indexer<usize> for usize {}

impl Indexer<usize> for RangeTo<usize> {}

impl Indexer<usize> for Range<usize> {}

impl<T: Default + Copy, I: Indexer<T>, const N: usize> Index<I> for LocalStorageVec<T, N>
where
    [T]: Index<I>,
{
    type Output = <[T] as Index<I>>::Output;

    fn index(&self, index: I) -> &Self::Output {
        let slice: &[T] = self.as_ref();
        slice.index(index)
    }
}
