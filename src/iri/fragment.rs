use std::{fmt, cmp};
use std::hash::{Hash, Hasher};
use std::convert::TryFrom;
use pct_str::PctStr;
use crate::parsing;
use super::Error;

pub struct Fragment<'a> {
	/// The fragment slice.
	pub(crate) data: &'a [u8]
}

impl<'a> Fragment<'a> {
    pub fn as_ref(&self) -> &[u8] {
		self.data
	}

    /// Get the underlying fragment slice as a string slice.
	pub fn as_str(&self) -> &str {
		unsafe {
			std::str::from_utf8_unchecked(&self.data)
		}
	}

    /// Get the underlying fragment slice as a percent-encoded string slice.
	pub fn as_pct_str(&self) -> &PctStr {
		unsafe {
			PctStr::new_unchecked(self.as_str())
		}
	}

    /// Checks if the fragment is empty.
	pub fn is_empty(&self) -> bool {
		self.data.is_empty()
	}
}

impl<'a> TryFrom<&'a str> for Fragment<'a> {
	type Error = Error;

	fn try_from(str: &'a str) -> Result<Fragment<'a>, Error> {
		let fragment_len = parsing::parse_fragment(str.as_ref(), 0)?;
		if fragment_len < str.len() {
			Err(Error::InvalidFragment)
		} else {
			Ok(Fragment {
				data: str.as_ref()
			})
		}
	}
}

impl<'a> fmt::Display for Fragment<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl<'a> fmt::Debug for Fragment<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl<'a> cmp::PartialEq for Fragment<'a> {
	fn eq(&self, other: &Fragment) -> bool {
		self.as_pct_str() == other.as_pct_str()
	}
}

impl<'a> Eq for Fragment<'a> { }

impl<'a> Hash for Fragment<'a> {
	fn hash<H: Hasher>(&self, hasher: &mut H) {
		self.as_pct_str().hash(hasher)
	}
}
