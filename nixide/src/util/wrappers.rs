pub trait AsInnerPtr<T> {
    /// Acquires the underlying pointer to the inner `libnix` C struct.
    ///
    /// # Safety
    ///
    /// Although this function isn't inherently `unsafe`, it is
    /// marked as such intentionally to force calls to be wrapped
    /// in `unsafe` blocks for clarity.
    unsafe fn as_ptr(&self) -> *mut T;

    /// Returns a shared reference to the inner `libnix` C struct.
    ///
    /// For the mutable counterpart see [Self::as_mut].
    ///
    /// # Safety
    ///
    /// Although this function isn't inherently `unsafe`, it is
    /// marked as such intentionally to force calls to be wrapped
    /// in `unsafe` blocks for clarity.
    unsafe fn as_ref(&self) -> &T;

    /// Returns a unique reference to the inner `libnix` C struct.
    ///
    /// For the shared counterpart see [Self::as_ref].
    ///
    /// # Safety
    ///
    /// Although this function isn't inherently `unsafe`, it is
    /// marked as such intentionally to force calls to be wrapped
    /// in `unsafe` blocks for clarity.
    unsafe fn as_mut(&mut self) -> &mut T;
}
