mod attrs;
mod bool;
mod external;
mod float;
mod function;
mod int;
mod list;
mod null;
mod path;
mod string;
mod thunk;

pub use attrs::NixAttrs;
pub use bool::NixBool;
// pub use external::NixExternal;
// pub use failed::NixFailed; // only in latest nix version
pub use float::NixFloat;
pub use function::NixFunction;
pub use int::NixInt;
pub use list::NixList;
pub use null::NixNull;
pub use path::NixPath;
pub use string::NixString;
pub use thunk::NixThunk;

use std::cell::RefCell;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::ptr::NonNull;
use std::rc::Rc;

use crate::EvalState;
use crate::errors::ErrorContext;
use crate::sys;
use crate::sys::{
    ValueType_NIX_TYPE_ATTRS, ValueType_NIX_TYPE_BOOL, ValueType_NIX_TYPE_EXTERNAL,
    ValueType_NIX_TYPE_FLOAT, ValueType_NIX_TYPE_FUNCTION, ValueType_NIX_TYPE_INT,
    ValueType_NIX_TYPE_LIST, ValueType_NIX_TYPE_NULL, ValueType_NIX_TYPE_PATH,
    ValueType_NIX_TYPE_STRING, ValueType_NIX_TYPE_THUNK,
};
use crate::util::wrappers::AsInnerPtr;
use crate::util::{panic_issue_call_failed, wrap};

pub trait NixValue: Drop + Display + Debug + AsInnerPtr<sys::nix_value> {
    /// TODO
    fn type_id(&self) -> sys::ValueType;

    /// TODO
    fn from(inner: NonNull<sys::nix_value>, state: Rc<RefCell<NonNull<sys::EvalState>>>) -> Self;
}

/// A Nix value
///
/// This represents any value in the Nix language, including primitives,
/// collections, and functions.
///
/// # Nix C++ API Internals
///
/// ```cpp
/// typedef enum {
///     NIX_TYPE_THUNK    = 0,
///     NIX_TYPE_INT      = 1,
///     NIX_TYPE_FLOAT    = 2,
///     NIX_TYPE_BOOL     = 3,
///     NIX_TYPE_STRING   = 4,
///     NIX_TYPE_PATH     = 5,
///     NIX_TYPE_NULL     = 6,
///     NIX_TYPE_ATTRS    = 7,
///     NIX_TYPE_LIST     = 8,
///     NIX_TYPE_FUNCTION = 9,
///     NIX_TYPE_EXTERNAL = 10,
///     NIX_TYPE_FAILED   = 11,
///} ValueType;
/// ```
///
pub enum Value {
    /// Unevaluated expression
    ///
    /// Thunks often contain an expression and closure, but may contain other
    /// representations too.
    ///
    /// Their state is mutable, unlike that of the other types.
    Thunk(NixThunk),

    /// TODO
    Int(NixInt),

    /// TODO
    Float(NixFloat),

    /// TODO
    Bool(NixBool),

    /// TODO
    String(NixString),

    /// TODO
    Path(NixPath),

    /// TODO
    Null(NixNull),

    /// TODO
    Attrs(NixAttrs),

    /// TODO
    List(NixList),

    /// TODO
    Function(NixFunction),
    // External(NixExternal),
    // Failed(NixFailed),
}

impl
    From<(
        NonNull<sys::nix_value>,
        Rc<RefCell<NonNull<sys::EvalState>>>,
    )> for Value
{
    fn from(
        value: (
            NonNull<sys::nix_value>,
            Rc<RefCell<NonNull<sys::EvalState>>>,
        ),
    ) -> Self {
        let (inner, state) = value;

        wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_value_force(ctx.as_ptr(), state.borrow().as_ptr(), inner.as_ptr())
        })
        .unwrap_or_else(|err| panic_issue_call_failed!("{}", err));

        let type_id = wrap::nix_fn!(|ctx: &ErrorContext| unsafe {
            sys::nix_get_type(ctx.as_ptr(), inner.as_ptr())
        })
        .unwrap_or_else(|err| panic_issue_call_failed!("{}", err));

        #[allow(non_upper_case_globals)]
        match type_id {
            ValueType_NIX_TYPE_THUNK => Value::Thunk(<NixThunk as NixValue>::from(inner, state)),
            ValueType_NIX_TYPE_INT => Value::Int(<NixInt as NixValue>::from(inner, state)),
            ValueType_NIX_TYPE_FLOAT => Value::Float(<NixFloat as NixValue>::from(inner, state)),
            ValueType_NIX_TYPE_BOOL => Value::Bool(<NixBool as NixValue>::from(inner, state)),
            ValueType_NIX_TYPE_STRING => Value::String(<NixString as NixValue>::from(inner, state)),
            ValueType_NIX_TYPE_PATH => Value::Path(<NixPath as NixValue>::from(inner, state)),
            ValueType_NIX_TYPE_NULL => Value::Null(<NixNull as NixValue>::from(inner, state)),
            ValueType_NIX_TYPE_ATTRS => Value::Attrs(<NixAttrs as NixValue>::from(inner, state)),
            ValueType_NIX_TYPE_LIST => Value::List(<NixList as NixValue>::from(inner, state)),
            ValueType_NIX_TYPE_FUNCTION => {
                Value::Function(<NixFunction as NixValue>::from(inner, state))
            },
            // ValueType_NIX_TYPE_EXTERNAL => {
            //     Value::External(<NixExternal as NixValue>::from(inner, state))
            // },
            // ValueType_NIX_TYPE_FAILED => {
            //     Value::Failed(<NixFailed as NixValue>::from(inner, state))
            // },
            _ => unreachable!(),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Value::Thunk(value) => write!(f, "{value}"),
            Value::Int(value) => write!(f, "{value}"),
            Value::Float(value) => write!(f, "{value}"),
            Value::Bool(value) => write!(f, "{value}"),
            Value::String(value) => write!(f, "{value}"),
            Value::Path(value) => write!(f, "{value}"),
            Value::Null(value) => write!(f, "{value}"),
            Value::Attrs(value) => write!(f, "{value}"),
            Value::List(value) => write!(f, "{value}"),
            Value::Function(value) => write!(f, "{value}"),
            // Value::External(value) => write!(f, "{value}"),
            // Value::Failed(value) => write!(f, "{value}"),
        }
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Value::Thunk(value) => write!(f, "Value::Thunk({value:?})"),
            Value::Int(value) => write!(f, "Value::Int({value:?})"),
            Value::Float(value) => write!(f, "Value::Float({value:?})"),
            Value::Bool(value) => write!(f, "Value::Bool({value:?})"),
            Value::String(value) => write!(f, "Value::String({value:?})"),
            Value::Path(value) => write!(f, "Value::Path({value:?})"),
            Value::Null(value) => write!(f, "Value::Null({value:?})"),
            Value::Attrs(value) => write!(f, "Value::Attrs({value:?})"),
            Value::List(value) => write!(f, "Value::List({value:?})"),
            Value::Function(value) => write!(f, "Value::Function({value:?})"),
            // Value::External(value) => write!(f, "Value::External({value:?})"),
            // Value::Failed(value) => write!(f, "Value::Failed({value:?})"),
        }
    }
}

// macro_rules! is_type {
//     ($expr:expr, $tt:tt) => {{
//         match $expr {
//             $tt => true,
//             _ => false,
//         }
//     }};
// }
// pub(self) use is_type;
