use std::{fmt, cmp};
use std::hash::{Hash, Hasher};
use std::convert::TryFrom;
use pct_str::PctStr;
use crate::parsing;
use super::Error;

pub struct Segment<'a> {
	/// The path segment slice.
	pub(crate) data: &'a [u8]
}

impl<'a> Segment<'a> {
    pub fn as_ref(&self) -> &[u8] {
		self.data
	}

    /// Get the underlying segment slice as a string slice.
	pub fn as_str(&self) -> &str {
		unsafe {
			std::str::from_utf8_unchecked(&self.data)
		}
	}

    /// Get the underlying segment slice as a percent-encoded string slice.
	pub fn as_pct_str(&self) -> &PctStr {
		unsafe {
			PctStr::new_unchecked(self.as_str())
		}
	}

    /// Checks if the segment is empty.
	pub fn is_empty(&self) -> bool {
		self.data.is_empty()
	}
}

impl<'a> TryFrom<&'a str> for Segment<'a> {
	type Error = Error;

	fn try_from(str: &'a str) -> Result<Segment<'a>, Error> {
		let segment_len = parsing::parse_path_segment(str.as_ref(), 0)?;
		if segment_len < str.len() {
			Err(Error::InvalidSegment)
		} else {
			Ok(Segment {
				data: str.as_ref()
			})
		}
	}
}

impl<'a> fmt::Display for Segment<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl<'a> fmt::Debug for Segment<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl<'a> cmp::PartialEq for Segment<'a> {
	fn eq(&self, other: &Segment) -> bool {
		self.as_pct_str() == other.as_pct_str()
	}
}

impl<'a> Eq for Segment<'a> { }

impl<'a> cmp::PartialEq<&'a str> for Segment<'a> {
	fn eq(&self, other: &&'a str) -> bool {
		self.as_str() == *other
	}
}

impl<'a> Hash for Segment<'a> {
	fn hash<H: Hasher>(&self, hasher: &mut H) {
		self.as_pct_str().hash(hasher)
	}
}
