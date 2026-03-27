use std::cell::{Ref, RefCell};

#[derive(Debug)]
pub struct LazyArray<T, F = fn(usize) -> T>
where
    F: Fn(usize) -> T,
{
    inner: RefCell<Vec<Option<T>>>,
    size: usize,
    delegate: F,
}

impl<T, F> LazyArray<T, F>
where
    F: Fn(usize) -> T,
{
    pub fn new(size: usize, delegate: F) -> LazyArray<T, F> {
        let mut vec = Vec::with_capacity(size);
        for _ in 0..size {
            vec.push(None);
        }

        LazyArray {
            inner: RefCell::new(vec),
            size,
            delegate,
        }
    }

    /// Returns `None` if `index >= self.size` otherwise always succeeds
    /// (unless of course the callback you supply panics).
    ///
    pub fn get<'a>(&'a mut self, index: usize) -> Option<Ref<'a, Option<T>>> {
        let borrowed = self.inner.borrow();

        if index >= self.size {
            return None;
        } else if borrowed[index].is_none() {
            let value = (self.delegate)(index);
            self.inner.borrow_mut()[index] = Some(value);
        }

        Some(Ref::map(borrowed, |v| &v[index]))
    }
}
