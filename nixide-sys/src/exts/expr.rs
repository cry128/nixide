use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

impl Display for crate::ValueType {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Debug::fmt(self, f)
    }
}
