use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::ptr::NonNull;

use super::{EvalState, ValueType};
use crate::sys;
use crate::NixErrorCode;

/// A Nix value
///
/// This represents any value in the Nix language, including primitives,
/// collections, and functions.
pub struct Value<'a> {
    pub(crate) inner: NonNull<sys::nix_value>,
    pub(crate) state: &'a EvalState,
}

impl Value<'_> {
    /// Force evaluation of this value.
    ///
    /// If the value is a thunk, this will evaluate it to its final form.
    ///
    /// # Errors
    ///
    /// Returns an error if evaluation fails.
    pub fn force(&mut self) -> Result<(), NixErrorCode> {
        NixErrorCode::from(
            // SAFETY: context, state, and value are valid
            unsafe {
                sys::nix_value_force(
                    self.state.context.as_ptr(),
                    self.state.as_ptr(),
                    self.inner.as_ptr(),
                )
            },
            "nix_value_force",
        )
    }

    /// Force deep evaluation of this value.
    ///
    /// This forces evaluation of the value and all its nested components.
    ///
    /// # Errors
    ///
    /// Returns an error if evaluation fails.
    pub fn force_deep(&mut self) -> Result<(), NixErrorCode> {
        NixErrorCode::from(
            // SAFETY: context, state, and value are valid
            unsafe {
                sys::nix_value_force_deep(
                    self.state.context.as_ptr(),
                    self.state.as_ptr(),
                    self.inner.as_ptr(),
                )
            },
            "nix_value_force_deep",
        )
    }

    /// Get the type of this value.
    #[must_use]
    pub fn value_type(&self) -> ValueType {
        // SAFETY: context and value are valid
        let c_type = unsafe { sys::nix_get_type(self.state.context.as_ptr(), self.inner.as_ptr()) };
        ValueType::from_c(c_type)
    }

    /// Convert this value to an integer.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is not an integer.
    pub fn as_int(&self) -> Result<i64, NixErrorCode> {
        if self.value_type() != ValueType::Int {
            return Err(NixErrorCode::InvalidType {
                location: "nixide::Value::as_int",
                expected: "int",
                got: self.value_type().to_string(),
            });
        }

        // SAFETY: context and value are valid, type is checked
        let result = unsafe { sys::nix_get_int(self.state.context.as_ptr(), self.inner.as_ptr()) };

        Ok(result)
    }

    /// Convert this value to a float.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is not a float.
    pub fn as_float(&self) -> Result<f64, NixErrorCode> {
        if self.value_type() != ValueType::Float {
            return Err(NixErrorCode::InvalidType {
                location: "nixide::Value::as_float",
                expected: "float",
                got: self.value_type().to_string(),
            });
        }

        // SAFETY: context and value are valid, type is checked
        let result =
            unsafe { sys::nix_get_float(self.state.context.as_ptr(), self.inner.as_ptr()) };

        Ok(result)
    }

    /// Convert this value to a boolean.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is not a boolean.
    pub fn as_bool(&self) -> Result<bool, NixErrorCode> {
        if self.value_type() != ValueType::Bool {
            return Err(NixErrorCode::InvalidType {
                location: "nixide::Value::as_bool",
                expected: "bool",
                got: self.value_type().to_string(),
            });
        }

        // SAFETY: context and value are valid, type is checked
        let result = unsafe { sys::nix_get_bool(self.state.context.as_ptr(), self.inner.as_ptr()) };

        Ok(result)
    }

    /// Convert this value to a string.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is not a string.
    pub fn as_string(&self) -> Result<String, NixErrorCode> {
        if self.value_type() != ValueType::String {
            return Err(NixErrorCode::InvalidType {
                location: "nixide::Value::as_string",
                expected: "string",
                got: self.value_type().to_string(),
            });
        }

        // For string values, we need to use realised string API
        // SAFETY: context and value are valid, type is checked
        let realised_str = unsafe {
            sys::nix_string_realise(
                self.state.context.as_ptr(),
                self.state.as_ptr(),
                self.inner.as_ptr(),
                false, // don't copy more
            )
        };

        if realised_str.is_null() {
            return Err(NixErrorCode::NullPtr {
                location: "nix_string_realise",
            });
        }

        // SAFETY: realised_str is non-null and points to valid RealizedString
        let buffer_start = unsafe { sys::nix_realised_string_get_buffer_start(realised_str) };
        let buffer_size = unsafe { sys::nix_realised_string_get_buffer_size(realised_str) };
        if buffer_start.is_null() {
            // Clean up realised string
            unsafe {
                sys::nix_realised_string_free(realised_str);
            }
            return Err(NixErrorCode::NullPtr {
                location: "nix_realised_string_free",
            });
        }

        // SAFETY: buffer_start is non-null and buffer_size gives us the length
        let bytes = unsafe { std::slice::from_raw_parts(buffer_start.cast::<u8>(), buffer_size) };
        let string = std::str::from_utf8(bytes)
            .map_err(|_| NixErrorCode::Unknown {
                location: "nixide::Value::as_string",
                reason: "Invalid UTF-8 in string".to_string(),
            })?
            .to_owned();

        // Clean up realised string
        unsafe {
            sys::nix_realised_string_free(realised_str);
        }

        Ok(string)
    }

    /// Get the raw value pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure the pointer is used safely.
    #[allow(dead_code)]
    unsafe fn as_ptr(&self) -> *mut sys::nix_value {
        self.inner.as_ptr()
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
    pub fn to_nix_string(&self) -> Result<String, NixErrorCode> {
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

impl Drop for Value<'_> {
    fn drop(&mut self) {
        // SAFETY: We own the value and it's valid until drop
        unsafe {
            sys::nix_value_decref(self.state.context.as_ptr(), self.inner.as_ptr());
        }
    }
}

impl Display for Value<'_> {
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

impl Debug for Value<'_> {
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
