use std::{fmt, cmp};
use std::cmp::{PartialOrd, Ord, Ordering};
use std::hash::{Hash, Hasher};
use std::convert::TryFrom;
use crate::parsing;
use super::Error;

#[derive(Clone, Copy)]
pub struct Port<'a> {
	/// The path slice.
	pub(crate) data: &'a [u8]
}

impl<'a> Port<'a> {
    pub fn as_ref(&self) -> &[u8] {
		self.data
	}

    /// Get the underlying port slice as a string slice.
	pub fn as_str(&self) -> &str {
		unsafe {
			std::str::from_utf8_unchecked(&self.data)
		}
	}

    /// Checks if the port is empty.
	pub fn is_empty(&self) -> bool {
		self.data.is_empty()
	}
}

impl<'a> TryFrom<&'a str> for Port<'a> {
	type Error = Error;

	fn try_from(str: &'a str) -> Result<Port<'a>, Error> {
		let port_len = parsing::parse_port(str.as_ref(), 0)?;
		if port_len < str.len() {
			Err(Error::InvalidPort)
		} else {
			Ok(Port {
				data: str.as_ref()
			})
		}
	}
}

impl<'a> fmt::Display for Port<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl<'a> fmt::Debug for Port<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl<'a> cmp::PartialEq for Port<'a> {
	fn eq(&self, other: &Port) -> bool {
		self.as_str() == other.as_str()
	}
}

impl<'a> Eq for Port<'a> { }

impl<'a> cmp::PartialEq<&'a str> for Port<'a> {
	fn eq(&self, other: &&'a str) -> bool {
		self.as_str() == *other
	}
}

impl<'a> PartialOrd for Port<'a> {
	fn partial_cmp(&self, other: &Port<'a>) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl<'a> Ord for Port<'a> {
	fn cmp(&self, other: &Port<'a>) -> Ordering {
		self.as_str().cmp(other.as_str())
	}
}

impl<'a> Hash for Port<'a> {
	fn hash<H: Hasher>(&self, hasher: &mut H) {
		self.as_str().hash(hasher)
	}
}
