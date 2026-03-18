use std::cmp::Ordering;
use std::ffi::CStr;
use std::num::ParseIntError;

use crate::sys;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NixVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub is_prerelease: bool,
}

impl NixVersion {
    /// Constructs a new [NixVersion] struct given the raw attributes.
    ///
    /// # Warning
    ///
    /// You are most likely interested in using the [NixVersion::current]
    /// and [NixVersion::parse] functions instead of this one.
    pub fn new(major: u32, minor: u32, patch: u32, is_prerelease: bool) -> Self {
        Self {
            major,
            minor,
            patch,
            is_prerelease,
        }
    }

    /// Get the current Nix library version in the comparable [NixVersion] type.
    pub fn current() -> Result<Self, ParseIntError> {
        NixVersion::parse(NixVersion::current_string().as_ref())
    }

    /// Get the current Nix library version as an owned [String].
    pub fn current_string() -> String {
        unsafe {
            let version_ptr = sys::nix_version_get();
            CStr::from_ptr(version_ptr).to_string_lossy().into_owned()
        }
    }

    /// Parse a Nix version string into the comparable [NixVersion] type.
    ///
    /// # Examples
    ///
    /// ```
    /// use nixide::NixVersion;
    ///
    /// assert_eq!(NixVersion::parse("2.26"), Ok(NixVersion::new(2, 26, 0, false)));
    /// assert_eq!(NixVersion::parse("2.33.0pre"), Ok(NixVersion::new(2, 33, 0, true)));
    /// assert_eq!(NixVersion::parse("2.33"), Ok(NixVersion::new(2, 33, 0, false)));
    /// assert_eq!(NixVersion::parse("2.33.1"), Ok(NixVersion::new(2, 33, 1, false)));
    ///
    /// // Pre-release versions sort before stable
    /// assert!(NixVersion::parse("2.33.0pre").unwrap() < NixVersion::parse("2.33").unwrap());
    /// ```
    pub fn parse(version_str: &str) -> Result<Self, ParseIntError> {
        let parts = version_str.split('.').collect::<Vec<&str>>();
        let major = parts[0].parse::<u32>()?;
        let minor = parts[1].parse::<u32>()?;

        let (patch, is_prerelease) = match parts.get(2) {
            Some(s) => {
                let length = s.len();
                let mut offset = length;
                if length > 3 {
                    offset = offset.saturating_sub(3)
                }
                (
                    s[..offset].parse::<u32>()?,      // patch
                    length > 3 && s.ends_with("pre"), // is_prerelease
                )
            }
            None => (0, false),
        };

        Ok(Self {
            major,
            minor,
            patch,
            is_prerelease,
        })
    }
}

impl PartialOrd for NixVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else if self.major < other.major
            || self.minor < other.minor
            || (self.patch < other.patch)
            || (self.patch == other.patch && self.is_prerelease && !other.is_prerelease)
        {
            Some(Ordering::Less)
        } else {
            Some(Ordering::Greater)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NixVersion;

    #[test]
    fn test_parse_version() {
        assert_eq!(
            NixVersion::parse("2.26"),
            Ok(NixVersion::new(2, 26, 0, false))
        );
        assert_eq!(
            NixVersion::parse("2.33.0pre"),
            Ok(NixVersion::new(2, 33, 0, true))
        );
        assert_eq!(
            NixVersion::parse("2.33"),
            Ok(NixVersion::new(2, 33, 0, false))
        );
        assert_eq!(
            NixVersion::parse("2.33.1"),
            Ok(NixVersion::new(2, 33, 1, false))
        );
    }

    #[test]
    fn test_version_ordering() {
        // Pre-release versions should sort before stable
        assert!(NixVersion::parse("2.33.0pre").unwrap() < NixVersion::parse("2.33").unwrap());
        assert!(NixVersion::parse("2.33.0pre").unwrap() < NixVersion::parse("2.33.0").unwrap());

        // Normal version ordering
        assert!(NixVersion::parse("2.26").unwrap() < NixVersion::parse("2.33").unwrap());
        assert!(NixVersion::parse("2.33").unwrap() < NixVersion::parse("2.33.1").unwrap());
    }
}
