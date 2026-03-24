use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::ptr::NonNull;

use super::{EvalState, ValueType};
use crate::errors::{new_nixide_error, ErrorContext, NixideError};
use crate::sys;
use crate::util::wrappers::{AsInnerPtr, FromC as _};
use crate::util::AsErr;

/// A Nix value
///
/// This represents any value in the Nix language, including primitives,
/// collections, and functions.
pub struct Value {
    pub(crate) inner: NonNull<sys::nix_value>,
}

impl AsInnerPtr<sys::nix_value> for Value {
    unsafe fn as_ptr(&self) -> *mut sys::nix_value {
        self.inner.as_ptr()
    }
}

impl Value {
    pub(crate) unsafe fn new(inner: *mut sys::Value) -> Self {
        Value {
            inner: NonNull::new(inner).unwrap(),
        }
    }

    /// Force evaluation of this value.
    ///
    /// If the value is a thunk, this will evaluate it to its final form.
    ///
    /// # Errors
    ///
    /// Returns an error if evaluation fails.
    pub fn force(&mut self, state: &EvalState) -> Result<(), NixideError> {
        // XXX: TODO: move force and force_deep to the EvalState
        let ctx = ErrorContext::new();

        unsafe { sys::nix_value_force(ctx.as_ptr(), state.as_ptr(), self.as_ptr()) };
        ctx.peak()
    }

    /// Force deep evaluation of this value.
    ///
    /// This forces evaluation of the value and all its nested components.
    ///
    /// # Errors
    ///
    /// Returns an error if evaluation fails.
    pub fn force_deep(&mut self, state: &EvalState) -> Result<(), NixideError> {
        let ctx = ErrorContext::new();

        unsafe { sys::nix_value_force_deep(ctx.as_ptr(), state.as_ptr(), self.as_ptr()) };
        ctx.peak()
    }

    /// Get the type of this value.
    #[must_use]
    pub fn value_type(&self) -> ValueType {
        let ctx = ErrorContext::new();
        let value_type =
            unsafe { ValueType::from_c(sys::nix_get_type(ctx.as_ptr(), self.as_ptr())) };
        // NOTE: an error here only occurs if `nix_get_type` catches an error,
        // NOTE: which in turn only happens if the `sys::nix_value*` is a null pointer
        // NOTE: or points to an uninitialised `nix_value` struct.
        ctx.peak().unwrap_or_else(|_| panic!("TODO im sleepy rn"));
        value_type
    }

    fn expect_type(&self, expected: ValueType) -> Result<(), NixideError> {
        let got = self.value_type();
        if got != expected {
            return Err(new_nixide_error!(
                InvalidType,
                expected.to_string(),
                got.to_string()
            ));
        }
        Ok(())
    }

    /// Convert this value to an integer.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is not an integer.
    pub fn as_int(&self) -> Result<i64, NixideError> {
        self.expect_type(ValueType::Int)?;

        let ctx = ErrorContext::new();
        let result = unsafe { sys::nix_get_int(ctx.as_ptr(), self.as_ptr()) };
        match ctx.peak() {
            Some(err) => Err(err),
            None => Ok(result),
        }
    }

    /// Convert this value to a float.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is not a float.
    pub fn as_float(&self) -> Result<f64, NixideError> {
        self.expect_type(ValueType::Float)?;

        let ctx = ErrorContext::new();
        let result = unsafe { sys::nix_get_float(ctx.as_ptr(), self.as_ptr()) };
        match ctx.peak() {
            Some(err) => Err(err),
            None => Ok(result),
        }
    }

    /// Convert this value to a boolean.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is not a boolean.
    pub fn as_bool(&self) -> Result<bool, NixideError> {
        self.expect_type(ValueType::Bool)?;

        let ctx = ErrorContext::new();
        let result = unsafe { sys::nix_get_bool(ctx.as_ptr(), self.as_ptr()) };
        match ctx.peak() {
            Some(err) => Err(err),
            None => Ok(result),
        }
    }

    /// Convert this value to a string.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is not a string.
    pub fn as_string(&self) -> Result<String, NixideError> {
        self.expect_type(ValueType::String)?;

        let ctx = ErrorContext::new();

        // For string values, we need to use realised string API
        let realised_str = unsafe {
            sys::nix_string_realise(
                ctx.as_ptr(),
                self.state.as_ptr(),
                self.as_ptr(),
                false, // don't copy more
            )
        };

        if realised_str.is_null() {
            return Err(new_nixide_error!(NullPtr));
        }

        let buffer_start = unsafe { sys::nix_realised_string_get_buffer_start(realised_str) };
        let buffer_size = unsafe { sys::nix_realised_string_get_buffer_size(realised_str) };
        if buffer_start.is_null() {
            // Clean up realised string
            unsafe {
                sys::nix_realised_string_free(realised_str);
            }
            return Err(new_nixide_error!(NullPtr));
        }

        let bytes = unsafe { std::slice::from_raw_parts(buffer_start.cast::<u8>(), buffer_size) };
        let string = std::str::from_utf8(bytes)
            .map_err(|_| new_nixide_error!(StringNotUtf8))?
            .to_owned();

        // Clean up realised string
        unsafe {
            sys::nix_realised_string_free(realised_str);
        }

        Ok(string)
    }

    /// Format this value as Nix syntax.
    ///
    /// This provides a string representation that matches Nix's own syntax,
    /// making it useful for debugging and displaying values to users.
    ///
    /// # Errors
    ///
    /// Returns an error if the value cannot be converted to a string
    /// representation.
    pub fn to_nix_string(&self) -> Result<String, NixideError> {
        match self.value_type() {
            ValueType::Int => Ok(self.as_int()?.to_string()),
            ValueType::Float => Ok(self.as_float()?.to_string()),
            ValueType::Bool => Ok(if self.as_bool()? {
                "true".to_string()
            } else {
                "false".to_string()
            }),
            ValueType::String => Ok(format!("\"{}\"", self.as_string()?.replace('"', "\\\""))),
            ValueType::Null => Ok("null".to_string()),
            ValueType::Attrs => Ok("{ <attrs> }".to_string()),
            ValueType::List => Ok("[ <list> ]".to_string()),
            ValueType::Function => Ok("<function>".to_string()),
            ValueType::Path => Ok("<path>".to_string()),
            ValueType::Thunk => Ok("<thunk>".to_string()),
            ValueType::External => Ok("<external>".to_string()),
        }
    }
}

impl Drop for Value {
    fn drop(&mut self) {
        let ctx = ErrorContext::new();
        unsafe {
            sys::nix_value_decref(ctx.as_ptr(), self.as_ptr());
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self.value_type() {
            ValueType::Int => {
                if let Ok(val) = self.as_int() {
                    write!(f, "{val}")
                } else {
                    write!(f, "<int error>")
                }
            }
            ValueType::Float => {
                if let Ok(val) = self.as_float() {
                    write!(f, "{val}")
                } else {
                    write!(f, "<float error>")
                }
            }
            ValueType::Bool => {
                if let Ok(val) = self.as_bool() {
                    write!(f, "{val}")
                } else {
                    write!(f, "<bool error>")
                }
            }
            ValueType::String => {
                if let Ok(val) = self.as_string() {
                    write!(f, "{val}")
                } else {
                    write!(f, "<string error>")
                }
            }
            ValueType::Null => write!(f, "null"),
            ValueType::Attrs => write!(f, "{{ <attrs> }}"),
            ValueType::List => write!(f, "[ <list> ]"),
            ValueType::Function => write!(f, "<function>"),
            ValueType::Path => write!(f, "<path>"),
            ValueType::Thunk => write!(f, "<thunk>"),
            ValueType::External => write!(f, "<external>"),
        }
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let value_type = self.value_type();
        match value_type {
            ValueType::Int => {
                if let Ok(val) = self.as_int() {
                    write!(f, "Value::Int({val})")
                } else {
                    write!(f, "Value::Int(<error>)")
                }
            }
            ValueType::Float => {
                if let Ok(val) = self.as_float() {
                    write!(f, "Value::Float({val})")
                } else {
                    write!(f, "Value::Float(<error>)")
                }
            }
            ValueType::Bool => {
                if let Ok(val) = self.as_bool() {
                    write!(f, "Value::Bool({val})")
                } else {
                    write!(f, "Value::Bool(<error>)")
                }
            }
            ValueType::String => {
                if let Ok(val) = self.as_string() {
                    write!(f, "Value::String({val:?})")
                } else {
                    write!(f, "Value::String(<error>)")
                }
            }
            ValueType::Null => write!(f, "Value::Null"),
            ValueType::Attrs => write!(f, "Value::Attrs({{ <attrs> }})"),
            ValueType::List => write!(f, "Value::List([ <list> ])"),
            ValueType::Function => write!(f, "Value::Function(<function>)"),
            ValueType::Path => write!(f, "Value::Path(<path>)"),
            ValueType::Thunk => write!(f, "Value::Thunk(<thunk>)"),
            ValueType::External => write!(f, "Value::External(<external>)"),
        }
    }
}
