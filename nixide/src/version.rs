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
    /// use nixide::NixVersion::parse;
    ///
    /// assert_eq!(parse("2.26"), NixVersion::new(2, 26, 0, false));
    /// assert_eq!(parse("2.33.0pre"), NixVersion::new(2, 33, 0, true));
    /// assert_eq!(parse("2.33"), NixVersion::new(2, 33, 0, false));
    /// assert_eq!(parse("2.33.1"), NixVersion::new(2, 33, 1, false));
    ///
    /// // Pre-release versions sort before stable
    /// assert!(parse("2.33.0pre") < parse("2.33"));
    /// ```
    pub fn parse(version_str: &str) -> Result<Self, ParseIntError> {
        let parts = version_str.split('.').collect::<Vec<&str>>();
        let major = parts[0].parse::<u32>()?;
        let minor = parts[1].parse::<u32>()?;

        let (patch, is_prerelease) = match parts.get(2) {
            Some(s) => (s[..s.len() - 3].parse::<u32>()?, s.ends_with("pre")),
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
