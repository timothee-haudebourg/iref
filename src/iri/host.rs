use super::Error;
use crate::parsing;
use pct_str::PctStr;
use std::cmp::{Ord, Ordering, PartialOrd};
use std::convert::TryFrom;
use std::hash::{Hash, Hasher};
use std::{cmp, fmt};

#[derive(Clone, Copy)]
pub struct Host<'a> {
	/// The path slice.
	pub(crate) data: &'a [u8],
}

impl<'a> Host<'a> {
	/// Returns a reference to the byte representation of the host.
	#[inline]
	pub fn as_bytes(&self) -> &[u8] {
		self.data
	}

	/// Get the underlying host slice as a string slice.
	#[inline]
	pub fn as_str(&self) -> &str {
		unsafe { std::str::from_utf8_unchecked(&self.data) }
	}

	/// Get the underlying host slice as a percent-encoded string slice.
	#[inline]
	pub fn as_pct_str(&self) -> &PctStr {
		unsafe { PctStr::new_unchecked(self.as_str()) }
	}

	/// Checks if the host is empty.
	#[inline]
	pub fn is_empty(&self) -> bool {
		self.data.is_empty()
	}
}

impl<'a> AsRef<[u8]> for Host<'a> {
	#[inline]
	fn as_ref(&self) -> &[u8] {
		self.as_bytes()
	}
}

impl<'a> TryFrom<&'a str> for Host<'a> {
	type Error = Error;

	#[inline]
	fn try_from(str: &'a str) -> Result<Host<'a>, Error> {
		let host_len = parsing::parse_host(str.as_ref(), 0)?;
		if host_len < str.len() {
			Err(Error::InvalidHost)
		} else {
			Ok(Host { data: str.as_ref() })
		}
	}
}

impl<'a> fmt::Display for Host<'a> {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl<'a> fmt::Debug for Host<'a> {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl<'a> cmp::PartialEq for Host<'a> {
	#[inline]
	fn eq(&self, other: &Host) -> bool {
		self.as_pct_str() == other.as_pct_str()
	}
}

impl<'a> Eq for Host<'a> {}

impl<'a> cmp::PartialEq<&'a str> for Host<'a> {
	#[inline]
	fn eq(&self, other: &&'a str) -> bool {
		self.as_str() == *other
	}
}

impl<'a> PartialOrd for Host<'a> {
	#[inline]
	fn partial_cmp(&self, other: &Host<'a>) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl<'a> Ord for Host<'a> {
	#[inline]
	fn cmp(&self, other: &Host<'a>) -> Ordering {
		self.as_pct_str().cmp(other.as_pct_str())
	}
}

impl<'a> Hash for Host<'a> {
	#[inline]
	fn hash<H: Hasher>(&self, hasher: &mut H) {
		self.as_pct_str().hash(hasher)
	}
}
