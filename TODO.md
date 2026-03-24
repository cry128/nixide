- [ ] rename `AsInnerPtr::as_ptr` to `AsInnerPtr::as_mut_ptr`

- [ ] add NixError::from_nonnull that replaces calls to NonNull::new(...).ok_or(...)
- [ ] replace all `use nixide_sys as sys;` -> `use crate::sys;`
- [ ] store NonNull pointers in structs!
- [ ] improve documentation situation on context.rs

- [ ] rename `as_ptr()` to `as_inner_ptr()` or `inner_ptr()`?
- [ ] ^^^ this fn should be added to a trait (maybe just `trait NixStructWrapper : AsPtr { ... }`)
- [ ] ^^^ also make `as_ptr()` public

- [ ] add mutexs and make the library thread safe!!

- [ ] grep all `self.inner.as_ptr()` calls and replace them with `self.as_ptr()`


- [ ] `ErrorContext::peak` should return `Result<(), NixideError>` **not** `Option<NixideError>`
- [ ] `self.expect_type` should instead be a macro to preserve the trace macro location

- [ ] make `Value` an enum instead because like duhh

- [ ] ensure we're always calling `ctx.peak()` unless it's ACTUALLY not necessary

- [ ] replace *most* calls to `ErrorContext::peak()` with `ErrorContext::pop()`
