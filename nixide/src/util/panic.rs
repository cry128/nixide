macro_rules! panic_issue {
    ($($arg:expr),*) => {{
        panic!(
            "{}: please open an issue on https://github.com/cry128/nixide",
            format!($($arg),*)
        )
    }};
}

macro_rules! panic_issue_call_failed {
    () => {{
        crate::util::panic_issue!("[nixide] call to `{}` failed", ::stdext::debug_name!())
    }};
    ($($arg:expr),*) => {{
        crate::util::panic_issue!("[nixide] call to `{}` failed with \"{}\"", ::stdext::debug_name!(), format!($($arg),*))
    }};
}

pub(crate) use panic_issue;
pub(crate) use panic_issue_call_failed;
