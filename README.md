# LocalStorageVec

## Useful links

- https://doc.rust-lang.org/std/

## 0. Intro

Let's create a resizable buffer called `LocalStorageVec`. Its elements live on the stack when there are only a few, but are moved to the heap when the `LocalStorageVec` length goes above a certain size. On the stack, elements are stored in a fixed-size array. On the heap they are stored in a dynamically-sized `Vec`.

## 1. General Structure

Our data structure representation will look like this:

```rust
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LocalStorageVec<T, const N: usize> {
    Stack { buf: [T; N], len: usize },
    Heap(Vec<T>),
}
```

We have two cases

- `Stack`: stores up to `N` values on the stack. `len` tracks how many of the elements of `buf` have a useful value.
- `Heap`: stores values on the heap

The `T` type parameter makes this data structure generic over the element type. The `const N: usize` parameter controls how many elements (at most) are stored on the stack. The `N` value must be known at compile time. 

## Step 2: Basic methods

Try running

```
cargo test test2
```

This should give the following output

```
running 3 tests
test exercise::test2::is_empty ... FAILED
test exercise::test2::len_capacity_vec ... FAILED
test exercise::test2::len_capacity_array ... FAILED

failures:

---- exercise::test2::is_empty stdout ----
thread 'exercise::test2::is_empty' panicked at 'not yet implemented', src/exercise.rs:73:9
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- exercise::test2::len_capacity_vec stdout ----
thread 'exercise::test2::len_capacity_vec' panicked at 'not yet implemented', src/exercise.rs:31:9

---- exercise::test2::len_capacity_array stdout ----
thread 'exercise::test2::len_capacity_array' panicked at 'not yet implemented', src/exercise.rs:31:9


failures:
    exercise::test2::is_empty
    exercise::test2::len_capacity_array
    exercise::test2::len_capacity_vec
```

Make these tests pass by implementing the functions of `STEP 2`.


## Steps 3..=6

Like step 2, there will be a couple of skeleton definitions with `todo!`s and a module with tests that you can run with `cargo test`. Try to implement the `todo!`s so that the tests pass.

## Step 7

We're into bonus territory here. Try to implement more functions from `Vec` for our `LocalStorageVec`. `insert` and `remove` are suggestions

## Bonus

Read the documentation for `MaybeUninit` and try to use it to get rid of the `Default` bound on most functions.

