use super::Error;
use crate::parsing;
use std::cmp::{Ord, Ordering, PartialOrd};
use std::convert::TryFrom;
use std::hash::{Hash, Hasher};
use std::{cmp, fmt};

#[derive(Clone, Copy)]
pub struct Scheme<'a> {
	/// The scheme slice.
	pub(crate) data: &'a [u8],
}

impl<'a> Scheme<'a> {
	/// Returns a reference to the byte representation of the scheme.
	#[inline]
	pub fn as_bytes(&self) -> &[u8] {
		self.data
	}

	/// Get the underlying scheme slice as a string slice.
	#[inline]
	pub fn as_str(&self) -> &str {
		unsafe { std::str::from_utf8_unchecked(&self.data) }
	}

	/// Checks if the scheme is empty.
	#[inline]
	pub fn is_empty(&self) -> bool {
		self.data.is_empty()
	}
}

impl<'a> AsRef<[u8]> for Scheme<'a> {
	#[inline]
	fn as_ref(&self) -> &[u8] {
		self.as_bytes()
	}
}

impl<'a> TryFrom<&'a str> for Scheme<'a> {
	type Error = Error;

	#[inline]
	fn try_from(str: &'a str) -> Result<Scheme<'a>, Error> {
		let scheme_len = parsing::parse_scheme(str.as_ref(), 0)?;
		if scheme_len < str.len() {
			Err(Error::InvalidScheme)
		} else {
			Ok(Scheme { data: str.as_ref() })
		}
	}
}

impl<'a> fmt::Display for Scheme<'a> {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl<'a> fmt::Debug for Scheme<'a> {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl<'a> cmp::PartialEq for Scheme<'a> {
	#[inline]
	fn eq(&self, other: &Scheme) -> bool {
		self.as_str() == other.as_str()
	}
}

impl<'a> Eq for Scheme<'a> {}

impl<'a> PartialOrd for Scheme<'a> {
	#[inline]
	fn partial_cmp(&self, other: &Scheme<'a>) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl<'a> Ord for Scheme<'a> {
	#[inline]
	fn cmp(&self, other: &Scheme<'a>) -> Ordering {
		self.as_str().cmp(other.as_str())
	}
}

impl<'a> cmp::PartialEq<&'a str> for Scheme<'a> {
	#[inline]
	fn eq(&self, other: &&'a str) -> bool {
		self.as_str() == *other
	}
}

impl<'a> Hash for Scheme<'a> {
	#[inline]
	fn hash<H: Hasher>(&self, hasher: &mut H) {
		self.as_str().hash(hasher)
	}
}
