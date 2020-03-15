use std::{fmt, cmp};
use std::cmp::{PartialOrd, Ord, Ordering};
use std::hash::{Hash, Hasher};
use std::convert::TryFrom;
use pct_str::PctStr;
use crate::parsing;
use super::Error;

#[derive(Clone, Copy)]
pub struct Query<'a> {
	/// The path slice.
	pub(crate) data: &'a [u8]
}

impl<'a> Query<'a> {
    pub fn as_ref(&self) -> &[u8] {
		self.data
	}

    /// Get the underlying query slice as a string slice.
	pub fn as_str(&self) -> &str {
		unsafe {
			std::str::from_utf8_unchecked(&self.data)
		}
	}

    /// Get the underlying query slice as a percent-encoded string slice.
	pub fn as_pct_str(&self) -> &PctStr {
		unsafe {
			PctStr::new_unchecked(self.as_str())
		}
	}

    /// Checks if the query is empty.
	pub fn is_empty(&self) -> bool {
		self.data.is_empty()
	}
}

impl<'a> TryFrom<&'a str> for Query<'a> {
	type Error = Error;

	fn try_from(str: &'a str) -> Result<Query<'a>, Error> {
		let query_len = parsing::parse_query(str.as_ref(), 0)?;
		if query_len < str.len() {
			Err(Error::InvalidQuery)
		} else {
			Ok(Query {
				data: str.as_ref()
			})
		}
	}
}

impl<'a> fmt::Display for Query<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl<'a> fmt::Debug for Query<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl<'a> cmp::PartialEq for Query<'a> {
	fn eq(&self, other: &Query) -> bool {
		self.as_pct_str() == other.as_pct_str()
	}
}

impl<'a> Eq for Query<'a> { }

impl<'a> cmp::PartialEq<&'a str> for Query<'a> {
	fn eq(&self, other: &&'a str) -> bool {
		self.as_str() == *other
	}
}

impl<'a> PartialOrd for Query<'a> {
	fn partial_cmp(&self, other: &Query<'a>) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl<'a> Ord for Query<'a> {
	fn cmp(&self, other: &Query<'a>) -> Ordering {
		self.as_pct_str().cmp(other.as_pct_str())
	}
}

impl<'a> Hash for Query<'a> {
	fn hash<H: Hasher>(&self, hasher: &mut H) {
		self.as_pct_str().hash(hasher)
	}
}
