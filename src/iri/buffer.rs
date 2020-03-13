use std::ops::Deref;
use std::{fmt, cmp};
use std::hash::{Hash, Hasher};
use std::convert::TryFrom;
use pct_str::PctStr;
use crate::IriRefBuf;
use super::{Iri, Error, Scheme, Authority, AuthorityMut, Path, PathMut, Query, Fragment};

/// Owned IRI.
pub struct IriBuf(IriRefBuf);

impl IriBuf {
	pub fn new<S: AsRef<[u8]> + ?Sized>(buffer: &S) -> Result<IriBuf, Error> {
		let iri_ref = IriRefBuf::new(buffer)?;
		if iri_ref.scheme().is_some() {
			Ok(IriBuf(iri_ref))
		} else {
			Err(Error::Invalid)
		}
	}

	pub fn from_scheme(scheme: Scheme) -> IriBuf {
		let mut iri_ref = IriRefBuf::default();
		iri_ref.set_scheme(Some(scheme));
		IriBuf(iri_ref)
	}

	pub fn as_iri(&self) -> Iri {
		Iri(self.0.as_iri_ref())
	}

	pub fn scheme(&self) -> Scheme {
		self.0.scheme().unwrap()
	}

	/// Set the scheme of the IRI.
	pub fn set_scheme(&mut self, scheme: Scheme) {
		self.0.set_scheme(Some(scheme))
	}

	pub fn authority_mut(&mut self) -> AuthorityMut {
		self.0.authority_mut()
	}

	/// Set the authority of the IRI.
	///
	/// It must be a syntactically correct authority. If not,
	/// this method returns an error, and the IRI is unchanged.
	pub fn set_authority(&mut self, authority: Authority) {
		self.0.set_authority(authority)
	}

	pub fn path_mut(&mut self) -> PathMut {
		self.0.path_mut()
	}

	/// Set the IRI path.
	pub fn set_path(&mut self, path: Path) {
		self.0.set_path(path)
	}

	pub fn set_query(&mut self, query: Option<Query>) {
		self.0.set_query(query)
	}

	pub fn set_fragment(&mut self, fragment: Option<Fragment>) {
		self.0.set_fragment(fragment)
	}
}

impl Deref for IriBuf {
	type Target = IriRefBuf;

	fn deref(&self) -> &IriRefBuf {
		&self.0
	}
}

impl fmt::Display for IriBuf {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_iri().fmt(f)
	}
}

impl fmt::Debug for IriBuf {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_iri().fmt(f)
	}
}

impl<'a> From<Iri<'a>> for IriBuf {
	fn from(iri: Iri<'a>) -> IriBuf {
		let iri_ref_buf = iri.into();
		IriBuf(iri_ref_buf)
	}
}

impl<'a> From<&'a Iri<'a>> for IriBuf {
	fn from(iri: &'a Iri<'a>) -> IriBuf {
		let iri_ref_buf = iri.into();
		IriBuf(iri_ref_buf)
	}
}

impl TryFrom<IriRefBuf> for IriBuf {
	type Error = IriRefBuf;

	fn try_from(iri_ref: IriRefBuf) -> Result<IriBuf, IriRefBuf> {
		if iri_ref.p.scheme_len.is_some() {
			Ok(IriBuf(iri_ref))
		} else {
			Err(iri_ref)
		}
	}
}

impl<'a> cmp::PartialEq<Iri<'a>> for IriBuf {
	fn eq(&self, other: &Iri<'a>) -> bool {
		self.as_iri() == *other
	}
}

impl<'a> cmp::PartialEq<&'a str> for IriBuf {
	fn eq(&self, other: &&'a str) -> bool {
		if let Ok(other) = Iri::new(other) {
			self == &other
		} else {
			false
		}
	}
}
