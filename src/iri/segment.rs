use std::{fmt, cmp};
use std::cmp::{PartialOrd, Ord, Ordering};
use std::hash::{Hash, Hasher};
use std::convert::TryFrom;
use pct_str::PctStr;
use crate::parsing;
use super::Error;

#[derive(Clone, Copy)]
pub struct Segment<'a> {
	/// The path segment slice.
	pub(crate) data: &'a [u8],

	pub(crate) open: bool
}

impl<'a> Segment<'a> {
	pub fn dot() -> Segment<'static> {
		Segment {
			data: &[0x2e],
			open: false
		}
	}

	pub fn len(&self) -> usize {
		self.data.len()
	}

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

	pub fn is_open(&self) -> bool {
		self.open
	}

    /// Checks if the segment is empty.
	pub fn is_empty(&self) -> bool {
		self.data.is_empty()
	}

	pub fn open(&mut self) {
		self.open = true
	}
}

impl<'a> TryFrom<&'a str> for Segment<'a> {
	type Error = Error;

	fn try_from(str: &'a str) -> Result<Segment<'a>, Error> {
		let segment_len = parsing::parse_path_segment(str.as_ref(), 0)?;
		let data: &[u8] = str.as_ref();
		if segment_len < data.len() {
			if segment_len == data.len() - 1 && data[segment_len] == 0x2f {
				Ok(Segment {
					data: &data[0..segment_len],
					open: true
				})
			} else {
				Err(Error::InvalidSegment)
			}
		} else {
			Ok(Segment {
				data,
				open: false
			})
		}
	}
}

impl<'a> fmt::Display for Segment<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if self.open {
			write!(f, "{}/", self.as_str())
		} else {
			self.as_str().fmt(f)
		}
	}
}

impl<'a> fmt::Debug for Segment<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if self.open {
			write!(f, "{}/", self.as_str())
		} else {
			self.as_str().fmt(f)
		}
	}
}

impl<'a> cmp::PartialEq for Segment<'a> {
	fn eq(&self, other: &Segment) -> bool {
		self.open == other.open && self.as_pct_str() == other.as_pct_str()
	}
}

impl<'a> Eq for Segment<'a> { }

impl<'a> cmp::PartialEq<&'a str> for Segment<'a> {
	fn eq(&self, other: &&'a str) -> bool {
		self.as_pct_str() == *other
	}
}

impl<'a> PartialOrd for Segment<'a> {
	fn partial_cmp(&self, other: &Segment<'a>) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl<'a> Ord for Segment<'a> {
	fn cmp(&self, other: &Segment<'a>) -> Ordering {
		self.as_pct_str().cmp(other.as_pct_str())
	}
}

impl<'a> Hash for Segment<'a> {
	fn hash<H: Hasher>(&self, hasher: &mut H) {
		self.as_pct_str().hash(hasher)
	}
}
