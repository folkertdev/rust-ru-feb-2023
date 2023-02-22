# LocalStorageVec

Steps

## 0. Intro

Let's create a resizable buffer called `LocalStorageVec`. Its elements live on the stack when there are only a few, but are moved to the heap when the `LocalStorageVec` length goes above a certain size. On the stack, elements are stored in a fixed-size array. On the heap they are stored in a dynamically-sized `Vec`.

**Questions**

1. When is such a data structure more efficient than a standard `Vec`?
2. What are the downsides, compared to `Vec`?

## 1. General Structure

To get started, we'll hardcode our element type as `i32`. Define

```rust
const BUF_SIZE: usize = 16;

enum LocalStorageVec { 
    // TODO
}
```

As mentioned, `LocalStorageVec` will have two variants: `Stack` and `Heap`. The `Stack` variant is struct-like, contains fields `len: usize` and `buf: [i32, BUF_SIZE]`. The `Heap` variant just wraps a `Vec<i32>`.

## 2. Basic Methods

Implement the following methods

```
impl LocalStorageVec { 
    /// Create a new, empty, stack-allocated LocalStorageVec
    fn new() -> Self { 
        todo!()
    }

    /// Create a LocalStorageVec into which `capacity` can be stored without additional allocations.
    fn with_capacity(capacity: usize) -> Self { 
        todo!()
    }

    /// Current number of elements in this LocalStorageVec
    fn len(&self) -> usize {
        todo!()
    }
}
```

Hint: The [`Vec::with_capacity`](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.with_capacity) method.

# 3. Generic

Right now, our collection can only hold `i32` values. Next we'll generalize so that it can hold any type `T` (with some restrictions). First, add a generic type parameter to the `LocalStorageVec` type and the `impl` block:

```
enum LocalStorageVec<T> { 
    Stack { 
        buf: [T; BUF_SIZE],
        len: usize,
    },
    Heap(Vec<T>),
}

impl<T> LocalStorageVec<T> { 
    ..
}
```

This will cause some type errors. You'll likely see something like this:

```
error[E0308]: mismatched types
  --> src/main.rs:14:19
   |
10 | impl<T> LocalStorageVec<T> {
   |      - this type parameter
...
14 |             buf: [0i32; 16],
   |                   ^^^^ expected type parameter `T`, found `i32`
   |
   = note: expected type parameter `T`
                        found type `i32`

error[E0277]: the trait bound `T: Copy` is not satisfied
  --> src/main.rs:14:19
   |
14 |             buf: [0i32; 16],
   |                   ^^^^ the trait `Copy` is not implemented for `T`
   |
   = note: the `Copy` trait is required because this value will be copied for each element of the array
help: consider restricting type parameter `T`
   |
10 | impl<T: std::marker::Copy> LocalStorageVec<T> {
   |       +++++++++++++++++++
```

We've been making implicit assumptions about the element type: we pick an arbitrary default value `0i32`, and `i32` is `Copy`. Using a generic type parameter makes these constraints explicit.

In this case, we will require that `T: Default`:

```
impl<T: Default> LocalStorageVec<T> {
    ..
}
```

And rework our implementation so that the `Copy` requirement disappears. When we write:

```rust
[Default::default(); BUF_SIZE]
```

this runs the `Default::default()` function once, and then tries to re-use that value for all of the elements. It can only do that when the element type is `Copy`. Instead we can run `Default::default()` for each element, like this:

```rust
[(); BUF_SIZE].map(|_| Default::default())
```

This creates an array of `()` values of the correct length, then maps over that array a function that ignores its input, and returns our generic default value.

This trick comes at a cost of course: we must produce default values to fill up the array. What if creating this value is slow? In later lectures, we'll see ways to work around this problem. For now we'll take this performance hit.

# 4. Generic Length

In the previous step, you've introduced what is called a 'generic type parameter', the `T` in `LocalStorageVec<T>`.

Previously we hardcoded the element type, and made it generic. We also hardcode the number of elements stored on the stack, and next we'll make it generic too. This is done using a 'const generic parameter'

Change the `LocalStorageVec` definition to

```rust
enum LocalStorageVec<T, const N: usize> {
    Stack { 
        len: N,
        ..
    },
    ..
}
```

and follow the compiler. The const generic also needs to be added to the `impl` block. 

## 5. From vector 

Implement

```rust
impl<T, const N: usize> From<Vec<T>> for LocalStorageVec<T, N> {
    // TODO
}
```

Our input is already a vector (the allocation has already been made), so we can just re-use it.  

## 6. Push & Pop

Next we'll implement `push` and `pop` for `LocalStorageVec`.

```rust
impl<T: Default, const N: usize> LocalStorageVec<T, N> {
    /// Appends an element to the LocalStorageVec
    fn push(&mut self, value: T) { 
        todo!()
    }

    /// Remove and return the last element (if any)
    fn pop(&mut self) -> Option<T> { 
        // when the input is Heap, just keep the output Heap too 
        // converting to stack is not needed, the allocation has been made anyway
        todo!()
    }
}
```

Hint: to remove a value from a `&mut [T; N]`, use [`std::mem::swap`](https://doc.rust-lang.org/std/mem/fn.take.html).

## 7. Convert from array

Add an implementation for:

```rust
impl<T, const N1: usize, const N2: usize> From<[T; N1]> for LocalStorageVec<T, N2> {
    // TODO
}
```
