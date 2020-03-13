use std::{fmt, cmp};
use std::hash::{Hash, Hasher};
use std::convert::TryFrom;
use pct_str::PctStr;
use crate::parsing;
use super::Error;

pub struct Scheme<'a> {
	/// The scheme slice.
	pub(crate) data: &'a [u8]
}

impl<'a> Scheme<'a> {
    pub fn as_ref(&self) -> &[u8] {
		self.data
	}

    /// Get the underlying scheme slice as a string slice.
	pub fn as_str(&self) -> &str {
		unsafe {
			std::str::from_utf8_unchecked(&self.data)
		}
	}

    /// Checks if the scheme is empty.
	pub fn is_empty(&self) -> bool {
		self.data.is_empty()
	}
}

impl<'a> TryFrom<&'a str> for Scheme<'a> {
	type Error = Error;

	fn try_from(str: &'a str) -> Result<Scheme<'a>, Error> {
		let scheme_len = parsing::parse_scheme(str.as_ref(), 0)?;
		if scheme_len < str.len() {
			Err(Error::InvalidScheme)
		} else {
			Ok(Scheme {
				data: str.as_ref()
			})
		}
	}
}

impl<'a> fmt::Display for Scheme<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl<'a> fmt::Debug for Scheme<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl<'a> cmp::PartialEq for Scheme<'a> {
	fn eq(&self, other: &Scheme) -> bool {
		self.as_str() == other.as_str()
	}
}

impl<'a> Eq for Scheme<'a> { }

impl<'a> cmp::PartialEq<&'a str> for Scheme<'a> {
	fn eq(&self, other: &&'a str) -> bool {
		self.as_str() == *other
	}
}

impl<'a> Hash for Scheme<'a> {
	fn hash<H: Hasher>(&self, hasher: &mut H) {
		self.as_str().hash(hasher)
	}
}
