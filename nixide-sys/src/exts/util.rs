use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

impl Display for crate::NixErr {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Debug::fmt(self, f)
    }
}

impl Display for crate::NixVerbosity {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Debug::fmt(self, f)
    }
}
