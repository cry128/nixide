use std::iter;
use std::ptr::null_mut;

use crate::util::wrappers::AsInnerPtr;

pub trait SliceExt<T, S> {
    #[allow(unused)]
    fn as_c_array(&self) -> *mut *mut S;

    #[allow(unused)]
    fn into_c_array(self) -> *mut *mut S;
}

impl<T, S> SliceExt<T, S> for &[&T]
where
    T: AsInnerPtr<S>,
{
    fn as_c_array(&self) -> *mut *mut S {
        let mut ptrs: Vec<*mut S> = self
            .into_iter()
            .map(|x| unsafe { x.as_ptr() })
            .chain(iter::once(null_mut()))
            .collect();

        let ptr = ptrs.as_mut_ptr();
        std::mem::forget(ptrs);

        ptr
    }

    fn into_c_array(self) -> *mut *mut S {
        let mut ptrs: Vec<*mut S> = self
            .into_iter()
            .map(|x| unsafe { x.as_ptr() })
            .chain(iter::once(null_mut()))
            .collect();

        let ptr = ptrs.as_mut_ptr();
        std::mem::forget(ptrs);

        ptr
    }
}
