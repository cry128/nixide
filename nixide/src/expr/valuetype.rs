use std::fmt::{Display, Formatter, Result as FmtResult};

use crate::sys;

/// Nix value types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    /// Thunk (unevaluated expression).
    Thunk,
    /// Integer value.
    Int,
    /// Float value.
    Float,
    /// Boolean value.
    Bool,
    /// String value.
    String,
    /// Path value.
    Path,
    /// Null value.
    Null,
    /// Attribute set.
    Attrs,
    /// List.
    List,
    /// Function.
    Function,
    /// External value.
    External,
}

impl ValueType {
    pub(super) fn from_c(value_type: sys::ValueType) -> Self {
        match value_type {
            sys::ValueType_NIX_TYPE_THUNK => ValueType::Thunk,
            sys::ValueType_NIX_TYPE_INT => ValueType::Int,
            sys::ValueType_NIX_TYPE_FLOAT => ValueType::Float,
            sys::ValueType_NIX_TYPE_BOOL => ValueType::Bool,
            sys::ValueType_NIX_TYPE_STRING => ValueType::String,
            sys::ValueType_NIX_TYPE_PATH => ValueType::Path,
            sys::ValueType_NIX_TYPE_NULL => ValueType::Null,
            sys::ValueType_NIX_TYPE_ATTRS => ValueType::Attrs,
            sys::ValueType_NIX_TYPE_LIST => ValueType::List,
            sys::ValueType_NIX_TYPE_FUNCTION => ValueType::Function,
            sys::ValueType_NIX_TYPE_EXTERNAL => ValueType::External,
            _ => ValueType::Thunk, // fallback (TODO: is this ok?)
        }
    }
}

impl Display for ValueType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let name = match self {
            ValueType::Thunk => "thunk",
            ValueType::Int => "int",
            ValueType::Float => "float",
            ValueType::Bool => "bool",
            ValueType::String => "string",
            ValueType::Path => "path",
            ValueType::Null => "null",
            ValueType::Attrs => "attrs",
            ValueType::List => "list",
            ValueType::Function => "function",
            ValueType::External => "external",
        };
        write!(f, "{name}")
    }
}
