// XXX: TODO: find a way to read directly from FlakeSettings and FetchersSettings (the C++ classes)

use std::ptr::NonNull;

use super::{FetchersSettings, FlakeLockFlags, FlakeReference, FlakeSettings};
use crate::errors::ErrorContext;
use crate::sys;
use crate::util::wrap;
use crate::util::wrappers::AsInnerPtr;
use crate::{EvalState, NixideResult, Value};

pub struct LockedFlake {
    inner: NonNull<sys::nix_locked_flake>,

    flakeref: FlakeReference,
    state: EvalState,
    flags: FlakeLockFlags,
    fetch_settings: FetchersSettings,
    flake_settings: FlakeSettings,
}

// impl Clone for LockedFlake {
//     fn clone(&self) -> Self {
//         wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
//             sys::nix_gc_incref(ctx.as_ptr(), self.as_ptr() as *mut c_void);
//         })
//         .unwrap();
//
//         Self {
//             inner: self.inner.clone(),
//             flakeref: self.flakeref.clone(),
//             state: self.state.clone(),
//             flags: self.flags.clone(),
//             fetch_settings: self.fetch_settings.clone(),
//             flake_settings: self.flake_settings.clone(),
//         }
//     }
// }

impl Drop for LockedFlake {
    fn drop(&mut self) {
        unsafe {
            sys::nix_locked_flake_free(self.as_ptr());
        }
    }
}

impl AsInnerPtr<sys::nix_locked_flake> for LockedFlake {
    #[inline]
    unsafe fn as_ptr(&self) -> *mut sys::nix_locked_flake {
        self.inner.as_ptr()
    }

    #[inline]
    unsafe fn as_ref(&self) -> &sys::nix_locked_flake {
        unsafe { self.inner.as_ref() }
    }

    #[inline]
    unsafe fn as_mut(&mut self) -> &mut sys::nix_locked_flake {
        unsafe { self.inner.as_mut() }
    }
}

impl LockedFlake {
    pub fn lock(
        fetch_settings: &FetchersSettings,
        flake_settings: &FlakeSettings,
        state: &EvalState,
        flags: &FlakeLockFlags,
        flakeref: &FlakeReference,
    ) -> NixideResult<LockedFlake> {
        let inner = wrap::nix_ptr_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_flake_lock(
                ctx.as_ptr(),
                fetch_settings.as_ptr(),
                flake_settings.as_ptr(),
                state.as_ptr(),
                flags.as_ptr(),
                flakeref.as_ptr(),
            )
        })?;

        Ok(Self {
            inner,
            flakeref: flakeref.clone(),
            state: state.clone(),
            flags: flags.clone(),
            fetch_settings: fetch_settings.clone(),
            flake_settings: flake_settings.clone(),
        })
    }

    /// Returns the outputs of the flake - the result of calling the `outputs` attribute.
    pub fn outputs(&self) -> NixideResult<Value> {
        let value = wrap::nix_ptr_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_locked_flake_get_output_attrs(
                ctx.as_ptr(),
                self.flake_settings.as_ptr(),
                self.state.as_ptr(),
                self.inner.as_ptr(),
            )
        })?;

        Ok(Value::from((value, &self.state)))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::Once;

    use super::{FetchersSettings, FlakeLockFlags, FlakeReference, FlakeSettings, LockedFlake};
    use crate::flake::{FlakeLockMode, FlakeReferenceParseFlags};
    use crate::{EvalStateBuilder, Store, Value, set_global_setting};

    static INIT: Once = Once::new();

    fn init() {
        // Only set experimental-features once to minimize the window where
        // concurrent Nix operations might read the setting while it's being modified
        INIT.call_once(|| unsafe {
            set_global_setting("experimental-features", "flakes").unwrap();
        });
    }

    #[test]
    fn flake_settings_getflake_exists() {
        init();

        let store_ref = Store::default().expect("Failed to open store connection");
        let state = EvalStateBuilder::new(store_ref.clone())
            .unwrap()
            .flakes()
            .unwrap()
            .build()
            .unwrap();

        let value = state.interpret("builtins?getFlake", "<test>").unwrap();

        assert!(matches!(value, Value::Bool(_)));
        if let Value::Bool(v) = value {
            assert!(v.as_bool());
        }
    }

    #[test]
    fn flake_lock_load_flake() {
        init();

        // Create flake.nix
        let tmp_dir = tempfile::tempdir().unwrap();
        let flake_nix = tmp_dir.path().join("flake.nix");
        fs::write(
            &flake_nix,
            r#"
{
    outputs = { ... }: {
        hello = "world";
    };
}
        "#,
        )
        .unwrap();

        let store_ref = Store::default().unwrap();
        let flake_settings = FlakeSettings::new().unwrap();

        let mut eval_state = EvalStateBuilder::new(store_ref.clone())
            .unwrap()
            .set_flake_settings(&flake_settings)
            .unwrap()
            .build()
            .unwrap();

        let fetchers_settings = FetchersSettings::new().unwrap();
        let flake_lock_flags = FlakeLockFlags::new(&flake_settings).unwrap();

        let flakeref = FlakeReference::parse(
            &fetchers_settings,
            &flake_settings,
            &FlakeReferenceParseFlags::new(&flake_settings).unwrap(),
            &format!("path:{}#subthing", tmp_dir.path().display()),
        )
        .unwrap();

        assert_eq!(flakeref.fragment(), "subthing");

        let locked_flake = LockedFlake::lock(
            &fetchers_settings,
            &flake_settings,
            &eval_state,
            &flake_lock_flags,
            &flakeref,
        )
        .unwrap();

        let outputs = locked_flake.outputs().unwrap();

        assert!(matches!(outputs, Value::Attrs(_)));
        if let Value::Attrs(outputs) = outputs {
            let value = outputs.get("hello").unwrap();

            assert!(matches!(value, Value::String(_)));
            if let Value::String(value) = value {
                assert_eq!(value.as_string(), "world");
            }
        }
    }

    #[test]
    fn flake_lock_load_flake_with_flags() {
        init();

        let store_ref = Store::default().unwrap();
        let fetchers_settings = FetchersSettings::new().unwrap();
        let flake_settings = FlakeSettings::new().unwrap();
        let mut eval_state = EvalStateBuilder::new(store_ref.clone())
            .unwrap()
            .set_flake_settings(&flake_settings)
            .unwrap()
            .build()
            .unwrap();

        let tmp_dir = tempfile::tempdir().unwrap();

        let flake_dir_a = tmp_dir.path().join("a");
        let flake_dir_b = tmp_dir.path().join("b");
        let flake_dir_c = tmp_dir.path().join("c");

        std::fs::create_dir_all(&flake_dir_a).unwrap();
        std::fs::create_dir_all(&flake_dir_b).unwrap();
        std::fs::create_dir_all(&flake_dir_c).unwrap();

        let flake_dir_a_str = flake_dir_a.to_str().unwrap();
        let flake_dir_c_str = flake_dir_c.to_str().unwrap();
        assert!(!flake_dir_a_str.is_empty());
        assert!(!flake_dir_c_str.is_empty());

        // a
        std::fs::write(
            tmp_dir.path().join("a/flake.nix"),
            r#"
            {
                inputs.b.url = "@flake_dir_b@";
                outputs = { b, ... }: {
                    hello = b.hello;
                };
            }
            "#
            .replace("@flake_dir_b@", flake_dir_b.to_str().unwrap()),
        )
        .unwrap();

        // b
        std::fs::write(
            tmp_dir.path().join("b/flake.nix"),
            r#"
            {
                outputs = { ... }: {
                    hello = "ALICE";
                };
            }
            "#,
        )
        .unwrap();

        // c
        std::fs::write(
            tmp_dir.path().join("c/flake.nix"),
            r#"
            {
                outputs = { ... }: {
                    hello = "Claire";
                };
            }
            "#,
        )
        .unwrap();

        let mut flake_lock_flags = FlakeLockFlags::new(&flake_settings).unwrap();

        let mut flake_reference_parse_flags =
            FlakeReferenceParseFlags::new(&flake_settings).unwrap();

        flake_reference_parse_flags
            .set_base_directory(tmp_dir.path().to_str().unwrap())
            .unwrap();

        let flakeref_a = FlakeReference::parse(
            &fetchers_settings,
            &flake_settings,
            &flake_reference_parse_flags,
            &format!("path:{}", &flake_dir_a_str),
        )
        .unwrap();

        assert_eq!(flakeref_a.fragment(), "");

        // Step 1: Do not update (check), fails
        flake_lock_flags.set_mode(&FlakeLockMode::Check).unwrap();

        let locked_flake = LockedFlake::lock(
            &fetchers_settings,
            &flake_settings,
            &eval_state,
            &flake_lock_flags,
            &flakeref_a,
        );
        // Has not been locked and would need to write a lock file.
        assert!(locked_flake.is_err());
        let saved_err = match locked_flake {
            Ok(_) => panic!("Expected error, but got Ok"),
            Err(e) => e,
        };

        // Step 2: Update but do not write, succeeds
        flake_lock_flags.set_mode(&FlakeLockMode::Virtual).unwrap();

        let locked_flake = LockedFlake::lock(
            &fetchers_settings,
            &flake_settings,
            &eval_state,
            &flake_lock_flags,
            &flakeref_a,
        )
        .unwrap();

        let outputs = locked_flake.outputs().unwrap();

        assert!(matches!(outputs, Value::Attrs(_)));
        if let Value::Attrs(outputs) = outputs {
            let value = outputs.get("hello").unwrap();

            assert!(matches!(value, Value::String(_)));
            if let Value::String(value) = value {
                assert_eq!(value.as_string(), "ALICE");
            }
        }

        // Step 3: The lock was not written, so Step 1 would fail again
        flake_lock_flags.set_mode(&FlakeLockMode::Check).unwrap();

        let locked_flake = LockedFlake::lock(
            &fetchers_settings,
            &flake_settings,
            &eval_state,
            &flake_lock_flags,
            &flakeref_a,
        );
        // Has not been locked and would need to write a lock file.
        match locked_flake {
            Ok(_) => panic!("Expected error, but got Ok"),
            Err(e) => {
                assert_eq!(e.to_string(), saved_err.to_string());
            },
        };

        // Step 4: Update and write, succeeds
        flake_lock_flags
            .set_mode(&FlakeLockMode::WriteAsNeeded)
            .unwrap();

        let locked_flake = LockedFlake::lock(
            &fetchers_settings,
            &flake_settings,
            &eval_state,
            &flake_lock_flags,
            &flakeref_a,
        )
        .unwrap();

        let outputs = locked_flake.outputs().unwrap();

        assert!(matches!(outputs, Value::Attrs(_)));
        if let Value::Attrs(outputs) = outputs {
            let value = outputs.get("hello").unwrap();

            assert!(matches!(value, Value::String(_)));
            if let Value::String(value) = value {
                assert_eq!(value.as_string(), "ALICE");
            }
        }

        // Step 5: Lock was written, so Step 1 succeeds
        flake_lock_flags.set_mode(&FlakeLockMode::Check).unwrap();

        let locked_flake = LockedFlake::lock(
            &fetchers_settings,
            &flake_settings,
            &eval_state,
            &flake_lock_flags,
            &flakeref_a,
        )
        .unwrap();

        let outputs = locked_flake.outputs().unwrap();

        assert!(matches!(outputs, Value::Attrs(_)));
        if let Value::Attrs(outputs) = outputs {
            let value = outputs.get("hello").unwrap();

            assert!(matches!(value, Value::String(_)));
            if let Value::String(value) = value {
                assert_eq!(value.as_string(), "ALICE");
            }
        }

        // Step 6: Lock with override, do not write

        // This shouldn't matter; write_as_needed will be overridden
        flake_lock_flags
            .set_mode(&FlakeLockMode::WriteAsNeeded)
            .unwrap();

        let flakeref_c = FlakeReference::parse(
            &fetchers_settings,
            &flake_settings,
            &flake_reference_parse_flags,
            &format!("path:{}", &flake_dir_c_str),
        )
        .unwrap();
        assert_eq!(flakeref_c.fragment(), "");

        flake_lock_flags.override_input("b", &flakeref_c).unwrap();

        let locked_flake = LockedFlake::lock(
            &fetchers_settings,
            &flake_settings,
            &eval_state,
            &flake_lock_flags,
            &flakeref_a,
        )
        .unwrap();

        let outputs = locked_flake.outputs().unwrap();

        assert!(matches!(outputs, Value::Attrs(_)));
        if let Value::Attrs(outputs) = outputs {
            let value = outputs.get("hello").unwrap();

            assert!(matches!(value, Value::String(_)));
            if let Value::String(value) = value {
                assert_eq!(value.as_string(), "Claire");
            }
        }

        // Can't delete overrides, so trash it
        let mut flake_lock_flags = FlakeLockFlags::new(&flake_settings).unwrap();

        // Step 7: Override was not written; lock still points to b
        flake_lock_flags.set_mode(&FlakeLockMode::Check).unwrap();

        let locked_flake = LockedFlake::lock(
            &fetchers_settings,
            &flake_settings,
            &eval_state,
            &flake_lock_flags,
            &flakeref_a,
        )
        .unwrap();

        let outputs = locked_flake.outputs().unwrap();

        assert!(matches!(outputs, Value::Attrs(_)));
        if let Value::Attrs(outputs) = outputs {
            let value = outputs.get("hello").unwrap();

            assert!(matches!(value, Value::String(_)));
            if let Value::String(value) = value {
                assert_eq!(value.as_string(), "ALICE");
            }
        }
    }
}
