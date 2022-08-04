use super::Error;
use crate::parsing;
use std::borrow::Borrow;
use std::cmp::{Ord, Ordering, PartialOrd};
use std::convert::TryFrom;
use std::hash::{Hash, Hasher};
use std::{cmp, fmt};

#[derive(Clone, Copy)]
pub struct Port<'a> {
	/// The Port slice.
	pub(crate) data: &'a [u8],
}

impl<'a> Port<'a> {
	/// Returns a reference to the byte representation of the port.
	#[inline]
	pub fn as_bytes(&self) -> &[u8] {
		self.data
	}

	/// Get the underlying port slice as a string slice.
	#[inline]
	pub fn as_str(&self) -> &str {
		unsafe { std::str::from_utf8_unchecked(self.data) }
	}

	/// Checks if the port is empty.
	#[inline]
	pub fn is_empty(&self) -> bool {
		self.data.is_empty()
	}
}

impl<'a> AsRef<str> for Port<'a> {
	#[inline(always)]
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

impl<'a> AsRef<[u8]> for Port<'a> {
	#[inline(always)]
	fn as_ref(&self) -> &[u8] {
		self.as_bytes()
	}
}

impl<'a> Borrow<str> for Port<'a> {
	#[inline(always)]
	fn borrow(&self) -> &str {
		self.as_str()
	}
}

impl<'a> Borrow<[u8]> for Port<'a> {
	#[inline(always)]
	fn borrow(&self) -> &[u8] {
		self.as_bytes()
	}
}

impl<'a> TryFrom<&'a str> for Port<'a> {
	type Error = Error;

	#[inline]
	fn try_from(str: &'a str) -> Result<Port<'a>, Error> {
		let port_len = parsing::parse_port(str.as_ref(), 0)?;
		if port_len < str.len() {
			Err(Error::InvalidPort)
		} else {
			Ok(Port { data: str.as_ref() })
		}
	}
}

impl<'a> fmt::Display for Port<'a> {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl<'a> fmt::Debug for Port<'a> {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl<'a> cmp::PartialEq for Port<'a> {
	#[inline]
	fn eq(&self, other: &Port) -> bool {
		self.as_str() == other.as_str()
	}
}

impl<'a> Eq for Port<'a> {}

impl<'a> cmp::PartialEq<&'a str> for Port<'a> {
	#[inline]
	fn eq(&self, other: &&'a str) -> bool {
		self.as_str() == *other
	}
}

impl<'a> PartialOrd for Port<'a> {
	#[inline]
	fn partial_cmp(&self, other: &Port<'a>) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl<'a> Ord for Port<'a> {
	#[inline]
	fn cmp(&self, other: &Port<'a>) -> Ordering {
		self.as_str().cmp(other.as_str())
	}
}

impl<'a> Hash for Port<'a> {
	#[inline]
	fn hash<H: Hasher>(&self, hasher: &mut H) {
		self.as_str().hash(hasher)
	}
}
