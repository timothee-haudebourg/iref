mod buffer;

use std::{fmt, cmp};
use std::hash::{Hash, Hasher};
use std::convert::TryInto;
// use log::*;
use pct_str::PctStr;

use crate::parsing::ParsedIriRef;
use crate::{Scheme, Authority, Path, Query, Fragment, Error, Iri, IriBuf};

pub use self::buffer::*;

/// IRI reference slice.
///
/// Note that in future versions, this will most likely become a custom dynamic sized type,
/// similar to `str`.
#[derive(Clone, Copy)]
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

	pub fn scheme(&self) -> Option<Scheme> {
		if let Some(scheme_len) = self.p.scheme_len {
			Some(Scheme {
				data: &self.data[0..scheme_len]
			})
		} else {
			None
		}
	}

	pub fn authority(&self) -> Authority {
		Authority {
			data: &self.data[self.p.authority.offset..(self.p.authority.offset+self.p.authority.len())],
			p: self.p.authority
		}
	}

	pub fn path(&'a self) -> Path<'a> {
		let offset = self.p.authority.offset + self.p.authority.len();
		Path {
			data: &self.data[offset..(offset+self.p.path_len)]
		}
	}

	pub fn query(&self) -> Option<Query> {
		if let Some(len) = self.p.query_len {
			let offset = self.p.query_offset();
			Some(Query {
				data: &self.data[offset..(offset+len)]
			})
		} else {
			None
		}
	}

	pub fn fragment(&self) -> Option<Fragment> {
		if let Some(len) = self.p.fragment_len {
			let offset = self.p.fragment_offset();
			Some(Fragment {
				data: &self.data[offset..(offset+len)]
			})
		} else {
			None
		}
	}

	/// Resolve the IRI reference against the given base IRI.
	pub fn resolve<'b, Base: Into<Iri<'b>>>(&self, base_iri: Base) -> Result<IriBuf, Error> {
		let base_iri: Iri<'b> = base_iri.into();
		let mut resolved;

		if let Some(scheme) = self.scheme() {
			resolved = IriBuf::from_scheme(scheme);
			resolved.set_authority(self.authority());
			resolved.path_mut().symbolic_append(self.path())?;
			resolved.set_query(self.query());
		} else {
			resolved = IriBuf::from_scheme(base_iri.scheme());
			if self.authority().is_empty() {
				if self.path().is_relative() && self.path().is_empty() {
					resolved.set_path(base_iri.path());
					if let Some(query) = self.query() {
						resolved.set_query(Some(query));
					} else {
						resolved.set_query(base_iri.query());
					}
				} else {
					if self.path().is_absolute() {
						resolved.path_mut().symbolic_append(self.path())?;
					} else {
						if !base_iri.authority().is_empty() && base_iri.path().is_empty() {
							resolved.set_path("/".try_into().unwrap());
						} else {
							resolved.set_path(base_iri.path().directory());
						}
						resolved.path_mut().symbolic_append(self.path())?;
					}
					resolved.set_query(self.query());
				}
				resolved.set_authority(base_iri.authority());
			} else {
				resolved.set_authority(self.authority());
				resolved.path_mut().symbolic_append(self.path())?;
				resolved.set_query(self.query());
			}
		}

		resolved.set_fragment(self.fragment());
		Ok(resolved)
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
