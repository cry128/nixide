pub struct LockedFlake {
    pub(crate) ptr: NonNull<raw::locked_flake>,
}
impl Drop for LockedFlake {
    fn drop(&mut self) {
        unsafe {
            raw::locked_flake_free(self.ptr.as_ptr());
        }
    }
}
impl LockedFlake {
    pub fn lock(
        fetch_settings: &FetchersSettings,
        flake_settings: &FlakeSettings,
        eval_state: &EvalState,
        flags: &FlakeLockFlags,
        flake_ref: &FlakeReference,
    ) -> Result<LockedFlake> {
        let mut ctx = Context::new();
        let ptr = unsafe {
            context::check_call!(raw::flake_lock(
                &mut ctx,
                fetch_settings.raw_ptr(),
                flake_settings.ptr,
                eval_state.raw_ptr(),
                flags.ptr,
                flake_ref.ptr.as_ptr()
            ))
        }?;
        let ptr = NonNull::new(ptr).context("flake_lock unexpectedly returned null")?;
        Ok(LockedFlake { ptr })
    }

    /// Returns the outputs of the flake - the result of calling the `outputs` attribute.
    pub fn outputs(
        &self,
        flake_settings: &FlakeSettings,
        eval_state: &mut EvalState,
    ) -> Result<nix_bindings_expr::value::Value> {
        let mut ctx = Context::new();
        unsafe {
            let r = context::check_call!(raw::locked_flake_get_output_attrs(
                &mut ctx,
                flake_settings.ptr,
                eval_state.raw_ptr(),
                self.ptr.as_ptr()
            ))?;
            Ok(nix_bindings_expr::value::__private::raw_value_new(r))
        }
    }
}

#[cfg(test)]
mod tests {
    use nix_bindings_expr::eval_state::{gc_register_my_thread, EvalStateBuilder};
    use nix_bindings_store::store::Store;

    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn init() {
        // Only set experimental-features once to minimize the window where
        // concurrent Nix operations might read the setting while it's being modified
        INIT.call_once(|| {
            nix_bindings_expr::eval_state::init().unwrap();
            nix_bindings_util::settings::set("experimental-features", "flakes").unwrap();
        });
    }

    #[test]
    fn flake_settings_getflake_exists() {
        init();
        let gc_registration = gc_register_my_thread();
        let store = Store::open(None, []).unwrap();
        let mut eval_state = EvalStateBuilder::new(store)
            .unwrap()
            .flakes(&FlakeSettings::new().unwrap())
            .unwrap()
            .build()
            .unwrap();

        let v = eval_state
            .eval_from_string("builtins?getFlake", "<test>")
            .unwrap();

        let b = eval_state.require_bool(&v).unwrap();

        assert!(b);

        drop(gc_registration);
    }

    #[test]
    fn flake_lock_load_flake() {
        init();
        let gc_registration = gc_register_my_thread();
        let store = Store::open(None, []).unwrap();
        let fetchers_settings = FetchersSettings::new().unwrap();
        let flake_settings = FlakeSettings::new().unwrap();
        let mut eval_state = EvalStateBuilder::new(store)
            .unwrap()
            .flakes(&flake_settings)
            .unwrap()
            .build()
            .unwrap();

        let tmp_dir = tempfile::tempdir().unwrap();

        // Create flake.nix
        let flake_nix = tmp_dir.path().join("flake.nix");
        std::fs::write(
            &flake_nix,
            r#"
{
    outputs = { ... }: {
        hello = "potato";
    };
}
        "#,
        )
        .unwrap();

        let flake_lock_flags = FlakeLockFlags::new(&flake_settings).unwrap();

        let (flake_ref, fragment) = FlakeReference::parse_with_fragment(
            &fetchers_settings,
            &flake_settings,
            &FlakeReferenceParseFlags::new(&flake_settings).unwrap(),
            &format!("path:{}#subthing", tmp_dir.path().display()),
        )
        .unwrap();

        assert_eq!(fragment, "subthing");

        let locked_flake = LockedFlake::lock(
            &fetchers_settings,
            &flake_settings,
            &eval_state,
            &flake_lock_flags,
            &flake_ref,
        )
        .unwrap();

        let outputs = locked_flake
            .outputs(&flake_settings, &mut eval_state)
            .unwrap();

        let hello = eval_state.require_attrs_select(&outputs, "hello").unwrap();
        let hello = eval_state.require_string(&hello).unwrap();

        assert_eq!(hello, "potato");

        drop(fetchers_settings);
        drop(tmp_dir);
        drop(gc_registration);
    }

    #[test]
    fn flake_lock_load_flake_with_flags() {
        init();
        let gc_registration = gc_register_my_thread();
        let store = Store::open(None, []).unwrap();
        let fetchers_settings = FetchersSettings::new().unwrap();
        let flake_settings = FlakeSettings::new().unwrap();
        let mut eval_state = EvalStateBuilder::new(store)
            .unwrap()
            .flakes(&flake_settings)
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
                    hello = "BOB";
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

        let (flake_ref_a, fragment) = FlakeReference::parse_with_fragment(
            &fetchers_settings,
            &flake_settings,
            &flake_reference_parse_flags,
            &format!("path:{}", &flake_dir_a_str),
        )
        .unwrap();

        assert_eq!(fragment, "");

        // Step 1: Do not update (check), fails

        flake_lock_flags.set_mode_check().unwrap();

        let locked_flake = LockedFlake::lock(
            &fetchers_settings,
            &flake_settings,
            &eval_state,
            &flake_lock_flags,
            &flake_ref_a,
        );
        // Has not been locked and would need to write a lock file.
        assert!(locked_flake.is_err());
        let saved_err = match locked_flake {
            Ok(_) => panic!("Expected error, but got Ok"),
            Err(e) => e,
        };

        // Step 2: Update but do not write, succeeds
        flake_lock_flags.set_mode_virtual().unwrap();

        let locked_flake = LockedFlake::lock(
            &fetchers_settings,
            &flake_settings,
            &eval_state,
            &flake_lock_flags,
            &flake_ref_a,
        )
        .unwrap();

        let outputs = locked_flake
            .outputs(&flake_settings, &mut eval_state)
            .unwrap();

        let hello = eval_state.require_attrs_select(&outputs, "hello").unwrap();
        let hello = eval_state.require_string(&hello).unwrap();

        assert_eq!(hello, "BOB");

        // Step 3: The lock was not written, so Step 1 would fail again

        flake_lock_flags.set_mode_check().unwrap();

        let locked_flake = LockedFlake::lock(
            &fetchers_settings,
            &flake_settings,
            &eval_state,
            &flake_lock_flags,
            &flake_ref_a,
        );
        // Has not been locked and would need to write a lock file.
        assert!(locked_flake.is_err());
        match locked_flake {
            Ok(_) => panic!("Expected error, but got Ok"),
            Err(e) => {
                assert_eq!(e.to_string(), saved_err.to_string());
            }
        };

        // Step 4: Update and write, succeeds

        flake_lock_flags.set_mode_write_as_needed().unwrap();

        let locked_flake = LockedFlake::lock(
            &fetchers_settings,
            &flake_settings,
            &eval_state,
            &flake_lock_flags,
            &flake_ref_a,
        )
        .unwrap();

        let outputs = locked_flake
            .outputs(&flake_settings, &mut eval_state)
            .unwrap();
        let hello = eval_state.require_attrs_select(&outputs, "hello").unwrap();
        let hello = eval_state.require_string(&hello).unwrap();
        assert_eq!(hello, "BOB");

        // Step 5: Lock was written, so Step 1 succeeds

        flake_lock_flags.set_mode_check().unwrap();

        let locked_flake = LockedFlake::lock(
            &fetchers_settings,
            &flake_settings,
            &eval_state,
            &flake_lock_flags,
            &flake_ref_a,
        )
        .unwrap();

        let outputs = locked_flake
            .outputs(&flake_settings, &mut eval_state)
            .unwrap();
        let hello = eval_state.require_attrs_select(&outputs, "hello").unwrap();
        let hello = eval_state.require_string(&hello).unwrap();
        assert_eq!(hello, "BOB");

        // Step 6: Lock with override, do not write

        // This shouldn't matter; write_as_needed will be overridden
        flake_lock_flags.set_mode_write_as_needed().unwrap();

        let (flake_ref_c, fragment) = FlakeReference::parse_with_fragment(
            &fetchers_settings,
            &flake_settings,
            &flake_reference_parse_flags,
            &format!("path:{}", &flake_dir_c_str),
        )
        .unwrap();
        assert_eq!(fragment, "");

        flake_lock_flags
            .add_input_override("b", &flake_ref_c)
            .unwrap();

        let locked_flake = LockedFlake::lock(
            &fetchers_settings,
            &flake_settings,
            &eval_state,
            &flake_lock_flags,
            &flake_ref_a,
        )
        .unwrap();

        let outputs = locked_flake
            .outputs(&flake_settings, &mut eval_state)
            .unwrap();
        let hello = eval_state.require_attrs_select(&outputs, "hello").unwrap();
        let hello = eval_state.require_string(&hello).unwrap();
        assert_eq!(hello, "Claire");

        // Can't delete overrides, so trash it
        let mut flake_lock_flags = FlakeLockFlags::new(&flake_settings).unwrap();

        // Step 7: Override was not written; lock still points to b

        flake_lock_flags.set_mode_check().unwrap();

        let locked_flake = LockedFlake::lock(
            &fetchers_settings,
            &flake_settings,
            &eval_state,
            &flake_lock_flags,
            &flake_ref_a,
        )
        .unwrap();

        let outputs = locked_flake
            .outputs(&flake_settings, &mut eval_state)
            .unwrap();
        let hello = eval_state.require_attrs_select(&outputs, "hello").unwrap();
        let hello = eval_state.require_string(&hello).unwrap();
        assert_eq!(hello, "BOB");

        drop(fetchers_settings);
        drop(tmp_dir);
        drop(gc_registration);
    }
}
