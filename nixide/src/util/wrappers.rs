use crate::NixideError;

pub trait AsErr<T> {
    fn as_err(self) -> Result<(), T>;
}

impl AsErr<NixideError> for Option<NixideError> {
    fn as_err(self) -> Result<(), NixideError> {
        match self {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }
}

pub trait AsInnerPtr<T> {
    /// Get a pointer to the underlying (`inner`) `libnix` C struct.
    ///
    /// # Safety
    ///
    /// Although this function isn't inherently `unsafe`, it is
    /// marked as such intentionally to force calls to be wrapped
    /// in `unsafe` blocks for clarity.
    unsafe fn as_ptr(&self) -> *mut T;
}

pub trait FromC<T> {
    /// Creates a new instance of [Self] from the underlying
    /// libnix C type [T].
    unsafe fn from_c(value: T) -> Self;
}
