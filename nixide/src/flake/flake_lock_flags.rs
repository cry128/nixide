/// Parameters that affect the locking of a flake.
pub struct FlakeLockFlags {
    pub(crate) ptr: *mut raw::flake_lock_flags,
}
impl Drop for FlakeLockFlags {
    fn drop(&mut self) {
        unsafe {
            raw::flake_lock_flags_free(self.ptr);
        }
    }
}
impl FlakeLockFlags {
    pub fn new(settings: &FlakeSettings) -> Result<Self> {
        let mut ctx = Context::new();
        let s = unsafe { context::check_call!(raw::flake_lock_flags_new(&mut ctx, settings.ptr)) }?;
        Ok(FlakeLockFlags { ptr: s })
    }
    /// Configures [LockedFlake::lock] to make incremental changes to the lock file as needed. Changes are written to file.
    pub fn set_mode_write_as_needed(&mut self) -> Result<()> {
        let mut ctx = Context::new();
        unsafe {
            context::check_call!(raw::flake_lock_flags_set_mode_write_as_needed(
                &mut ctx, self.ptr
            ))
        }?;
        Ok(())
    }
    /// Make [LockedFlake::lock] check if the lock file is up to date. If not, an error is returned.
    pub fn set_mode_check(&mut self) -> Result<()> {
        let mut ctx = Context::new();
        unsafe { context::check_call!(raw::flake_lock_flags_set_mode_check(&mut ctx, self.ptr)) }?;
        Ok(())
    }
    /// Like `set_mode_write_as_needed`, but does not write to the lock file.
    pub fn set_mode_virtual(&mut self) -> Result<()> {
        let mut ctx = Context::new();
        unsafe {
            context::check_call!(raw::flake_lock_flags_set_mode_virtual(&mut ctx, self.ptr))
        }?;
        Ok(())
    }
    /// Adds an input override to the lock file that will be produced. The [LockedFlake::lock] operation will not write to the lock file.
    pub fn add_input_override(
        &mut self,
        override_path: &str,
        override_ref: &FlakeReference,
    ) -> Result<()> {
        let mut ctx = Context::new();
        unsafe {
            context::check_call!(raw::flake_lock_flags_add_input_override(
                &mut ctx,
                self.ptr,
                CString::new(override_path)
                    .context("Failed to create CString for override_path")?
                    .as_ptr(),
                override_ref.ptr.as_ptr()
            ))
        }?;
        Ok(())
    }
}
