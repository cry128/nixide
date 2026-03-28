use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::ptr::NonNull;

use super::{EvalState, ValueType};
use crate::errors::{ErrorContext, NixideResult};
use crate::sys;
use crate::util::wrappers::AsInnerPtr;
use crate::util::{panic_issue_call_failed, wrap};

pub use crate::expr::values::{NixBool, NixFloat, NixInt, NixString, NixValue};

/// A Nix value
///
/// This represents any value in the Nix language, including primitives,
/// collections, and functions.
///
/// ```cpp
/// pub const ValueType_NIX_TYPE_THUNK: ValueType = 0;
/// pub const ValueType_NIX_TYPE_INT: ValueType = 1;
/// pub const ValueType_NIX_TYPE_FLOAT: ValueType = 2;
/// pub const ValueType_NIX_TYPE_BOOL: ValueType = 3;
/// pub const ValueType_NIX_TYPE_STRING: ValueType = 4;
/// pub const ValueType_NIX_TYPE_PATH: ValueType = 5;
/// pub const ValueType_NIX_TYPE_NULL: ValueType = 6;
/// pub const ValueType_NIX_TYPE_ATTRS: ValueType = 7;
/// pub const ValueType_NIX_TYPE_LIST: ValueType = 8;
/// pub const ValueType_NIX_TYPE_FUNCTION: ValueType = 9;
/// pub const ValueType_NIX_TYPE_EXTERNAL: ValueType = 10;
/// ```
pub enum Value {
    // Thunk(NixThunk),
    Int(NixInt),
    Float(NixFloat),
    Bool(NixBool),
    String(NixString),
    // Path(NixPath),
    // Null(NixNull),
    // Attrs(NixAttrs),
    // List(NixList),
    // Function(NixFunction),
    // External(NixExternal),
}

impl<'a> Value<'a> {
    pub(crate) fn new(inner: NonNull<sys::nix_value>, state: &'a EvalState) -> Self {
        Value { inner, state }
    }

    /// Force evaluation of this value.
    ///
    /// If the value is a thunk, this will evaluate it to its final form.
    ///
    /// # Errors
    ///
    /// Returns an error if evaluation fails.
    pub fn force(&mut self) -> NixideResult<()> {
        // XXX: TODO: move force and force_deep to the EvalState
        wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_value_force(ctx.as_ptr(), self.state.as_ptr(), self.as_ptr());
        })
    }

    /// Force deep evaluation of this value.
    ///
    /// This forces evaluation of the value and all its nested components.
    ///
    /// # Errors
    ///
    /// Returns an error if evaluation fails.
    pub fn force_deep(&mut self) -> NixideResult<()> {
        // XXX: TODO: move force and force_deep to the EvalState
        wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_value_force_deep(ctx.as_ptr(), self.state.as_ptr(), self.as_ptr());
        })
    }

    /// Get the type of this value.
    #[must_use]
    pub fn value_type(&self) -> ValueType {
        // NOTE: an error here only occurs if `nix_get_type` catches an error,
        // NOTE: which in turn only happens if the `sys::nix_value*` is a null pointer
        // NOTE: or points to an uninitialised `nix_value` struct.
        ValueType::from({
            wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
                sys::nix_get_type(ctx.as_ptr(), self.as_ptr())
            })
            .unwrap_or_else(|err| panic_issue_call_failed!("{}", err))
        })
    }

    // XXX: TODO: rewrite `expr/value.rs` to make this redundant
    // fn expect_type(&self, expected: ValueType) -> NixideResult<()> {
    //     let got = self.value_type();
    //     if got != expected {
    //         return Err(new_nixide_error!(
    //             InvalidType,
    //             expected.to_string(),
    //             got.to_string()
    //         ));
    //     }
    //     Ok(())
    // }

    /// Format this value as Nix syntax.
    ///
    /// This provides a string representation that matches Nix's own syntax,
    /// making it useful for debugging and displaying values to users.
    ///
    /// # Errors
    ///
    /// Returns an error if the value cannot be converted to a string
    /// representation.
    pub fn to_nix_string(&self) -> NixideResult<String> {
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
