use std::{
    ops::{Deref, DerefMut, Index, IndexMut, Range, RangeFrom, RangeTo},
    slice::SliceIndex,
};

/// Steps
/// 0. Intro: Let's create a resizable buffer called `LocalStorageVec` that lives on the stack if it's short,
/// but is moved to the heap if it needs to grow large. On the stack, the
/// buffer is backed by a fixed-size array, and on the heap it's backed by a dynamically-sized `Vec`. 
/// Q1: in which case might such a type provice a perforance boost compared to `Vec`?
/// Q2: At what cost does this boost come, compared to `Vec`?
/// 1. Define const BUF_SIZE = 16. Define enum with variants Stack and Heap. 
/// Stack variant is struct-like, contains fields `len: usize` and `buf: [i32, BUF_SIZE]`
/// 2. Implement methods `new`, `len`, and `with_capacity`
/// 3. Introduce type generic param `T`, and bound the type `T: Default + Copy`. 
/// This allows us to initialize the buffer using the `[T::default(); BUF_SIZE]` syntax; Apart from making the buffer
/// less flexible in use, this comes at a performance cost we need to instantiate a default value of `T` for each slot in the buffer array.
/// This also prevents us from using the buffer with types that are `Drop`, as types that are `Copy` cannot be `Drop`.
/// 4. In the previous step, you've introduced what is called a 'type generic parameter'. To make the buffer more versatile, 
/// we will introduce a 'const generic parameter' as well. A const generic parameter is a parameter that repersent a constant. In our case,
/// it can represent the size of the array that acts as the stack buffer and can therefore replace `BUF_SIZE`. This allows
/// the user of the `LocalStorageVec` to decide how many items can fit on the stack-based variant of the buffer.
/// 5. Implement `From<Vec<T>>` for `LocalStorageVec<T>`. As the `Vec` is already on the heap, 
/// this `From` implementation simply returns the heap-backed variant of the `LocalStorageVec` into which the passed `Vec` is moved.
/// 6. Implmement `push` and `pop`. `push` appends one element to the end of the buffer, and `pop` removes and returns the last element.
/// Within `push`, if the buffer is backed by an array on the stack, the contents are moved to the heap before the element is appended.
/// If `pop` causes the length of the buffer to become smaller than `N`, it still won't move
/// any data from the heap over to the stack. In other words, a heap-backed variant is never turned into a stack-based variant, even though
/// the opposite transition sometimes occur.
/// 7. Implement From<[T; N]>

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LocalStorageVec<T: Default + Copy, const N: usize> {
    Stack { buf: [T; N], len: usize },
    Heap(Vec<T>),
}

impl<T: Default + Copy, const N: usize> LocalStorageVec<T, N> {
    // Done
    pub fn new() -> Self {
        Self::Stack {
            buf: [Default::default(); N],
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
                v.extend_from_slice(&buf[..]);
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

    // Done
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

impl<T: Default + Copy, const N: usize> Deref for LocalStorageVec<T, N> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T: Default + Copy, const N: usize> DerefMut for LocalStorageVec<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
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

impl<T: Default + Copy, const N: usize> Extend<T> for LocalStorageVec<T, N> {
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
