pub struct FlakeReference {
    pub(crate) ptr: NonNull<raw::flake_reference>,
}
impl Drop for FlakeReference {
    fn drop(&mut self) {
        unsafe {
            raw::flake_reference_free(self.ptr.as_ptr());
        }
    }
}
impl FlakeReference {
    /// Parse a flake reference from a string.
    /// The string must be a valid flake reference, such as `github:owner/repo`.
    /// It may also be suffixed with a `#` and a fragment, such as `github:owner/repo#something`,
    /// in which case, the returned string will contain the fragment.
    pub fn parse_with_fragment(
        fetch_settings: &FetchersSettings,
        flake_settings: &FlakeSettings,
        flags: &FlakeReferenceParseFlags,
        reference: &str,
    ) -> Result<(FlakeReference, String)> {
        let mut ctx = Context::new();
        let mut r = result_string_init!();
        let mut ptr: *mut raw::flake_reference = std::ptr::null_mut();
        unsafe {
            context::check_call!(raw::flake_reference_and_fragment_from_string(
                &mut ctx,
                fetch_settings.raw_ptr(),
                flake_settings.ptr,
                flags.ptr.as_ptr(),
                reference.as_ptr() as *const c_char,
                reference.len(),
                // pointer to ptr
                &mut ptr,
                Some(callback_get_result_string),
                callback_get_result_string_data(&mut r)
            ))
        }?;
        let ptr = NonNull::new(ptr)
            .context("flake_reference_and_fragment_from_string unexpectedly returned null")?;
        Ok((FlakeReference { ptr }, r?))
    }
}
