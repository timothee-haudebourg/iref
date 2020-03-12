mod buffer;

use std::{fmt, cmp};
use std::hash::{Hash, Hasher};
// use log::*;
use pct_str::PctStr;

use crate::parsing::ParsedIriRef;
use crate::{Authority, Path, Error};

pub use self::buffer::*;

/// IRI reference slice.
///
/// Note that in future versions, this will most likely become a custom dynamic sized type,
/// similar to `str`.
pub struct IriRef<'a> {
	p: ParsedIriRef,
	data: &'a [u8],
}

impl<'a> IriRef<'a> {
	pub fn new<S: AsRef<[u8]> + ?Sized>(buffer: &'a S) -> Result<IriRef<'a>, Error> {
		Ok(IriRef {
			data: buffer.as_ref(),
			p: ParsedIriRef::new(buffer)?
		})
	}

	/// Length in bytes.
	pub fn len(&self) -> usize {
		self.p.len()
	}

	pub fn as_str(&self) -> &str {
		unsafe {
			std::str::from_utf8_unchecked(&self.data[0..self.len()])
		}
	}

	pub fn as_pct_str(&self) -> &PctStr {
		unsafe {
			PctStr::new_unchecked(self.as_str())
		}
	}

	pub fn scheme(&self) -> Option<&PctStr> {
		if let Some(scheme_len) = self.p.scheme_len {
			if scheme_len > 0 {
				unsafe {
					Some(PctStr::new_unchecked(std::str::from_utf8_unchecked(&self.data[0..scheme_len])))
				}
			} else {
				None
			}
		} else {
			None
		}
	}

	pub fn authority(&'a self) -> Authority<'a> {
		Authority {
			data: self.data,
			authority: &self.p.authority
		}
	}

	pub fn path(&self) -> Path<'a> {
		let offset = self.p.authority.offset + self.p.authority.len();
		Path {
			data: &self.data[offset..(offset+self.p.path_len)]
		}
	}

	pub fn query(&self) -> Option<&PctStr> {
		if let Some(len) = self.p.query_len {
			if len > 0 {
				unsafe {
					let offset = self.p.query_offset();
					Some(PctStr::new_unchecked(std::str::from_utf8_unchecked(&self.data[offset..(offset+len)])))
				}
			} else {
				None
			}
		} else {
			None
		}
	}

	pub fn fragment(&self) -> Option<&PctStr> {
		if let Some(len) = self.p.fragment_len {
			if len > 0 {
				unsafe {
					let offset = self.p.fragment_offset();
					Some(PctStr::new_unchecked(std::str::from_utf8_unchecked(&self.data[offset..(offset+len)])))
				}
			} else {
				None
			}
		} else {
			None
		}
	}

	/// Resolve the IRI reference against the given base IRI.
	pub fn resolve(&self, base_iri: &Iri) {
		if let Some(scheme) = self.scheme() {
			// ...
		} else {
			//
		}
	}
}

impl<'a> fmt::Display for IriRef<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl<'a> fmt::Debug for IriRef<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl<'a> cmp::PartialEq for IriRef<'a> {
	fn eq(&self, other: &IriRef) -> bool {
		self.scheme() == other.scheme() && self.fragment() == other.fragment() && self.authority() == other.authority() && self.path() == other.path() && self.query() == other.query()
	}
}

impl<'a> Eq for IriRef<'a> { }

impl<'a> cmp::PartialEq<&'a str> for IriRef<'a> {
	fn eq(&self, other: &&'a str) -> bool {
		if let Ok(other) = IriRef::new(other) {
			self == &other
		} else {
			false
		}
	}
}

impl<'a> Hash for IriRef<'a> {
	fn hash<H: Hasher>(&self, hasher: &mut H) {
		self.scheme().hash(hasher);
		self.authority().hash(hasher);
		self.path().hash(hasher);
		self.query().hash(hasher);
		self.fragment().hash(hasher);
	}
}
