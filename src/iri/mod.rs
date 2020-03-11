use crate::parsing::{self, ParsedIri, ParsedAuthority};

use std::{fmt, cmp};
use std::hash::{Hash, Hasher};
use std::ops::Range;
use log::*;
use pct_str::PctStr;

pub type Error = crate::parsing::Error;

/// IRI slice.
///
/// Note that in future versions, this will most likely become a custom dynamic sized type,
/// similar to `str`.
pub struct Iri<'a> {
	p: ParsedIri,
	data: &'a [u8],
}

impl<'a> Iri<'a> {
	pub fn new<S: AsRef<[u8]> + ?Sized>(buffer: &'a S) -> Result<Iri<'a>, Error> {
		Ok(Iri {
			data: buffer.as_ref(),
			p: ParsedIri::new(buffer)?
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

	pub fn scheme(&self) -> &PctStr {
		unsafe {
			PctStr::new_unchecked(std::str::from_utf8_unchecked(&self.data[0..self.p.scheme_len]))
		}
	}

	pub fn authority(&'a self) -> Authority<'a> {
		Authority {
			data: self.data,
			authority: &self.p.authority
		}
	}

	pub fn path(&self) -> Path<'a> {
		// if self.p.path_len > 0 {
		// 	unsafe {
		// 		let offset = self.p.authority.offset + self.p.authority.len();
		// 		Some(PctStr::new_unchecked(std::str::from_utf8_unchecked(&self.data[offset..(offset+self.p.path_len)])))
		// 	}
		// } else {
		// 	None
		// }
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
}

impl<'a> fmt::Display for Iri<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl<'a> fmt::Debug for Iri<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl<'a> cmp::PartialEq for Iri<'a> {
	fn eq(&self, other: &Iri) -> bool {
		self.scheme() == other.scheme() && self.fragment() == other.fragment() && self.authority() == other.authority() && self.path() == other.path() && self.query() == other.query()
	}
}

impl<'a> Eq for Iri<'a> { }

impl<'a> cmp::PartialEq<&'a str> for Iri<'a> {
	fn eq(&self, other: &&'a str) -> bool {
		if let Ok(other) = Iri::new(other) {
			self == &other
		} else {
			false
		}
	}
}

impl<'a> Hash for Iri<'a> {
	fn hash<H: Hasher>(&self, hasher: &mut H) {
		self.scheme().hash(hasher);
		self.authority().hash(hasher);
		self.path().hash(hasher);
		self.query().hash(hasher);
		self.fragment().hash(hasher);
	}
}
