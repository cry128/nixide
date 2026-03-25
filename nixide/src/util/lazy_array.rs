use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct LazyArray<T, F>
where
    F: Fn(usize) -> T,
{
    inner: Rc<RefCell<Vec<Option<T>>>>,
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
            inner: Rc::new(RefCell::new(vec)),
            size,
            delegate,
        }
    }

    /// Returns `None` if `index < self.size` otherwise always succeeds
    /// (unless of course the callback you supply panics).
    ///
    // pub fn get(&mut self, index: usize) -> Option<&T> {
    //     // let x = self.inner.get(index).copied().and_then(|value| match value {
    //     //     Some(value) => Some(value),
    //     //     None => {
    //     //         // store the value first
    //     //         let value = (self.delegate)(index);
    //     //         self.inner[index] = Some(value);

    //     //         // now get a reference to it
    //     //         if let Some(v) = &self.inner[index] {
    //     //             return Some(v);
    //     //         }
    //     //         None
    //     //     }
    //     // })
    //     match self.inner.clone().borrow().get(index) {
    //         Some(Some(value)) => Some(value),
    //         Some(None) => {
    //             let mut inner = self.inner.clone().borrow_mut();
    //             // store the value first
    //             inner[index] = Some((self.delegate)(index));

    //             // now get a reference to it
    //             inner[index].as_ref()
    //         }
    //         None => None,
    //     }
    // }
    pub fn get(&mut self, index: usize) -> Option<Rc<T>> {
        if index >= self.size {
            return None;
        }

        // let inner = self.inner.borrow();
        if let Some(value) = self.inner.borrow()[index].as_ref() {
            return Some(Rc::new(value));
        }

        // drop(inner); // explicitly drop the borrow

        let value = (self.delegate)(index);
        self.inner.borrow_mut()[index] = Some(value);

        Some(Rc::new(self.inner.borrow()[index].unwrap()))
    }
}
